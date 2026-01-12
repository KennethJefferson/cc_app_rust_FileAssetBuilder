use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use chrono::Local;

use crate::scanner::ScanResult;

const SEPARATOR: &str = "================================================================";
const FILE_SEPARATOR: &str = "================";

pub fn write_output(output_path: &Path, result: &ScanResult) -> Result<(), String> {
    let file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut writer = BufWriter::new(file);

    write_header(&mut writer)?;
    write_file_list(&mut writer, &result.file_list)?;
    write_files(&mut writer, result)?;

    writer
        .flush()
        .map_err(|e| format!("Failed to flush output: {}", e))?;

    Ok(())
}

fn write_header(writer: &mut BufWriter<File>) -> Result<(), String> {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

    writeln!(
        writer,
        "This file is a merged representation of the directory, combining all text-based files into a single document."
    )
    .map_err(|e| format!("Write error: {}", e))?;

    writeln!(writer, "Generated on: {}\n", timestamp)
        .map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}

fn write_file_list(writer: &mut BufWriter<File>, file_list: &str) -> Result<(), String> {
    writeln!(writer, "{}", SEPARATOR).map_err(|e| format!("Write error: {}", e))?;
    writeln!(writer, "Directory List").map_err(|e| format!("Write error: {}", e))?;
    writeln!(writer, "{}\n", SEPARATOR).map_err(|e| format!("Write error: {}", e))?;
    writeln!(writer, "{}", file_list).map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}

fn write_files(writer: &mut BufWriter<File>, result: &ScanResult) -> Result<(), String> {
    writeln!(writer, "{}", SEPARATOR).map_err(|e| format!("Write error: {}", e))?;
    writeln!(writer, "Files").map_err(|e| format!("Write error: {}", e))?;
    writeln!(writer, "{}\n", SEPARATOR).map_err(|e| format!("Write error: {}", e))?;

    for file in &result.files {
        writeln!(writer, "{}", FILE_SEPARATOR).map_err(|e| format!("Write error: {}", e))?;
        writeln!(writer, "File: {}", file.relative_path)
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(writer, "{}", FILE_SEPARATOR).map_err(|e| format!("Write error: {}", e))?;
        writeln!(writer, "{}", file.content).map_err(|e| format!("Write error: {}", e))?;
        writeln!(writer).map_err(|e| format!("Write error: {}", e))?;
    }

    Ok(())
}
