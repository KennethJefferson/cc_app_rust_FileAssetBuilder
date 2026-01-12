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
"#;

pub struct Config {
    pub excluded_extensions: HashSet<String>,
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

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if !trimmed.starts_with('.') {
                eprintln!("Warning: Skipping invalid extension '{}' (must start with '.')", trimmed);
                continue;
            }

            excluded_extensions.insert(trimmed.to_lowercase());
        }

        println!("Loaded {} excluded extensions from config", excluded_extensions.len());
        Ok(Self { excluded_extensions })
    }

    fn with_defaults() -> Self {
        let excluded_extensions: HashSet<String> = DEFAULT_CONFIG
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('#') && trimmed.starts_with('.')
            })
            .map(|s| s.trim().to_lowercase())
            .collect();

        Self { excluded_extensions }
    }

    pub fn should_exclude(&self, extension: &str) -> bool {
        self.excluded_extensions.contains(&extension.to_lowercase())
    }
}
