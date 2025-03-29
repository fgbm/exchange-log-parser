use clap::Parser;
use std::path::PathBuf;

/// Command line arguments
/// 
/// This struct is used to parse the command line arguments.
/// 
/// ### Examples
///
/// ```
/// let args = Args::parse();
/// ```
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Directory containing log files
    #[arg(default_value = ".")]
    pub logs_dir: PathBuf,

    /// PostgreSQL host
    #[arg(long, default_value = "localhost")]
    pub db_host: String,

    /// PostgreSQL port
    #[arg(long, default_value_t = 5432)]
    pub db_port: u16,

    /// PostgreSQL username
    #[arg(long, default_value = "postgres")]
    pub db_user: String,

    /// PostgreSQL password
    #[arg(long)]
    pub db_password: String,

    /// PostgreSQL database name
    #[arg(long, default_value = "exchange_logs")]
    pub db_name: String,

    /// Количество параллельно обрабатываемых файлов
    #[arg(short, long, default_value_t = 10)]
    pub concurrent_files: usize,
}
