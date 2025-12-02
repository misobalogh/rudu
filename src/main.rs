use clap::Parser;
use filesize::PathExt;
use owo_colors::OwoColorize;
use std::cmp::Reverse;
use std::fs;
use std::path::PathBuf;
use tabled::{Table, Tabled, settings::{Alignment, Style, object::Columns}};
use walkdir::WalkDir;

// TODO: Refactor into modules

#[derive(Parser)]
#[command(name = "rudu")]
#[command(about = "A disk usage analyzer")]
struct Cli {
    #[arg(default_value = ".")]
    path: PathBuf,
}

const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
const BAR_LENGTH: usize = 20;

#[derive(Tabled)]
struct FileInfo {
    #[tabled(rename = "File Name")]
    name: String,
    #[tabled(skip)]
    raw_size: u64,
    #[tabled(rename = "")]
    percentage: String,
      #[tabled(rename = "Size")]
    size: String,
}

impl FileInfo {
    fn new(name: String, raw_size: u64, total: u64) -> Self {
        let percentage = if total > 0 {
            raw_size as f64 / total as f64 * 100.0
        } else {
            0.0
        };

        Self {
            name: name.green().to_string(),
            size: colorize_by_size(raw_size),
            raw_size,
            percentage: format_percentage_bar(percentage),
        }
    }
}

fn human_readable(size: f64) -> (String, usize) {
    let (final_size, unit_index) = UNITS
        .iter()
        .enumerate()
        .take(UNITS.len() - 1)
        .try_fold((size, 0), |(size, _), (idx, _)| {
            if size >= 1024.0 {
                Ok((size / 1024.0, idx + 1))
            } else {
                Err((size, idx))
            }
        })
        .unwrap_or_else(|result| result);

    (format!("{final_size:.2} {}", UNITS[unit_index]), unit_index)
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

fn format_percentage_bar(percentage: f64) -> String {
    let filled_length = ((percentage / 100.0) * BAR_LENGTH as f64).round() as usize;
    let empty_length = BAR_LENGTH.saturating_sub(filled_length);

    let bar = format!(
        "[{}{}] {:5.1}%",
        "#".repeat(filled_length),
        ".".repeat(empty_length),
        percentage
    );

    match percentage {
        p if p < 20.0 => bar.white().to_string(),
        p if p < 40.0 => bar.green().to_string(),
        p if p < 60.0 => bar.blue().to_string(),
        p if p < 80.0 => bar.yellow().to_string(),
        _ => bar.red().to_string(),
    }
}

fn calculate_dir_size(path: &PathBuf) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| entry.path().size_on_disk().ok())
        .sum()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let path = &cli.path;

    if !path.exists() {
        return Err(format!("Path {path:?} does not exist").into());
    }

    // First pass: collect entries with their sizes
    let entries: Vec<_> = fs::read_dir(path)?
        .filter_map(Result::ok)
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            let size = calculate_dir_size(&entry.path());
            (name, size)
        })
        .collect();

    let total: u64 = entries.iter().map(|(_, size)| size).sum();

    // Second pass: create FileInfo with percentages and sort
    let mut rows: Vec<_> = entries
        .into_iter()
        .map(|(name, size)| FileInfo::new(name, size, total))
        .collect();

    rows.sort_by_key(|row| Reverse(row.raw_size));

    let table = Table::new(rows).with(Style::rounded()).modify(Columns::last(), Alignment::right()).to_string();

    println!("{table}");
    println!("Total size: {}", colorize_by_size(total));

    Ok(())
}
