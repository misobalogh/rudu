use clap::Parser;
use filesize::PathExt;
use owo_colors::OwoColorize;
use std::fs;
use std::path::PathBuf;
use tabled::{Table, Tabled, settings::Style};
use walkdir::WalkDir;

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

fn human_readable(mut size: f64) -> (String, usize) {
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index + 1 < UNITS.len() {
        size /= 1024.0;
        unit_index += 1;
    }

    (format!("{:.2} {}", size, UNITS[unit_index]), unit_index)
}

fn colorize_by_size(size: u64) -> String {
    let (human_size, unit_index) = human_readable(size as f64);
    match unit_index {
        0 => human_size.white().to_string(),
        1 => human_size.green().to_string(),
        2 => human_size.blue().to_string(),
        3 => human_size.yellow().to_string(),
        _ => human_size.red().to_string(),
    }
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
            size: colorize_by_size(size_subdir),
        });
    }

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", table);

    println!("Total size: {}", colorize_by_size(total));

    Ok(())
}
