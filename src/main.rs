mod config;
mod database;
mod models;
mod parser;

use clap::Parser;
use color_eyre::eyre::Result;
use colored::Colorize;
use config::Args;
use database::Database;
use futures::stream::{StreamExt, TryStreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info};
use parser::{LogParser, ParsedLog};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use walkdir::WalkDir;

/// Макрос для обработки и вставки логов в базу данных
macro_rules! process_logs {
    ($db:expr, $logs:expr, $path:expr, $success_counter:expr, $error_counter:expr, $log_type:expr, $insert_method:ident) => {
        if !$logs.is_empty() {
            if let Err(e) = $db.$insert_method($logs).await {
                error!(
                    "Error inserting {} logs for {}: {}",
                    $log_type,
                    $path.display(),
                    e
                );
                let mut count = $error_counter.lock().unwrap();
                *count += 1;
            } else {
                let mut count = $success_counter.lock().unwrap();
                *count += 1;
            }
        }
    };
}

/// Макрос для форматирования текста цветом
macro_rules! fmt {
    (success => $text:expr) => {
        $text.green().bold()
    };
    (highlight => $text:expr) => {
        $text.yellow().bold()
    };
    (info => $text:expr) => {
        $text.cyan().bold()
    };
    (label => $text:expr) => {
        $text.blue()
    };
    (error => $text:expr) => {
        $text.red().bold()
    };
    (ok => $text:expr) => {
        $text.green()
    };
    (num => $value:expr) => {
        $value.to_string().yellow()
    };
}

/// Выводит статистику обработки логов в консоль
fn print_statistics(
    total_files: u64,
    duration: std::time::Duration,
    smtp_receive: usize,
    smtp_send: usize,
    message_tracking: usize,
    errors: usize,
) {
    let files_per_second = total_files as f64 / duration.as_secs_f64();
    
    println!(
        "\n\n{} {} {} {} {:.2} {} ({:.2} {})",
        fmt!(success => "✓"),
        fmt!(success => "Обработано"),
        fmt!(highlight => total_files.to_string()),
        fmt!(success => "файлов за"),
        duration.as_secs_f64(),
        fmt!(success => "секунд"),
        files_per_second,
        fmt!(success => "файлов/сек")
    );

    println!(
        "\n{} {}",
        "📊".bold(),
        fmt!(info => "Статистика обработки:")
    );
    println!("  {} {}", fmt!(label => "SMTP Receive:"), fmt!(num => smtp_receive));
    println!("  {} {}", fmt!(label => "SMTP Send:"), fmt!(num => smtp_send));
    println!("  {} {}", fmt!(label => "Message Tracking:"), fmt!(num => message_tracking));
    
    if errors > 0 {
        println!("  {} {}", fmt!(error => "Ошибки:"), fmt!(error => errors.to_string()));
    } else {
        println!("  {} {}", fmt!(ok => "Ошибки:"), fmt!(ok => "0"));
    }
}

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
/// - `--concurrent-files`: The number of files to process concurrently.
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let args = Args::parse();
    let start_time = Instant::now();

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
        "Starting to process log files in {} with {} concurrent tasks",
        args.logs_dir.display(),
        args.concurrent_files
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

    // Счетчики для отслеживания типов обработанных файлов
    let smtp_receive_count = Arc::new(Mutex::new(0));
    let smtp_send_count = Arc::new(Mutex::new(0));
    let message_tracking_count = Arc::new(Mutex::new(0));
    let error_count = Arc::new(Mutex::new(0));

    // Обрабатываем файлы параллельно
    futures::stream::iter(files_to_process)
        .map(|entry| {
            let db_clone = Arc::clone(&db);
            let pb_clone = Arc::clone(&pb);
            let smtp_receive_count_clone = Arc::clone(&smtp_receive_count);
            let smtp_send_count_clone = Arc::clone(&smtp_send_count);
            let message_tracking_count_clone = Arc::clone(&message_tracking_count);
            let error_count_clone = Arc::clone(&error_count);

            async move {
                let path = entry.path();
                pb_clone.set_message(format!("Processing {}", path.display()));

                match LogParser::parse_log_file(path) {
                    Ok(parsed_log) => match parsed_log {
                        ParsedLog::SmtpReceive(logs) => {
                            process_logs!(
                                db_clone,
                                logs,
                                path,
                                smtp_receive_count_clone,
                                error_count_clone,
                                "SMTP Receive",
                                insert_smtp_receive_logs
                            );
                        }
                        ParsedLog::SmtpSend(logs) => {
                            process_logs!(
                                db_clone,
                                logs,
                                path,
                                smtp_send_count_clone,
                                error_count_clone,
                                "SMTP Send",
                                insert_smtp_send_logs
                            );
                        }
                        ParsedLog::MessageTracking(logs) => {
                            process_logs!(
                                db_clone,
                                logs,
                                path,
                                message_tracking_count_clone,
                                error_count_clone,
                                "Message Tracking",
                                insert_message_tracking_logs
                            );
                        }
                    },
                    Err(e) => {
                        error!("Error processing file {}: {}", path.display(), e);
                        let mut count = error_count_clone.lock().unwrap();
                        *count += 1;
                    }
                }
                pb_clone.inc(1);
                Ok::<(), color_eyre::eyre::Error>(())
            }
        })
        .buffer_unordered(args.concurrent_files) // Используем значение из аргументов
        .try_collect::<()>() // Собираем результаты (ждем завершения)
        .await?; // Обрабатываем возможную ошибку из потока

    pb.finish_with_message("Log processing completed");

    let duration = start_time.elapsed();
    
    // Получаем значения счетчиков
    let smtp_receive = *smtp_receive_count.lock().unwrap();
    let smtp_send = *smtp_send_count.lock().unwrap();
    let message_tracking = *message_tracking_count.lock().unwrap();
    let errors = *error_count.lock().unwrap();
    
    // Выводим статистику
    print_statistics(
        total_files,
        duration,
        smtp_receive,
        smtp_send,
        message_tracking,
        errors
    );

    Ok(())
}
