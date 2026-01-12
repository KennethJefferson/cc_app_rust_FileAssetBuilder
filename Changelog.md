# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/username/FileAssetBuilder/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/username/FileAssetBuilder/releases/tag/v0.1.0
