use crate::models::{MessageTrackingLog, SmtpReceiveLog, SmtpSendLog};
use color_eyre::eyre::Result;
use deadpool_postgres::{Config, Pool, Runtime};
use log::{debug, info};
use tokio_postgres::NoTls;

pub struct Database {
    pool: Pool,
}

/// Database implementation
/// 
/// This struct is used to interact with the database.
/// It provides methods to initialize the tables and insert the logs into the database.
/// 
/// ### Examples
/// 
/// ```
/// let db = Database::new("localhost", 5432, "postgres", "password", "exchange_logs").await?;
/// let logs = LogParser::parse_smtp_receive_log("path/to/log/file").await?;
/// db.insert_smtp_receive_logs(logs).await?;
/// ```
impl Database {
    pub async fn new(host: &str, port: u16, user: &str, password: &str, dbname: &str) -> Result<Self> {
        let mut cfg = Config::new();
        cfg.host = Some(host.to_string());
        cfg.port = Some(port);
        cfg.user = Some(user.to_string());
        cfg.password = Some(password.to_string());
        cfg.dbname = Some(dbname.to_string());
        
        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
        
        let db = Database { pool };
        db.init_tables().await?;
        
        Ok(db)
    }
    
    async fn init_tables(&self) -> Result<()> {
        let client = self.pool.get().await?;
        
        // Create SMTP Receive logs table
        client.batch_execute(
            r#"
            CREATE TABLE IF NOT EXISTS smtp_receive_logs (
                id SERIAL PRIMARY KEY,
                date_time TIMESTAMPTZ NOT NULL,
                connector_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                sequence_number INTEGER NOT NULL,
                local_endpoint TEXT NOT NULL,
                remote_endpoint TEXT NOT NULL,
                event TEXT NOT NULL,
                data TEXT,
                context TEXT,
                sender TEXT,
                recipient TEXT,
                message_id TEXT,
                subject TEXT,
                size INTEGER
            );
            CREATE UNIQUE INDEX IF NOT EXISTS smtp_receive_logs_unique_idx 
            ON smtp_receive_logs (date_time, session_id, sequence_number);
            "#,
        ).await?;
        
        // Create SMTP Send logs table
        client.batch_execute(
            r#"
            CREATE TABLE IF NOT EXISTS smtp_send_logs (
                id SERIAL PRIMARY KEY,
                date_time TIMESTAMPTZ NOT NULL,
                connector_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                sequence_number INTEGER NOT NULL,
                local_endpoint TEXT NOT NULL,
                remote_endpoint TEXT NOT NULL,
                event TEXT NOT NULL,
                data TEXT,
                context TEXT,
                proxy_session_id TEXT,
                sender TEXT,
                recipient TEXT,
                message_id TEXT,
                record_id TEXT
            );
            CREATE UNIQUE INDEX IF NOT EXISTS smtp_send_logs_unique_idx 
            ON smtp_send_logs (date_time, session_id, sequence_number);
            "#,
        ).await?;
        
        // Create Message Tracking logs table
        client.batch_execute(
            r#"
            CREATE TABLE IF NOT EXISTS message_tracking_logs (
                id SERIAL PRIMARY KEY,
                date_time TIMESTAMPTZ NOT NULL,
                client_ip TEXT,
                client_hostname TEXT,
                server_ip TEXT,
                server_hostname TEXT NOT NULL,
                source_context TEXT,
                connector_id TEXT,
                source TEXT,
                event_id TEXT NOT NULL,
                internal_message_id TEXT NOT NULL,
                message_id TEXT NOT NULL,
                network_message_id TEXT NOT NULL,
                recipient_address TEXT NOT NULL,
                recipient_status TEXT,
                total_bytes INTEGER,
                recipient_count INTEGER NOT NULL,
                related_recipient_address TEXT,
                reference TEXT,
                message_subject TEXT,
                sender_address TEXT NOT NULL,
                return_path TEXT,
                message_info TEXT,
                directionality TEXT,
                tenant_id TEXT,
                original_client_ip TEXT,
                original_server_ip TEXT,
                custom_data TEXT,
                transport_traffic_type TEXT,
                log_id TEXT,
                schema_version TEXT
            );
            CREATE UNIQUE INDEX IF NOT EXISTS message_tracking_logs_unique_idx 
            ON message_tracking_logs (date_time, internal_message_id, recipient_address, event_id);
            "#,
        ).await?;
        
        info!("Database tables initialized successfully");
        Ok(())
    }
    
