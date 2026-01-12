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
struct Args {
    /// Input directory to scan
    #[arg(required = true)]
    input_directory: PathBuf,

    /// Output filename (written to input directory root)
    #[arg(short, long, default_value = DEFAULT_OUTPUT_FILENAME)]
    output: String,
}

fn main() {
    let args = Args::parse();

    if !args.input_directory.exists() {
        eprintln!("Error: Input directory does not exist: {:?}", args.input_directory);
        std::process::exit(1);
    }

    if !args.input_directory.is_dir() {
        eprintln!("Error: Input path is not a directory: {:?}", args.input_directory);
        std::process::exit(1);
    }

    let config_path = get_config_path();
    let config = Config::load(&config_path);

    let input_dir = match args.input_directory.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: Could not resolve input directory: {}", e);
            std::process::exit(1);
        }
    };

    let output_path = input_dir.join(&args.output);

    println!("Starting directory scan and file processing...\n");
    println!("Input directory: {:?}", input_dir);
    println!("Output file: {:?}\n", output_path);

    match scan_directory(&input_dir, &config, &args.output) {
        Ok(result) => {
            println!("\nWriting output file...");

            if let Err(e) = write_output(&output_path, &result) {
                eprintln!("Error writing output: {}", e);
                std::process::exit(1);
            }

            println!("\nProcessing complete!");
            println!("Statistics:");
            println!("- Total directories scanned: {}", result.stats.total_directories);
            println!("- Total files found: {}", result.stats.total_files);
            println!("- Files excluded by config: {}", result.stats.files_excluded);
            println!("- Files processed: {}", result.stats.files_processed);
            println!("Output written to: {:?}", output_path);
        }
        Err(e) => {
            eprintln!("Error scanning directory: {}", e);
            std::process::exit(1);
        }
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
