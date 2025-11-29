use clap::Parser;
use filesize::PathExt;
use owo_colors::OwoColorize;
use std::fs;
use std::path::PathBuf;
use tabled::{Table, Tabled, settings::Style};
use walkdir::WalkDir;

// TODO: GB color as red, MB as yellow, KB as green, B as white
// TODO: Add "percentage bar" [.........####] 40.0%
// TODO: Sort by default
// TODO: Refactor into modules


#[derive(Parser)]
#[command(name = "rudu")]
#[command(about = "A disk usage analyzer")]
struct Cli {
    #[arg(default_value = ".")]
    path: PathBuf,
}

const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

#[derive(Tabled)]
struct FileInfo {
    #[tabled(rename = "File Name")]
    name: String,
    #[tabled(rename = "Size")]
    size: String,
}

fn human_readable(mut size: f64) -> String {
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index + 1 < UNITS.len() {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let path = &cli.path;

    if !path.exists() {
        return Err(format!("Path {:?} does not exist", path).into());
    }


    let mut total: u64 = 0;
    let mut rows: Vec<FileInfo> = Vec::new();

    let entries = fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_path = path.join(&file_name);
        let size_subdir: u64 = WalkDir::new(file_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.path().size_on_disk().ok())
            .sum();

        total += size_subdir;
        rows.push(FileInfo {
            name: file_name.green().to_string(),
            size: human_readable(size_subdir as f64).yellow().to_string(),
        });
    }

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", table);

    println!("Total size: {}", human_readable(total as f64));

    Ok(())
}
