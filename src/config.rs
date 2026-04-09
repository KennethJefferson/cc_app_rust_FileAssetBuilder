use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
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
"#;

pub struct Config {
    pub excluded_extensions: HashSet<String>,
    pub excluded_folders: GlobSet,
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
        let file = File::open(config_path)?;
        let reader = BufReader::new(file);
        let mut excluded_extensions = HashSet::new();
        let mut folder_patterns: Vec<String> = Vec::new();
        let mut in_folders_section = false;

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.eq_ignore_ascii_case("[folders]") {
                in_folders_section = true;
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                in_folders_section = false;
                continue;
            }

            if in_folders_section {
                folder_patterns.push(trimmed.to_string());
            } else {
                if !trimmed.starts_with('.') {
                    eprintln!(
                        "Warning: Skipping invalid extension '{}' (must start with '.')",
                        trimmed
                    );
                    continue;
                }
                excluded_extensions.insert(trimmed.to_lowercase());
            }
        }

        let excluded_folders = build_glob_set(&folder_patterns);

        println!(
            "Loaded {} excluded extensions and {} folder patterns from config",
            excluded_extensions.len(),
            folder_patterns.len()
        );

        Ok(Self {
            excluded_extensions,
            excluded_folders,
        })
    }

    fn with_defaults() -> Self {
        let mut excluded_extensions = HashSet::new();
        let mut folder_patterns: Vec<String> = Vec::new();
        let mut in_folders_section = false;

        for line in DEFAULT_CONFIG.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.eq_ignore_ascii_case("[folders]") {
                in_folders_section = true;
                continue;
            }

            if in_folders_section {
                folder_patterns.push(trimmed.to_string());
            } else if trimmed.starts_with('.') {
                excluded_extensions.insert(trimmed.to_lowercase());
            }
        }

        let excluded_folders = build_glob_set(&folder_patterns);

        Self {
            excluded_extensions,
            excluded_folders,
        }
    }

    pub fn should_exclude(&self, extension: &str) -> bool {
        self.excluded_extensions.contains(&extension.to_lowercase())
    }

    pub fn should_exclude_dir(&self, dir_name: &str) -> bool {
        self.excluded_folders.is_match(dir_name)
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
                eprintln!("Warning: Invalid folder pattern '{}': {}", pattern, e);
            }
        }
    }
    builder.build().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to build folder exclusion set: {}", e);
        GlobSet::empty()
    })
}
