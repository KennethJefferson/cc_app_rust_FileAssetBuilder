mod config;
mod output;
mod scanner;

use std::env;
use std::path::PathBuf;

use clap::Parser;

use config::Config;
use output::write_output;
use scanner::scan_directory;

const DEFAULT_OUTPUT_FILENAME: &str = "fileassets.txt";
const CONFIG_FILENAME: &str = "config.txt";

#[derive(Parser)]
#[command(name = "fileassetbuilder")]
#[command(about = "Consolidate directory files into a single output file")]
#[command(version)]
#[command(after_help = "EXAMPLES:\n  \
    fileassetbuilder -i \"[C:\\project]\"\n  \
    fileassetbuilder -i \"[C:\\project1 C:\\project2]\"\n  \
    fileassetbuilder -i \"[C:\\project]\" -o snapshot.txt")]
struct Args {
    /// Input directories in bracket syntax: "[dir1 dir2 dir3]"
    #[arg(short, long, required = true, value_parser = parse_bracket_input, num_args = 1..)]
    input: Vec<Vec<PathBuf>>,

    /// Output filename (written to each input directory root)
    #[arg(short, long, default_value = DEFAULT_OUTPUT_FILENAME)]
    output: String,

    /// Verbose output
    #[arg(short, long, default_value = "false")]
    verbose: bool,
}

/// Parse bracket-enclosed space-separated input: [dir1 dir2 dir3]
fn parse_bracket_input(s: &str) -> Result<Vec<PathBuf>, String> {
    let trimmed = s.trim();

    // Check for bracket syntax
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        let paths: Vec<PathBuf> = inner
            .split_whitespace()
            .map(PathBuf::from)
            .collect();

        if paths.is_empty() {
            return Err("Empty input list".to_string());
        }
        Ok(paths)
    } else {
        // Single path without brackets
        Ok(vec![PathBuf::from(trimmed)])
    }
}

fn main() {
    let args = Args::parse();

    // Flatten input paths from bracket syntax
    let input_dirs: Vec<PathBuf> = args.input.into_iter().flatten().collect();

    let config_path = get_config_path();
    let config = Config::load(&config_path);

    let mut has_errors = false;

    for input_directory in &input_dirs {
        println!("{}", "=".repeat(60));

        if !input_directory.exists() {
            eprintln!("Error: Input directory does not exist: {:?}", input_directory);
            has_errors = true;
            continue;
        }

        if !input_directory.is_dir() {
            eprintln!("Error: Input path is not a directory: {:?}", input_directory);
            has_errors = true;
            continue;
        }

        let input_dir = match input_directory.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error: Could not resolve input directory: {}", e);
                has_errors = true;
                continue;
            }
        };

        let output_path = input_dir.join(&args.output);

        if output_path.exists() {
            println!("Output file already exists: {:?}", output_path);
            println!("Skipping scan and file creation.");
            continue;
        }

        println!("Starting directory scan and file processing...\n");
        println!("Input directory: {:?}", input_dir);
        println!("Output file: {:?}\n", output_path);

        match scan_directory(&input_dir, &config, &args.output, args.verbose) {
            Ok(result) => {
                println!("\nWriting output file...");

                if let Err(e) = write_output(&output_path, &result) {
                    eprintln!("Error writing output: {}", e);
                    has_errors = true;
                    continue;
                }

                println!("\nProcessing complete!");
                println!("Statistics:");
                println!("- Total files found: {}", result.stats.total_files);
                println!("- Files excluded by config: {}", result.stats.files_excluded);
                println!("- Files processed: {}", result.stats.files_processed);
                println!("- Workers used: {}", result.stats.worker_count);
                println!("Output written to: {:?}", output_path);
            }
            Err(e) => {
                eprintln!("Error scanning directory: {}", e);
                has_errors = true;
            }
        }
    }

    if has_errors {
        std::process::exit(1);
    }
}

fn get_config_path() -> PathBuf {
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join(CONFIG_FILENAME);
        }
    }
    PathBuf::from(CONFIG_FILENAME)
}
