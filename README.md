# FileAssetBuilder

A fast, parallel file consolidation tool written in Rust. Scans a directory and merges all text-based files into a single output file, designed for creating AI-consumable representations of codebases.

## Features

- **Blacklist-based filtering** - Include everything except specified extensions
- **Parallel processing** - Concurrent file reading with dynamic worker pool sizing
- **Configurable exclusions** - Edit config.txt to customize which file types to skip
- **Cross-platform** - Works on Windows, macOS, and Linux
- **Deterministic output** - Files sorted alphabetically for consistent results
- **Dynamic scaling** - Worker count adjusts based on file count: `ceil(files / 10)`

## Installation

### From Source

```bash
git clone https://github.com/KennethJefferson/cc_app_rust_FileAssetBuilder.git
cd cc_app_rust_FileAssetBuilder
cargo build --release
```

The binary will be at `target/release/fileassetbuilder` (or `fileassetbuilder.exe` on Windows).

## Quick Start

```bash
# Scan a directory, output to directory/fileassets.txt
fileassetbuilder ./my-project

# Custom output filename
fileassetbuilder ./my-project -o codebase.txt
```

## Output Format

```
This file is a merged representation of the directory, combining all text-based files into a single document.
Generated on: 2026-01-12 10:30:00

================================================================
File List
================================================================

Cargo.toml
README.md
main.rs

================================================================
Files
================================================================

================
File: Cargo.toml
================
[package]
name = "my-project"
version = "0.1.0"

================
File: README.md
================
# My Project
...

================
File: main.rs
================
fn main() {
    println!("Hello, world!");
}

...
```

## Configuration

On first run, a `config.txt` file is created next to the executable with default exclusions:

- **Video**: .mp4, .mkv, .avi, .mov, .webm, etc.
- **Images**: .png, .jpg, .gif, .bmp, .ico, .svg, etc.
- **Audio**: .mp3, .wav, .flac, .aac, .ogg, etc.
- **Binaries**: .exe, .dll, .so, .bin, .o, .pyc, etc.
- **Archives**: .zip, .tar, .gz, .7z, .rar, etc.
- **Documents**: .pdf, .doc, .docx, .xls, .xlsx, etc.

Edit this file to customize which extensions are excluded.

## Worker Scaling

The tool dynamically adjusts the number of parallel workers based on file count:

| Files | Workers |
|-------|---------|
| 1-10  | 1       |
| 11-20 | 2       |
| 21-30 | 3       |
| 100   | 10      |
| 355   | 36      |

Formula: `workers = ceil(file_count / 10)`

## Use Cases

- **AI/LLM context** - Create a single file representation of your codebase for AI analysis
- **Code review** - Consolidate changes for easier review
- **Documentation** - Generate a snapshot of project structure and contents
- **Archival** - Create text-based backups of source code

## Notes

- Only processes files in the specified directory (no subdirectory recursion)
- Subdirectories are skipped entirely
- Output file is always written to the input directory root

## Related Projects

- [DirTextFilePrinter](../Directory%20Text%20File%20Printer) - The original C implementation with recursive scanning and whitelist approach

## License

MIT
