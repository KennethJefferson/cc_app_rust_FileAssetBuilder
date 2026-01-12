use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::Config;

pub struct FileEntry {
    pub relative_path: String,
    pub content: String,
}

pub struct ScanResult {
    pub files: Vec<FileEntry>,
    pub file_list: String,
    pub stats: ScanStats,
}

#[derive(Default)]
pub struct ScanStats {
    pub total_files: usize,
    pub files_processed: usize,
    pub files_excluded: usize,
    pub worker_count: usize,
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

    println!("Scanning directory...");

    let mut stats = ScanStats::default();
    let mut file_paths: Vec<(PathBuf, String)> = Vec::new();

    for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();

        // Skip directories - only process files
        if path.is_dir() {
            continue;
        }

        // Skip the output file
        if path == output_path {
            continue;
        }

        stats.total_files += 1;

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();

        if config.should_exclude(&extension) {
            stats.files_excluded += 1;
        } else {
            // Store relative path from root
            let relative = path
                .strip_prefix(&root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.file_name().unwrap_or_default().to_string_lossy().to_string());
            file_paths.push((path, relative));
        }
    }

    println!(
        "Found {} files ({} excluded by config)",
        stats.total_files, stats.files_excluded
    );

    // Build simple file list
    let file_list = build_file_list(&file_paths);

    // Calculate worker count: ceil(file_count / 10), minimum 1
    let files_to_process = file_paths.len();
    let worker_count = if files_to_process == 0 {
        1
    } else {
        ((files_to_process as f64) / 10.0).ceil() as usize
    };
    stats.worker_count = worker_count;

    println!(
        "Processing {} files with {} workers...",
        files_to_process, worker_count
    );

    // Create custom thread pool with calculated worker count
    let pool = ThreadPoolBuilder::new()
        .num_threads(worker_count)
        .build()
        .map_err(|e| format!("Failed to create thread pool: {}", e))?;

    // Process files in parallel using custom pool
    let files: Vec<FileEntry> = pool.install(|| {
        file_paths
            .par_iter()
            .filter_map(|(path, relative)| {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        println!("Processing: {}", relative);
                        Some(FileEntry {
                            relative_path: relative.clone(),
                            content,
                        })
                    }
                    Err(e) => {
                        eprintln!("Warning: Could not read '{}': {}", relative, e);
                        None
                    }
                }
            })
            .collect()
    });

    let mut files = files;
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    stats.files_processed = files.len();

    Ok(ScanResult {
        files,
        file_list,
        stats,
    })
}

fn build_file_list(file_paths: &[(PathBuf, String)]) -> String {
    let mut names: Vec<&str> = file_paths.iter().map(|(_, rel)| rel.as_str()).collect();
    names.sort();
    names.join("\n")
}
