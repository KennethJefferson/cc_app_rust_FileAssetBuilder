# Usage Guide

## Command Line Interface

```
fileassetbuilder [OPTIONS] <INPUT_DIRECTORY>

Arguments:
  <INPUT_DIRECTORY>  Input directory to scan

Options:
  -o, --output <OUTPUT>  Output filename (written to input directory root) [default: fileassets.txt]
  -h, --help             Print help
  -V, --version          Print version
```

## Examples

### Basic Usage

Scan a directory and create `fileassets.txt` in that directory:

```bash
fileassetbuilder ./my-project
```

Output: `./my-project/fileassets.txt`

### Custom Output Filename

Specify a different output filename with `-o`:

```bash
fileassetbuilder ./my-project -o snapshot.txt
```

Output: `./my-project/snapshot.txt`

### Absolute Paths

Works with absolute paths on any platform:

```bash
# Windows
fileassetbuilder C:\Users\dev\projects\myapp

# Linux/macOS
fileassetbuilder /home/dev/projects/myapp
```

### Multiple Directories

Process multiple directories separately:

```bash
fileassetbuilder ./frontend -o frontend-assets.txt
fileassetbuilder ./backend -o backend-assets.txt
fileassetbuilder ./shared -o shared-assets.txt
```

## Configuration

### Config File Location

The `config.txt` file is loaded from the same directory as the executable. If not found, it's automatically created with sensible defaults.

### Config File Format

```
# Lines starting with # are comments
# One extension per line, must include the dot

# Video files
.mp4
.mkv
.avi

# Image files
.png
.jpg
.gif

# Add custom exclusions below
.myformat
```

### Customizing Exclusions

**To exclude additional file types:**

1. Open `config.txt` next to the executable
2. Add the extension (with leading dot) on a new line
3. Save and run again

**To include a file type that's currently excluded:**

1. Open `config.txt`
2. Remove or comment out the extension line
3. Save and run again

### Example: Include SVG Files

By default, `.svg` is excluded. To include SVG files:

```
# config.txt
# .svg    <- comment out or delete this line
```

### Example: Exclude Log Files

To exclude `.log` files:

```
# config.txt
# Add at the end:
.log
```

## Output Structure

The generated output file has three sections:

### 1. Header

```
This file is a merged representation of the directory, combining all text-based files into a single document.
Generated on: 2026-01-12 10:30:00
```

### 2. Directory List

```
================================================================
Directory List
================================================================

├───Cargo.toml
├───README.md
└───src
    ├───config.rs
    └───main.rs
```

- Tree-style display matching `tree /f` command output
- Directories and files shown hierarchically with box-drawing characters
- Only shows files that will be processed (non-excluded)

### 3. File Contents

```
================================================================
Files
================================================================

================
File: "C:\projects\my-project\Cargo.toml"
================
[package]
name = "my-project"
version = "0.1.0"

================
File: "C:\projects\my-project\src\main.rs"
================
fn main() {
    println!("Hello, world!");
}
```

- Each file has a header with its full absolute path in quotes
- Files are sorted alphabetically by path
- Original content is preserved (including whitespace)

## Performance

FileAssetBuilder uses parallel processing with dynamic worker scaling:

- Directory listing is sequential
- File content reading is parallel using rayon
- Worker count: `ceil(file_count / 10)`
- Output writing is sequential (single file)

| Files | Workers |
|-------|---------|
| 1-10  | 1       |
| 11-20 | 2       |
| 50    | 5       |
| 100   | 10      |
| 355   | 36      |

## Important Notes

- **Recursive scanning** - All subdirectories are traversed automatically
- **Tree-style listing** - Directory structure displayed like `tree /f` with box-drawing characters
- **Absolute paths** - File content headers show full absolute paths in double quotes
- **Output location** - Always written to the input directory root
- **Skip if exists** - If the output file already exists, the tool skips processing and exits early

## Troubleshooting

### "Could not read file" warnings

Some files may fail to read due to:
- Encoding issues (non-UTF8 content)
- Permission denied
- File locked by another process

These files are skipped with a warning. Other files continue processing.

### Missing files in output

Check if the file extension is in `config.txt`. Remove it to include those files.

### Directory not found

Ensure the path exists and is accessible. Use absolute paths if relative paths cause issues.

### Binary garbage in output

If you see garbled content, the file is likely binary but has an extension not in the exclusion list. Add the extension to `config.txt`.
