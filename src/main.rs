mod config;
mod database;
mod models;
mod parser;

use clap::Parser;
use color_eyre::eyre::Result;
use config::Args;
use database::Database;
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

    // Process all files in the directory
    for entry in WalkDir::new(&args.logs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() {
            info!("Processing file: {}", path.display());

            match LogParser::detect_log_type(path) {
                Ok(LogType::SmtpReceive) => {
                    let logs = LogParser::parse_smtp_receive_log(path)?;
                    db.insert_smtp_receive_logs(logs).await?;
                }
                Ok(LogType::SmtpSend) => {
                    let logs = LogParser::parse_smtp_send_log(path)?;
                    db.insert_smtp_send_logs(logs).await?;
                }
                Ok(LogType::MessageTracking) => {
                    let logs = LogParser::parse_message_tracking_log(path)?;
                    db.insert_message_tracking_logs(logs).await?;
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
        }
    }

    info!("Log processing completed successfully");
    Ok(())
}
