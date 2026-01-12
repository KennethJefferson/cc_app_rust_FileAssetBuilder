use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::Config;

const EXCLUDED_DIRS: &[&str] = &[
    ".git",
    ".svn",
    ".hg",
    "node_modules",
    "__pycache__",
    ".idea",
    ".vs",
    ".vscode",
    "target",
    "build",
    "dist",
    "cmake-build-debug",
    "cmake-build-release",
    ".gradle",
    "vendor",
];

pub struct FileEntry {
    pub relative_path: String,
    pub content: String,
}

pub struct ScanResult {
    pub files: Vec<FileEntry>,
    pub directory_tree: String,
    pub stats: ScanStats,
}

#[derive(Default)]
pub struct ScanStats {
    pub total_directories: usize,
    pub total_files: usize,
    pub files_processed: usize,
    pub files_excluded: usize,
}

pub fn scan_directory(
    root: &Path,
    config: &Config,
    output_filename: &str,
) -> Result<ScanResult, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;

    let output_path = root.join(output_filename);

    println!("Scanning directory structure...");

    let mut stats = ScanStats::default();
    let mut file_paths: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| !is_excluded_dir(e.file_name().to_str().unwrap_or("")))
    {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Warning: Could not access entry: {}", e);
                continue;
            }
        };

        let path = entry.path();

        if path == output_path {
            continue;
        }

        if entry.file_type().is_dir() {
            stats.total_directories += 1;
        } else if entry.file_type().is_file() {
            stats.total_files += 1;

            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{}", e))
                .unwrap_or_default();

            if config.should_exclude(&extension) {
                stats.files_excluded += 1;
            } else {
                file_paths.push(path.to_path_buf());
            }
        }
    }

    println!(
        "Found {} directories, {} files ({} excluded by config)",
        stats.total_directories, stats.total_files, stats.files_excluded
    );

    let directory_tree = build_directory_tree(&root, config, output_filename);

    println!("Processing {} files in parallel...", file_paths.len());

    let files: Vec<FileEntry> = file_paths
        .par_iter()
        .filter_map(|path| {
            let relative = path
                .strip_prefix(&root)
                .ok()?
                .to_string_lossy()
                .to_string();

            match fs::read_to_string(path) {
                Ok(content) => {
                    let filename = path.file_name()?.to_str()?;
                    println!("Processing: {}", filename);
                    Some(FileEntry {
                        relative_path: relative,
                        content,
                    })
                }
                Err(e) => {
                    let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("unknown");
                    eprintln!("Warning: Could not read '{}': {}", filename, e);
                    None
                }
            }
        })
        .collect();

    let mut files = files;
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    stats.files_processed = files.len();

    Ok(ScanResult {
        files,
        directory_tree,
        stats,
    })
}

fn is_excluded_dir(name: &str) -> bool {
    EXCLUDED_DIRS.iter().any(|&excluded| name == excluded)
}

fn build_directory_tree(root: &Path, config: &Config, output_filename: &str) -> String {
    let mut tree = String::new();
    let output_path = root.join(output_filename);

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_excluded_dir(e.file_name().to_str().unwrap_or("")))
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        if path == output_path {
            continue;
        }

        if path == root {
            continue;
        }

        let depth = entry.depth();
        let indent = "  ".repeat(depth.saturating_sub(1));
        let name = entry.file_name().to_string_lossy();

        if entry.file_type().is_dir() {
            tree.push_str(&format!("{}{}/\n", indent, name));
        } else {
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{}", e))
                .unwrap_or_default();

            if !config.should_exclude(&extension) {
                tree.push_str(&format!("{}{}\n", indent, name));
            }
        }
    }

    tree
}