    pub async fn insert_smtp_receive_logs(&self, logs: Vec<SmtpReceiveLog>) -> Result<u64> {
        // Проверка на пустой вход
        if logs.is_empty() {
            debug!("Нет SMTP Receive логов для вставки");
            return Ok(0);
        }

        // Получаем соединение один раз для всей операции
        let mut client = self.pool.get().await?;
        let mut inserted_count = 0;
        
        // Начинаем транзакцию
        let tx = client.transaction().await?;
        
        // Подготавливаем statement
        let stmt = tx.prepare(
            "INSERT INTO smtp_receive_logs 
            (date_time, connector_id, session_id, sequence_number, local_endpoint, remote_endpoint, 
            event, data, context, sender, recipient, message_id, subject, size)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (date_time, session_id, sequence_number) DO NOTHING"
        ).await?;
        
        for log in logs {
            let result = tx.execute(
                &stmt,
                &[
                    &log.date_time, &log.connector_id, &log.session_id, &log.sequence_number,
                    &log.local_endpoint, &log.remote_endpoint, &log.event, &log.data, &log.context,
                    &log.sender, &log.recipient, &log.message_id, &log.subject, &log.size
                ],
            ).await?;
            inserted_count += result;
        }
        
        // Фиксируем транзакцию
        tx.commit().await?;
        
        debug!("Inserted {} SMTP Receive logs", inserted_count);
        Ok(inserted_count)
    }
    
    pub async fn insert_smtp_send_logs(&self, logs: Vec<SmtpSendLog>) -> Result<u64> {
        // Проверка на пустой вход
        if logs.is_empty() {
            debug!("Нет SMTP Send логов для вставки");
            return Ok(0);
        }
        
        // Получаем соединение один раз для всей операции
        let mut client = self.pool.get().await?;
        let mut inserted_count = 0;
        
        // Начинаем транзакцию
        let tx = client.transaction().await?;
        
        // Подготавливаем statement
        let stmt = tx.prepare(
            "INSERT INTO smtp_send_logs 
            (date_time, connector_id, session_id, sequence_number, local_endpoint, remote_endpoint, 
            event, data, context, proxy_session_id, sender, recipient, message_id, record_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (date_time, session_id, sequence_number) DO NOTHING"
        ).await?;
        
        for log in logs {
            let result = tx.execute(
                &stmt,
                &[
                    &log.date_time, &log.connector_id, &log.session_id, &log.sequence_number,
                    &log.local_endpoint, &log.remote_endpoint, &log.event, &log.data, &log.context,
                    &log.proxy_session_id, &log.sender, &log.recipient, &log.message_id, &log.record_id
                ],
            ).await?;
            inserted_count += result;
        }
        
        // Фиксируем транзакцию
        tx.commit().await?;
        
        debug!("Inserted {} SMTP Send logs", inserted_count);
        Ok(inserted_count)
    }
    
    pub async fn insert_message_tracking_logs(&self, logs: Vec<MessageTrackingLog>) -> Result<u64> {
        // Проверка на пустой вход
        if logs.is_empty() {
            debug!("Нет Message Tracking логов для вставки");
            return Ok(0);
        }
        
        // Получаем соединение один раз для всей операции
        let mut client = self.pool.get().await?;
        let mut inserted_count = 0;
        
        // Начинаем транзакцию
        let tx = client.transaction().await?;
        
        // Подготавливаем statement
        let stmt = tx.prepare(
            "INSERT INTO message_tracking_logs 
            (date_time, client_ip, client_hostname, server_ip, server_hostname, source_context,
            connector_id, source, event_id, internal_message_id, message_id, network_message_id,
            recipient_address, recipient_status, total_bytes, recipient_count, related_recipient_address,
            reference, message_subject, sender_address, return_path, message_info, directionality,
            tenant_id, original_client_ip, original_server_ip, custom_data, transport_traffic_type,
            log_id, schema_version)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30)
            ON CONFLICT (date_time, internal_message_id, recipient_address, event_id) DO NOTHING"
        ).await?;
        
        for log in logs {
            let result = tx.execute(
                &stmt,
                &[
                    &log.date_time, &log.client_ip, &log.client_hostname, &log.server_ip, &log.server_hostname,
                    &log.source_context, &log.connector_id, &log.source, &log.event_id, &log.internal_message_id,
                    &log.message_id, &log.network_message_id, &log.recipient_address, &log.recipient_status,
                    &log.total_bytes, &log.recipient_count, &log.related_recipient_address, &log.reference,
                    &log.message_subject, &log.sender_address, &log.return_path, &log.message_info,
                    &log.directionality, &log.tenant_id, &log.original_client_ip, &log.original_server_ip,
                    &log.custom_data, &log.transport_traffic_type, &log.log_id, &log.schema_version
                ],
            ).await?;
            inserted_count += result;
        }
        
        // Фиксируем транзакцию
        tx.commit().await?;
        
        debug!("Inserted {} Message Tracking logs", inserted_count);
        Ok(inserted_count)
    }
}
