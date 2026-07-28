use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const DEFAULT_CONFIG: &str = r#"# FileAssetBuilder Configuration
# List file extensions to EXCLUDE from processing (one per line)
# Lines starting with # are comments
# Extensions must include the dot (.)

# Video files
.mp4
.mkv
.avi
.mov
.webm
.wmv
.flv
.m4v
.mpg
.mpeg
.3gp

# Image files
.png
.jpg
.jpeg
.gif
.bmp
.ico
.webp
.tiff
.tif
.psd
.raw
.svg

# Audio files
.mp3
.wav
.flac
.aac
.ogg
.wma
.m4a

# Binary/Executable files
.exe
.dll
.so
.dylib
.bin
.o
.obj
.lib
.a
.pyc
.pyo
.class

# Archive files
.zip
.tar
.gz
.7z
.rar
.bz2
.xz
.iso

# Database files
.db
.sqlite
.sqlite3
.mdb

# Document files (often binary)
.pdf
.doc
.docx
.xls
.xlsx
.ppt
.pptx

# Font files
.ttf
.otf
.woff
.woff2
.eot

# Other binary files
.dat
.pak
.cache

# Folder names/patterns to EXCLUDE entirely (wildcards supported: *, ?)
# Matches on folder name, not full path. Matching folders and their
# contents are skipped.
[folders]
node_modules
.git
.svn
.hg
.vscode
.idea
__pycache__
.pytest_cache
.mypy_cache
.cache
target
dist
build
out
bin
obj
.next
.nuxt
.turbo
.parcel-cache
coverage
.nyc_output

# Exact filenames or glob patterns to EXCLUDE (wildcards supported: *, ?)
# Matches on file name, not full path.
[filenames]
package-lock.json
yarn.lock
pnpm-lock.yaml
*.min.js
*.min.css
*.bundle.js
*.bundle.css
*.map
"#;

pub struct Config {
    pub excluded_extensions: HashSet<String>,
    pub excluded_folders: GlobSet,
    pub excluded_filenames: GlobSet,
}

struct ParsedPatterns {
    extensions: HashSet<String>,
    folder_patterns: Vec<String>,
    filename_patterns: Vec<String>,
}

#[derive(PartialEq)]
enum Section {
    Extensions,
    Folders,
    Filenames,
}

fn parse_lines<'a>(lines: impl Iterator<Item = &'a str>) -> ParsedPatterns {
    let mut parsed = ParsedPatterns {
        extensions: HashSet::new(),
        folder_patterns: Vec::new(),
        filename_patterns: Vec::new(),
    };
    let mut section = Section::Extensions;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.eq_ignore_ascii_case("[folders]") {
            section = Section::Folders;
            continue;
        }
        if trimmed.eq_ignore_ascii_case("[filenames]") {
            section = Section::Filenames;
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = Section::Extensions;
            continue;
        }

        match section {
            Section::Extensions => {
                if !trimmed.starts_with('.') {
                    eprintln!(
                        "Warning: Skipping invalid extension '{}' (must start with '.')",
                        trimmed
                    );
                    continue;
                }
                parsed.extensions.insert(trimmed.to_lowercase());
            }
            Section::Folders => parsed.folder_patterns.push(trimmed.to_string()),
            Section::Filenames => parsed.filename_patterns.push(trimmed.to_string()),
        }
    }

    parsed
}

impl Config {
    pub fn load(config_path: &Path) -> Self {
        if !config_path.exists() {
            println!("Config file not found. Creating default config...");
            if let Err(e) = Self::create_default(config_path) {
                eprintln!("Warning: Could not create config file: {}", e);
                return Self::with_defaults();
            }
        }

        match Self::parse_config(config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: Could not load config: {}. Using defaults.", e);
                Self::with_defaults()
            }
        }
    }

    fn create_default(config_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(config_path)?;
        file.write_all(DEFAULT_CONFIG.as_bytes())?;
        Ok(())
    }

    fn parse_config(config_path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(config_path)?;
        let parsed = parse_lines(content.lines());

        println!(
            "Loaded {} excluded extensions, {} folder patterns, {} filename patterns from config",
            parsed.extensions.len(),
            parsed.folder_patterns.len(),
            parsed.filename_patterns.len()
        );

        Ok(Self::from_parsed(parsed))
    }

    fn with_defaults() -> Self {
        Self::from_parsed(parse_lines(DEFAULT_CONFIG.lines()))
    }

    fn from_parsed(parsed: ParsedPatterns) -> Self {
        Self {
            excluded_extensions: parsed.extensions,
            excluded_folders: build_glob_set(&parsed.folder_patterns),
            excluded_filenames: build_glob_set(&parsed.filename_patterns),
        }
    }

    pub fn should_exclude(&self, extension: &str) -> bool {
        self.excluded_extensions.contains(&extension.to_lowercase())
    }

    pub fn should_exclude_dir(&self, dir_name: &str) -> bool {
        self.excluded_folders.is_match(dir_name)
    }

    pub fn should_exclude_filename(&self, file_name: &str) -> bool {
        self.excluded_filenames.is_match(file_name)
    }
}

fn build_glob_set(patterns: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        match Glob::new(pattern) {
            Ok(glob) => {
                builder.add(glob);
            }
            Err(e) => {
                eprintln!("Warning: Invalid exclusion pattern '{}': {}", pattern, e);
            }
        }
    }
    builder.build().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to build exclusion set: {}", e);
        GlobSet::empty()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lines_reads_all_three_sections() {
        let input = "\
# comment
.mp4

[folders]
node_modules

[filenames]
package-lock.json
*.min.js
";
        let parsed = parse_lines(input.lines());
        assert!(parsed.extensions.contains(".mp4"));
        assert_eq!(parsed.folder_patterns, vec!["node_modules".to_string()]);
        assert_eq!(
            parsed.filename_patterns,
            vec!["package-lock.json".to_string(), "*.min.js".to_string()]
        );
    }

    #[test]
    fn default_config_includes_filename_patterns() {
        let parsed = parse_lines(DEFAULT_CONFIG.lines());
        assert!(parsed.filename_patterns.contains(&"package-lock.json".to_string()));
        assert!(parsed.filename_patterns.contains(&"*.min.js".to_string()));
    }

    #[test]
    fn should_exclude_filename_matches_exact_and_glob() {
        let config = Config {
            excluded_extensions: HashSet::new(),
            excluded_folders: GlobSet::empty(),
            excluded_filenames: build_glob_set(&[
                "package-lock.json".to_string(),
                "*.min.js".to_string(),
            ]),
        };
        assert!(config.should_exclude_filename("package-lock.json"));
        assert!(config.should_exclude_filename("app.min.js"));
        assert!(!config.should_exclude_filename("app.js"));
        assert!(!config.should_exclude_filename("notes.md"));
    }
}
