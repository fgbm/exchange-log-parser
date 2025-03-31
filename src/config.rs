use crate::database::DatabaseType;
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
    /// Path to the directory containing log files
    #[arg(default_value = ".")]
    pub logs_dir: PathBuf,

    /// Database type (postgres or mssql)
    #[arg(long, default_value = "postgres")]
    pub db_type: DatabaseType,

    /// Database host
    #[arg(long, default_value = "localhost")]
    pub db_host: String,

    /// Database port
    #[arg(long, default_value_t = 5432)]
    pub db_port: u16,

    /// Database username
    #[arg(long, default_value = "postgres")]
    pub db_user: String,

    /// Database password
    #[arg(long)]
    pub db_password: String,

    /// Database name
    #[arg(long, default_value = "exchange_logs")]
    pub db_name: String,

    /// Number of files to process concurrently
    #[arg(short, long, default_value_t = 10)]
    pub concurrent_files: usize,

    /// Table prefix
    #[arg(long)]
    pub table_prefix: Option<String>,
}
