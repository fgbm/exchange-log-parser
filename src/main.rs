mod config;
mod database;
mod models;
mod parser;

use clap::Parser;
use color_eyre::eyre::Result;
use config::Args;
use database::Database;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info};
use models::LogType;
use parser::LogParser;
use walkdir::WalkDir;

/// Main function
/// 
/// This function is the entry point of the program.
/// It parses the command line arguments, initializes the database connection, and processes the log files.
/// 
/// ### Arguments
/// 
/// - `--logs-dir`: The directory containing the log files.
/// - `--db-host`: The host of the database.
/// - `--db-port`: The port of the database.
/// - `--db-user`: The user of the database.
/// - `--db-password`: The password of the database.
/// - `--db-name`: The name of the database.
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let args = Args::parse();

    // Initialize database connection
    let db = Database::new(
        &args.db_host,
        args.db_port,
        &args.db_user,
        &args.db_password,
        &args.db_name,
    )
    .await?;

    info!(
        "Starting to process log files in {}",
        args.logs_dir.display()
    );

    // Count files for progress bar
    let total_files = WalkDir::new(&args.logs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .count() as u64;

    let pb = ProgressBar::new(total_files);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
            .expect("Failed to set progress bar style")
            .progress_chars("##-"),
    );

    // Process all files in the directory
    for entry in WalkDir::new(&args.logs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() {
            pb.set_message(format!("Processing {}", path.display()));

            match LogParser::detect_log_type(path) {
                Ok(LogType::SmtpReceive) => {
                    match LogParser::parse_smtp_receive_log(path) {
                        Ok(logs) => {
                            if let Err(e) = db.insert_smtp_receive_logs(logs).await {
                                error!("Error inserting SMTP Receive logs for {}: {}", path.display(), e);
                            }
                        }
                        Err(e) => {
                            error!("Error parsing SMTP Receive log {}: {}", path.display(), e);
                        }
                    }
                }
                Ok(LogType::SmtpSend) => {
                    match LogParser::parse_smtp_send_log(path) {
                        Ok(logs) => {
                            if let Err(e) = db.insert_smtp_send_logs(logs).await {
                                error!("Error inserting SMTP Send logs for {}: {}", path.display(), e);
                            }
                        }
                        Err(e) => {
                            error!("Error parsing SMTP Send log {}: {}", path.display(), e);
                        }
                    }
                }
                Ok(LogType::MessageTracking) => {
                    match LogParser::parse_message_tracking_log(path) {
                        Ok(logs) => {
                            if let Err(e) = db.insert_message_tracking_logs(logs).await {
                                error!("Error inserting Message Tracking logs for {}: {}", path.display(), e);
                            }
                        }
                        Err(e) => {
                            error!("Error parsing Message Tracking log {}: {}", path.display(), e);
                        }
                    }
                }
                Ok(LogType::Unknown) => {
                    info!("Skipping file with unknown log type: {}", path.display());
                }
                Err(e) => {
                    error!(
                        "Error detecting log type for file {}: {}",
                        path.display(),
                        e
                    );
                }
            }
            pb.inc(1);
        }
    }

    pb.finish_with_message("Log processing completed");
    Ok(())
}
