# Course Grouping Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Group files by top-level subdirectory (course) in the output, with parseable delimiters for external extraction tools.

**Architecture:** Keep existing walkdir + rayon pipeline. Add `relative_path` to `FileEntry`, group files by first path component after parallel read, write output with course-level start/end delimiters. No new dependencies.

**Tech Stack:** Rust, walkdir, rayon, clap, chrono (all existing)

---

## File Structure

- Modify: `src/scanner.rs` - Add `CourseGroup` struct, add `relative_path` to `FileEntry`, change `ScanResult.files` to `ScanResult.courses`, add grouping function
- Modify: `src/output.rs` - Course delimiter formatting, replace `write_files` with `write_courses`
- Modify: `src/main.rs` - Update stats output to include course count
- Modify: `Cargo.toml` - Bump version to 0.5.0, rename binary to `fileassetsbuildercourse`

---

### Task 1: Update scanner data structures

**Files:**
- Modify: `src/scanner.rs:10-19` (structs)

- [ ] **Step 1: Add `relative_path` to `FileEntry` and create `CourseGroup`**

Replace the existing `FileEntry` and `ScanResult` structs:

```rust
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
```

- [ ] **Step 2: Add `courses_found` to `ScanStats`**

```rust
#[derive(Default)]
pub struct ScanStats {
    pub total_files: usize,
    pub files_processed: usize,
    pub files_excluded: usize,
    pub worker_count: usize,
    pub courses_found: usize,
}
```

---

### Task 2: Add grouping logic to scanner

**Files:**
- Modify: `src/scanner.rs` - Add `group_into_courses` function, update `scan_directory` return

- [ ] **Step 1: Write the `group_into_courses` function**

Add after the `scan_directory` function (before `TreeNode`):

```rust
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
```

`BTreeMap` is already imported -- courses will be sorted alphabetically. Files within each course are already sorted from the prior `files.sort_by` call.

- [ ] **Step 2: Update the parallel read to store `relative_path`**

In `scan_directory`, the rayon `filter_map` closure at line 111-131. Change the `Some(FileEntry {...})` to include `relative_path`:

```rust
Some(FileEntry {
    absolute_path: abs_path,
    relative_path: relative.clone(),
    content,
})
```

- [ ] **Step 3: Update the return value of `scan_directory`**

Replace the final block (lines 134-143):

```rust
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
```

- [ ] **Step 4: Add unit test for grouping logic**

Add at the bottom of `scanner.rs`:

```rust
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
```

- [ ] **Step 5: Run test**

Run: `cargo test`
Expected: PASS - grouping produces 2 courses sorted alphabetically, correct file counts.

- [ ] **Step 6: Commit**

```bash
git add src/scanner.rs
git commit -m "feat: group scanned files by top-level directory (course)"
```

---

### Task 3: Update output format with course delimiters

**Files:**
- Modify: `src/output.rs` - Replace `write_files` with `write_courses`, add delimiter formatting

- [ ] **Step 1: Add course delimiter formatting functions**

Add after the `FILE_SEPARATOR` constant:

```rust
fn format_course_start(name: &str) -> String {
    let prefix = format!("===[ COURSE: {} ]", name);
    let padding = 64_usize.saturating_sub(prefix.len());
    format!("{}{}", prefix, "=".repeat(padding))
}

fn format_course_end(name: &str) -> String {
    let prefix = format!("===[ END COURSE: {} ]", name);
    let padding = 64_usize.saturating_sub(prefix.len());
    format!("{}{}", prefix, "=".repeat(padding))
}
```

- [ ] **Step 2: Replace `write_files` with `write_courses`**

Delete the `write_files` function (lines 52-67). Replace with:

```rust
fn write_courses(writer: &mut BufWriter<File>, result: &ScanResult) -> Result<(), String> {
    writeln!(writer, "{}", SEPARATOR).map_err(|e| format!("Write error: {}", e))?;
    writeln!(writer, "Courses").map_err(|e| format!("Write error: {}", e))?;
    writeln!(writer, "{}\n", SEPARATOR).map_err(|e| format!("Write error: {}", e))?;

    for course in &result.courses {
        writeln!(writer, "{}", format_course_start(&course.name))
            .map_err(|e| format!("Write error: {}", e))?;

        for file in &course.files {
            writeln!(writer, "{}", FILE_SEPARATOR)
                .map_err(|e| format!("Write error: {}", e))?;
            writeln!(writer, "File: \"{}\"", file.absolute_path)
                .map_err(|e| format!("Write error: {}", e))?;
            writeln!(writer, "{}", FILE_SEPARATOR)
                .map_err(|e| format!("Write error: {}", e))?;
            writeln!(writer, "{}", file.content)
                .map_err(|e| format!("Write error: {}", e))?;
            writeln!(writer).map_err(|e| format!("Write error: {}", e))?;
        }

        writeln!(writer, "{}\n", format_course_end(&course.name))
            .map_err(|e| format!("Write error: {}", e))?;
    }

    Ok(())
}
```

- [ ] **Step 3: Update `write_output` to call `write_courses`**

Change line 19 from:

```rust
write_files(&mut writer, result)?;
```

to:

```rust
write_courses(&mut writer, result)?;
```

- [ ] **Step 4: Build to verify compilation**

Run: `cargo build`
Expected: Compiles with no errors.

- [ ] **Step 5: Commit**

```bash
git add src/output.rs
git commit -m "feat: write output grouped by course with tagged delimiters"
```

---

### Task 4: Update main.rs and version

**Files:**
- Modify: `src/main.rs:119-124` (stats output)
- Modify: `Cargo.toml:3` (version)

- [ ] **Step 1: Add course count to stats output**

In `main.rs`, after the `"Statistics:"` line (line 119), add the courses line:

```rust
println!("Statistics:");
println!("- Courses found: {}", result.stats.courses_found);
println!("- Total files found: {}", result.stats.total_files);
println!("- Files excluded by config: {}", result.stats.files_excluded);
println!("- Files processed: {}", result.stats.files_processed);
println!("- Workers used: {}", result.stats.worker_count);
```

- [ ] **Step 2: Rename binary and bump version in Cargo.toml**

Change the `[[bin]]` section and version:

```toml
[package]
name = "fileassetbuilder"
version = "0.5.0"
edition = "2021"

[[bin]]
name = "fileassetsbuildercourse"
path = "src/main.rs"
```

- [ ] **Step 3: Update `#[command(name)]` in main.rs**

Change the clap command name attribute at line 18:

```rust
#[command(name = "fileassetsbuildercourse")]
```

Also update the after_help examples to reference the new binary name:

```rust
#[command(after_help = "EXAMPLES:\n  \
    fileassetsbuildercourse -i \"[C:\\project]\"\n  \
    fileassetsbuildercourse -i \"[C:\\project1 C:\\project2]\"\n  \
    fileassetsbuildercourse -i \"[C:\\project]\" -o snapshot.txt")]
```

- [ ] **Step 4: Build release and test against a real directory**

```bash
cargo build --release
./target/release/fileassetsbuildercourse -i "<test_directory_with_subfolders>"
```

Verify output file contains:
1. Directory tree listing (unchanged)
2. `Courses` section header instead of `Files`
3. `===[ COURSE: <name> ]===...` delimiters wrapping each top-level subdirectory
4. `===[ END COURSE: <name> ]===...` closing each group
5. Individual files within each course separated by `================`

- [ ] **Step 5: Commit**

```bash
git add src/main.rs Cargo.toml
git commit -m "v0.5.0: rename binary to fileassetsbuildercourse, course-grouped output with parseable delimiters"
```
