use crate::models::{MessageTrackingLog, SmtpReceiveLog, SmtpSendLog};
use async_trait::async_trait;
use color_eyre::eyre::Result;

pub mod mssql;
pub mod postgres;

#[async_trait]
pub trait Database: Send + Sync {
    /// Инициализирует таблицы в базе данных
    async fn init_tables(&self) -> Result<()>;

    /// Вставляет логи SMTP Receive
    async fn insert_smtp_receive_logs(&self, logs: Vec<SmtpReceiveLog>) -> Result<u64>;

    /// Вставляет логи SMTP Send
    async fn insert_smtp_send_logs(&self, logs: Vec<SmtpSendLog>) -> Result<u64>;

    /// Вставляет логи Message Tracking
    async fn insert_message_tracking_logs(&self, logs: Vec<MessageTrackingLog>) -> Result<u64>;
}

#[derive(Debug, Clone)]
pub enum DatabaseType {
    Postgres,
    MsSql,
}

impl std::str::FromStr for DatabaseType {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "postgres" => Ok(DatabaseType::Postgres),
            "mssql" => Ok(DatabaseType::MsSql),
            _ => Err(color_eyre::eyre::eyre!(
                "Неподдерживаемый тип базы данных: {}",
                s
            )),
        }
    }
}

pub async fn create_database(
    db_type: DatabaseType,
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    dbname: &str,
    table_prefix: Option<&str>,
) -> Result<Box<dyn Database>> {
    match db_type {
        DatabaseType::Postgres => {
            let db =
                postgres::PostgresDatabase::new(host, port, user, password, dbname, table_prefix)
                    .await?;
            Ok(Box::new(db))
        }
        DatabaseType::MsSql => {
            let db =
                mssql::MsSqlDatabase::new(host, port, user, password, dbname, table_prefix).await?;
            Ok(Box::new(db))
        }
    }
}
