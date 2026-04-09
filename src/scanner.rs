use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::WalkDir;

use crate::config::Config;

pub struct FileEntry {
    pub absolute_path: String,
    pub relative_path: String,
    pub content: String,
}

pub struct CourseGroup {
    pub name: String,
    pub files: Vec<FileEntry>,
}

pub struct ScanResult {
    pub courses: Vec<CourseGroup>,
    pub file_list: String,
    pub stats: ScanStats,
}

#[derive(Default)]
pub struct ScanStats {
    pub total_files: usize,
    pub files_processed: usize,
    pub files_excluded: usize,
    pub folders_skipped: usize,
    pub worker_count: usize,
    pub courses_found: usize,
}

pub fn scan_directory(
    root: &Path,
    config: &Config,
    output_filename: &str,
    verbose: bool,
) -> Result<ScanResult, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;

    let output_path = root.join(output_filename);

    let scan_spinner = ProgressBar::new_spinner();
    scan_spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    scan_spinner.enable_steady_tick(Duration::from_millis(80));
    scan_spinner.set_message("Scanning directory...");

    let mut stats = ScanStats::default();
    let mut file_paths: Vec<(PathBuf, String)> = Vec::new();

    let mut walker = WalkDir::new(&root).into_iter();
    loop {
        let entry = match walker.next() {
            Some(Ok(e)) => e,
            Some(Err(_)) => continue,
            None => break,
        };

        let path = entry.path().to_path_buf();

        // Prune excluded directories (but never the root itself)
        if entry.file_type().is_dir() {
            if path != root {
                let name = entry.file_name().to_string_lossy();
                if config.should_exclude_dir(&name) {
                    walker.skip_current_dir();
                    stats.folders_skipped += 1;
                }
            }
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

    scan_spinner.finish_with_message(format!(
        "Scanned: {} files found, {} excluded by extension, {} folders skipped",
        stats.total_files, stats.files_excluded, stats.folders_skipped
    ));

    // Build simple file list
    let file_list = build_file_list(&file_paths);

    // Calculate worker count: ceil(file_count / 10), minimum 1, maximum 50
    let files_to_process = file_paths.len();
    let worker_count = if files_to_process == 0 {
        1
    } else {
        (((files_to_process as f64) / 10.0).ceil() as usize).min(50)
    };
    stats.worker_count = worker_count;

    // Create custom thread pool with calculated worker count
    let pool = ThreadPoolBuilder::new()
        .num_threads(worker_count)
        .build()
        .map_err(|e| format!("Failed to create thread pool: {}", e))?;

    let progress = ProgressBar::new(files_to_process as u64);
    progress.set_style(
        ProgressStyle::with_template(
            "Reading files {bar:40.cyan/blue} {pos}/{len} ({percent}%) {msg}",
        )
        .unwrap()
        .progress_chars("━━─"),
    );
    progress.set_message(format!("{} workers", worker_count));

    // Process files in parallel using custom pool
    let files: Vec<FileEntry> = pool.install(|| {
        file_paths
            .par_iter()
            .filter_map(|(path, relative)| {
                let result = match fs::read_to_string(path) {
                    Ok(content) => {
                        if verbose {
                            progress.println(format!("Processing: {}", relative));
                        }
                        let abs_path = path.to_string_lossy().to_string();
                        // Strip Windows extended-length path prefix
                        let abs_path = abs_path.strip_prefix(r"\\?\").unwrap_or(&abs_path).to_string();
                        Some(FileEntry {
                            absolute_path: abs_path,
                            relative_path: relative.clone(),
                            content,
                        })
                    }
                    Err(e) => {
                        progress.println(format!("Warning: Could not read '{}': {}", relative, e));
                        None
                    }
                };
                progress.inc(1);
                result
            })
            .collect()
    });

    progress.finish_with_message("done");

    let mut files = files;
    files.sort_by(|a, b| a.absolute_path.cmp(&b.absolute_path));

    let courses = group_into_courses(files);
    stats.courses_found = courses.len();
    stats.files_processed = courses.iter().map(|c| c.files.len()).sum();

    Ok(ScanResult {
        courses,
        file_list,
        stats,
    })
}

fn group_into_courses(files: Vec<FileEntry>) -> Vec<CourseGroup> {
    let mut course_map: BTreeMap<String, Vec<FileEntry>> = BTreeMap::new();

    for file in files {
        let course_name = file
            .relative_path
            .split(|c: char| c == '\\' || c == '/')
            .next()
            .unwrap_or("Uncategorized")
            .to_string();

        course_map.entry(course_name).or_default().push(file);
    }

    course_map
        .into_iter()
        .map(|(name, files)| CourseGroup { name, files })
        .collect()
}

#[derive(Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    is_file: bool,
}

impl TreeNode {
    fn insert(&mut self, parts: &[&str]) {
        if parts.is_empty() {
            return;
        }
        let child = self.children.entry(parts[0].to_string()).or_default();
        if parts.len() == 1 {
            child.is_file = true;
        } else {
            child.insert(&parts[1..]);
        }
    }

    fn render(&self, prefix: &str, output: &mut String) {
        let entries: Vec<_> = self.children.iter().collect();
        for (i, (name, node)) in entries.iter().enumerate() {
            let is_last = i == entries.len() - 1;
            let connector = if is_last { "└───" } else { "├───" };
            output.push_str(prefix);
            output.push_str(connector);
            output.push_str(name);
            output.push('\n');

            if !node.children.is_empty() {
                let new_prefix = if is_last {
                    format!("{}    ", prefix)
                } else {
                    format!("{}│   ", prefix)
                };
                node.render(&new_prefix, output);
            }
        }
    }
}

fn build_file_list(file_paths: &[(PathBuf, String)]) -> String {
    let mut root = TreeNode::default();

    for (_, rel) in file_paths {
        let parts: Vec<&str> = rel.split(|c| c == '\\' || c == '/').collect();
        root.insert(&parts);
    }

    let mut output = String::new();
    root.render("", &mut output);

    // Remove trailing newline if present
    if output.ends_with('\n') {
        output.pop();
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_into_courses() {
        let files = vec![
            FileEntry {
                absolute_path: "C:\\Courses\\Rust\\01\\notes.txt".to_string(),
                relative_path: "Rust\\01\\notes.txt".to_string(),
                content: "rust notes".to_string(),
            },
            FileEntry {
                absolute_path: "C:\\Courses\\Python\\01\\notes.txt".to_string(),
                relative_path: "Python\\01\\notes.txt".to_string(),
                content: "python notes".to_string(),
            },
            FileEntry {
                absolute_path: "C:\\Courses\\Rust\\02\\notes.txt".to_string(),
                relative_path: "Rust\\02\\notes.txt".to_string(),
                content: "more rust".to_string(),
            },
        ];

        let courses = group_into_courses(files);

        assert_eq!(courses.len(), 2);
        assert_eq!(courses[0].name, "Python");
        assert_eq!(courses[0].files.len(), 1);
        assert_eq!(courses[1].name, "Rust");
        assert_eq!(courses[1].files.len(), 2);
    }
}
