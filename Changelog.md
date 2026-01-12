# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1] - 2026-01-12

### Changed

- Renamed "File List" section to "Directory List" in output
- Directory listing now uses tree-style format matching `tree /f` command:
  ```
  ├───folder1
  │   ├───file1.txt
  │   └───file2.txt
  └───folder2
      └───file3.txt
  ```

### Technical Details

- Added `TreeNode` struct in `scanner.rs` for building hierarchical tree structure
- Uses `BTreeMap` for sorted, deterministic output
- Box-drawing characters: `├───`, `└───`, `│   ` for tree visualization

## [0.3.0] - 2026-01-12

### Added

- Recursive directory scanning using `walkdir` crate
- Relative path display in file list and file headers (e.g., `src\main.rs` instead of just `main.rs`)

### Changed

- Scanner now traverses all subdirectories automatically
- File list shows full relative paths from input directory root

### Technical Details

- Added `walkdir 2.5` dependency for recursive traversal
- `scanner.rs` refactored to use `WalkDir` iterator

## [0.2.0] - 2026-01-12

### Changed

- **Breaking**: Removed recursive directory traversal - now only processes files in the specified directory (no subdirectories)
- Output section renamed from "Directory Structure" to "File List" (flat list of filenames)
- Dynamic worker pool sizing: `workers = ceil(file_count / 10)`
  - Example: 8 files = 1 worker, 35 files = 4 workers, 355 files = 36 workers
- Statistics output now shows "Workers used" instead of "Total directories scanned"

### Removed

- Recursive subdirectory scanning
- `walkdir` dependency (now uses `std::fs::read_dir`)
- Auto-exclusion of development directories (.git, node_modules, etc.) - no longer needed without recursion

### Technical Details

- Custom thread pool created per invocation with `rayon::ThreadPoolBuilder`
- Simplified codebase with single-level directory traversal

## [0.1.0] - 2026-01-12

### Added

- Initial release of FileAssetBuilder
- Recursive directory scanning with parallel file processing using rayon
- Blacklist-based file filtering via `config.txt`
- Default exclusions for common binary formats:
  - Video: .mp4, .mkv, .avi, .mov, .webm, .wmv, .flv, .m4v, .mpg, .mpeg, .3gp
  - Images: .png, .jpg, .jpeg, .gif, .bmp, .ico, .webp, .tiff, .tif, .psd, .raw, .svg
  - Audio: .mp3, .wav, .flac, .aac, .ogg, .wma, .m4a
  - Binaries: .exe, .dll, .so, .dylib, .bin, .o, .obj, .lib, .a, .pyc, .pyo, .class
  - Archives: .zip, .tar, .gz, .7z, .rar, .bz2, .xz, .iso
  - Documents: .pdf, .doc, .docx, .xls, .xlsx, .ppt, .pptx
  - Databases: .db, .sqlite, .sqlite3, .mdb
  - Fonts: .ttf, .otf, .woff, .woff2, .eot
- Auto-exclusion of common development directories:
  - .git, .svn, .hg
  - node_modules, __pycache__
  - .idea, .vs, .vscode
  - target, build, dist
  - cmake-build-debug, cmake-build-release
  - .gradle, vendor
- CLI interface with clap:
  - Required positional argument for input directory
  - Optional `-o, --output` flag for custom output filename
  - `--help` and `--version` flags
- Output format matching DirTextFilePrinter:
  - Header with generation timestamp
  - Directory structure tree
  - File contents with separators
- Auto-creation of `config.txt` with defaults on first run
- Cross-platform support (Windows, macOS, Linux)

### Technical Details

- Built with Rust 2021 edition
- Dependencies:
  - rayon 1.10 for parallel processing
  - walkdir 2.5 for directory traversal
  - clap 4.5 for CLI parsing
  - chrono 0.4 for timestamp formatting

[Unreleased]: https://github.com/KennethJefferson/cc_app_rust_FileAssetBuilder/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/KennethJefferson/cc_app_rust_FileAssetBuilder/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/KennethJefferson/cc_app_rust_FileAssetBuilder/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/KennethJefferson/cc_app_rust_FileAssetBuilder/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/KennethJefferson/cc_app_rust_FileAssetBuilder/releases/tag/v0.1.0
