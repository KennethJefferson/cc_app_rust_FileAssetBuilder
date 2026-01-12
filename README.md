# FileAssetBuilder

A fast, parallel file consolidation tool written in Rust. Recursively scans directories and merges all text-based files into a single output file, designed for creating AI-consumable representations of codebases.

## Features

- **Blacklist-based filtering** - Include everything except specified extensions
- **Parallel processing** - Concurrent file reading using rayon for maximum performance
- **Configurable exclusions** - Edit config.txt to customize which file types to skip
- **Auto-excludes common directories** - .git, node_modules, target, build, etc.
- **Cross-platform** - Works on Windows, macOS, and Linux
- **Deterministic output** - Files sorted alphabetically for consistent results

## Installation

### From Source

```bash
git clone <repository-url>
cd FileAssetBuilder
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
Directory Structure
================================================================

src/
  main.rs
  lib.rs
Cargo.toml
README.md

================================================================
Files
================================================================

================
File: src\main.rs
================
fn main() {
    println!("Hello, world!");
}

================
File: src\lib.rs
================
pub fn add(a: i32, b: i32) -> i32 {
    a + b
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

## Auto-Excluded Directories

The following directories are always skipped:

- `.git`, `.svn`, `.hg`
- `node_modules`
- `__pycache__`
- `.idea`, `.vs`, `.vscode`
- `target`, `build`, `dist`
- `cmake-build-debug`, `cmake-build-release`
- `.gradle`
- `vendor`

## Use Cases

- **AI/LLM context** - Create a single file representation of your codebase for AI analysis
- **Code review** - Consolidate changes for easier review
- **Documentation** - Generate a snapshot of project structure and contents
- **Archival** - Create text-based backups of source code

## Related Projects

- [DirTextFilePrinter](../Directory%20Text%20File%20Printer) - The original C implementation using a whitelist approach

## License

MIT
