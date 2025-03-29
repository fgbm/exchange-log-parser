mod config;
mod database;
mod models;
mod parser;

use clap::Parser;
use color_eyre::eyre::Result;
use config::Args;
use database::Database;
use futures::stream::{StreamExt, TryStreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info};
use parser::{LogParser, ParsedLog};
use std::sync::Arc;
use walkdir::WalkDir;

const MAX_CONCURRENT_FILES: usize = 10; // Ограничение на количество одновременно обрабатываемых файлов

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
    let db = Arc::new(
        Database::new(
            &args.db_host,
            args.db_port,
            &args.db_user,
            &args.db_password,
            &args.db_name,
        )
        .await?,
    );

    info!(
        "Starting to process log files in {}",
        args.logs_dir.display()
    );

    // Собираем список файлов для обработки
    let files_to_process: Vec<_> = WalkDir::new(&args.logs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    let total_files = files_to_process.len() as u64;
    let pb = Arc::new(ProgressBar::new(total_files)); // Используем Arc для ProgressBar
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
            .expect("Failed to set progress bar style")
            .progress_chars("##-"),
    );

    // Обрабатываем файлы параллельно
    futures::stream::iter(files_to_process)
        .map(|entry| {
            let db_clone = Arc::clone(&db);
            let pb_clone = Arc::clone(&pb);
            async move {
                let path = entry.path();
                pb_clone.set_message(format!("Processing {}", path.display()));

                match LogParser::parse_log_file(path) {
                    Ok(parsed_log) => match parsed_log {
                        ParsedLog::SmtpReceive(logs) => {
                            if !logs.is_empty() {
                                if let Err(e) = db_clone.insert_smtp_receive_logs(logs).await {
                                    error!(
                                        "Error inserting SMTP Receive logs for {}: {}",
                                        path.display(),
                                        e
                                    );
                                }
                            }
                        }
                        ParsedLog::SmtpSend(logs) => {
                            if !logs.is_empty() {
                                if let Err(e) = db_clone.insert_smtp_send_logs(logs).await {
                                    error!(
                                        "Error inserting SMTP Send logs for {}: {}",
                                        path.display(),
                                        e
                                    );
                                }
                            }
                        }
                        ParsedLog::MessageTracking(logs) => {
                            if !logs.is_empty() {
                                if let Err(e) = db_clone.insert_message_tracking_logs(logs).await {
                                    error!(
                                        "Error inserting Message Tracking logs for {}: {}",
                                        path.display(),
                                        e
                                    );
                                }
                            }
                        }
                    },
                    Err(e) => {
                        error!("Error processing file {}: {}", path.display(), e);
                    }
                }
                pb_clone.inc(1);
                Ok::<(), color_eyre::eyre::Error>(())
            }
        })
        .buffer_unordered(MAX_CONCURRENT_FILES) // Запускаем задачи параллельно
        .try_collect::<()>() // Собираем результаты (ждем завершения)
        .await?; // Обрабатываем возможную ошибку из потока

    pb.finish_with_message("Log processing completed");
    Ok(())
}
