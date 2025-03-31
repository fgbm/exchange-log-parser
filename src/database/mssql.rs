use crate::models::{MessageTrackingLog, SmtpReceiveLog, SmtpSendLog};
use async_trait::async_trait;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use color_eyre::eyre::Result;
use log::{debug, info};
use tiberius::{AuthMethod, Config, Query};

use super::Database;

pub struct MsSqlDatabase {
    pool: Pool<ConnectionManager>,
    table_prefix: String,
}

impl MsSqlDatabase {
    pub async fn new(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        dbname: &str,
        table_prefix: Option<&str>,
    ) -> Result<Self> {
        let mut config = Config::new();
        config.host(host);
        config.port(port);
        config.database(dbname);
        config.authentication(AuthMethod::sql_server(user, password));
        config.trust_cert(); // В продакшене нужно настроить правильную проверку сертификата

        let manager = ConnectionManager::build(config)?;
        let pool = Pool::builder().build(manager).await?;

        let db = MsSqlDatabase {
            pool,
            table_prefix: table_prefix.unwrap_or("").to_string(),
        };
        db.init_tables().await?;

        Ok(db)
    }
}

#[async_trait]
impl Database for MsSqlDatabase {
    async fn init_tables(&self) -> Result<()> {
        let mut client = self.pool.get().await?;

        // Create SMTP Receive logs table
        let sql_smtp_receive = format!(
            r#"
            IF NOT EXISTS (SELECT * FROM sys.objects WHERE object_id = OBJECT_ID(N'[dbo].[{prefix}smtp_receive_logs]') AND type in (N'U'))
            BEGIN
                CREATE TABLE [dbo].[{prefix}smtp_receive_logs] (
                    [id] [int] IDENTITY(1,1) PRIMARY KEY,
                    [date_time] [datetimeoffset](7) NOT NULL,
                    [connector_id] [nvarchar](max) NOT NULL,
                    [session_id] [nvarchar](450) NOT NULL,
                    [sequence_number] [int] NOT NULL,
                    [local_endpoint] [nvarchar](max) NOT NULL,
                    [remote_endpoint] [nvarchar](max) NOT NULL,
                    [event] [nvarchar](max) NOT NULL,
                    [data] [nvarchar](max) NULL,
                    [context] [nvarchar](max) NULL,
                    [sender] [nvarchar](max) NULL,
                    [recipient] [nvarchar](max) NULL,
                    [message_id] [nvarchar](max) NULL,
                    [subject] [nvarchar](max) NULL,
                    [size] [int] NULL
                )

                CREATE UNIQUE NONCLUSTERED INDEX [IX_{prefix}smtp_receive_logs_unique] ON [dbo].[{prefix}smtp_receive_logs]
                (
                    [date_time] ASC,
                    [session_id] ASC,
                    [sequence_number] ASC
                )
            END
            "#,
            prefix = self.table_prefix
        );
        let query = Query::new(sql_smtp_receive.as_str());
        query.execute(&mut client).await?;

        // Create SMTP Send logs table
        let sql_smtp_send = format!(
            r#"
            IF NOT EXISTS (SELECT * FROM sys.objects WHERE object_id = OBJECT_ID(N'[dbo].[{prefix}smtp_send_logs]') AND type in (N'U'))
            BEGIN
                CREATE TABLE [dbo].[{prefix}smtp_send_logs] (
                    [id] [int] IDENTITY(1,1) PRIMARY KEY,
                    [date_time] [datetimeoffset](7) NOT NULL,
                    [connector_id] [nvarchar](max) NOT NULL,
                    [session_id] [nvarchar](450) NOT NULL,
                    [sequence_number] [int] NOT NULL,
                    [local_endpoint] [nvarchar](max) NOT NULL,
                    [remote_endpoint] [nvarchar](max) NOT NULL,
                    [event] [nvarchar](max) NOT NULL,
                    [data] [nvarchar](max) NULL,
                    [context] [nvarchar](max) NULL,
                    [proxy_session_id] [nvarchar](max) NULL,
                    [sender] [nvarchar](max) NULL,
                    [recipient] [nvarchar](max) NULL,
                    [message_id] [nvarchar](max) NULL,
                    [record_id] [nvarchar](max) NULL
                )

                CREATE UNIQUE NONCLUSTERED INDEX [IX_{prefix}smtp_send_logs_unique] ON [dbo].[{prefix}smtp_send_logs]
                (
                    [date_time] ASC,
                    [session_id] ASC,
                    [sequence_number] ASC
                )
            END
            "#,
            prefix = self.table_prefix
        );
        let query = Query::new(sql_smtp_send.as_str());
        query.execute(&mut client).await?;

        // Create Message Tracking logs table
        let sql_msg_tracking = format!(
            r#"
            IF NOT EXISTS (SELECT * FROM sys.objects WHERE object_id = OBJECT_ID(N'[dbo].[{prefix}message_tracking_logs]') AND type in (N'U'))
            BEGIN
                CREATE TABLE [dbo].[{prefix}message_tracking_logs] (
                    [id] [int] IDENTITY(1,1) PRIMARY KEY,
                    [date_time] [datetimeoffset](7) NOT NULL,
                    [client_ip] [nvarchar](max) NULL,
                    [client_hostname] [nvarchar](max) NULL,
                    [server_ip] [nvarchar](max) NULL,
                    [server_hostname] [nvarchar](max) NOT NULL,
                    [source_context] [nvarchar](max) NULL,
                    [connector_id] [nvarchar](max) NULL,
                    [source] [nvarchar](max) NULL,
                    [event_id] [nvarchar](450) NOT NULL,
                    [internal_message_id] [nvarchar](450) NOT NULL,
                    [message_id] [nvarchar](max) NOT NULL,
                    [network_message_id] [nvarchar](max) NOT NULL,
                    [recipient_address] [nvarchar](450) NOT NULL,
                    [recipient_status] [nvarchar](max) NULL,
                    [total_bytes] [int] NULL,
                    [recipient_count] [int] NOT NULL,
                    [related_recipient_address] [nvarchar](max) NULL,
                    [reference] [nvarchar](max) NULL,
                    [message_subject] [nvarchar](max) NULL,
                    [sender_address] [nvarchar](max) NOT NULL,
                    [return_path] [nvarchar](max) NULL,
                    [message_info] [nvarchar](max) NULL,
                    [directionality] [nvarchar](max) NULL,
                    [tenant_id] [nvarchar](max) NULL,
                    [original_client_ip] [nvarchar](max) NULL,
                    [original_server_ip] [nvarchar](max) NULL,
                    [custom_data] [nvarchar](max) NULL,
                    [transport_traffic_type] [nvarchar](max) NULL,
                    [log_id] [nvarchar](max) NULL,
                    [schema_version] [nvarchar](max) NULL
                )

                CREATE UNIQUE NONCLUSTERED INDEX [IX_{prefix}message_tracking_logs_unique] ON [dbo].[{prefix}message_tracking_logs]
                (
                    [date_time] ASC,
                    [internal_message_id] ASC,
                    [recipient_address] ASC,
                    [event_id] ASC
                )
            END
            "#,
            prefix = self.table_prefix
        );
        let query = Query::new(sql_msg_tracking.as_str());
        query.execute(&mut client).await?;

        info!("Database tables initialized successfully");
        Ok(())
    }

    async fn insert_smtp_receive_logs(&self, logs: Vec<SmtpReceiveLog>) -> Result<u64> {
        if logs.is_empty() {
            debug!("Нет SMTP Receive логов для вставки");
            return Ok(0);
        }

        let mut client = self.pool.get().await?;
        let mut inserted_count = 0;

        client.simple_query("BEGIN TRANSACTION").await?;

        for log in logs {
            let sql = format!(
                r#"
                INSERT INTO [dbo].[{prefix}smtp_receive_logs]
                (date_time, connector_id, session_id, sequence_number, local_endpoint, remote_endpoint,
                event, data, context, sender, recipient, message_id, subject, size)
                VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10, @P11, @P12, @P13, @P14)
                "#,
                prefix = self.table_prefix
            );
            let mut query = Query::new(sql.as_str());

            query.bind(log.date_time);
            query.bind(&log.connector_id);
            query.bind(&log.session_id);
            query.bind(log.sequence_number);
            query.bind(&log.local_endpoint);
            query.bind(&log.remote_endpoint);
            query.bind(&log.event);
            query.bind(log.data.as_deref());
            query.bind(log.context.as_deref());
            query.bind(log.sender.as_deref());
            query.bind(log.recipient.as_deref());
            query.bind(log.message_id.as_deref());
            query.bind(log.subject.as_deref());
            query.bind(log.size);

            let result = query.execute(&mut client).await?;
            if let Some(rows) = result.rows_affected().first() {
                inserted_count += *rows as u64;
            }
        }

        client.simple_query("COMMIT TRANSACTION").await?;

        debug!("Inserted {} SMTP Receive logs", inserted_count);
        Ok(inserted_count)
    }

    async fn insert_smtp_send_logs(&self, logs: Vec<SmtpSendLog>) -> Result<u64> {
        if logs.is_empty() {
            debug!("Нет SMTP Send логов для вставки");
            return Ok(0);
        }

        let mut client = self.pool.get().await?;
        let mut inserted_count = 0;

        client.simple_query("BEGIN TRANSACTION").await?;

        for log in logs {
            let sql = format!(
                r#"
                INSERT INTO [dbo].[{prefix}smtp_send_logs]
                (date_time, connector_id, session_id, sequence_number, local_endpoint, remote_endpoint,
                event, data, context, proxy_session_id, sender, recipient, message_id, record_id)
                VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10, @P11, @P12, @P13, @P14)
                "#,
                prefix = self.table_prefix
            );
            let mut query = Query::new(sql.as_str());

            query.bind(log.date_time);
            query.bind(&log.connector_id);
            query.bind(&log.session_id);
            query.bind(log.sequence_number);
            query.bind(&log.local_endpoint);
            query.bind(&log.remote_endpoint);
            query.bind(&log.event);
            query.bind(log.data.as_deref());
            query.bind(log.context.as_deref());
            query.bind(log.proxy_session_id.as_deref());
            query.bind(log.sender.as_deref());
            query.bind(log.recipient.as_deref());
            query.bind(log.message_id.as_deref());
            query.bind(log.record_id.as_deref());

            let result = query.execute(&mut client).await?;
            if let Some(rows) = result.rows_affected().first() {
                inserted_count += *rows as u64;
            }
        }

        client.simple_query("COMMIT TRANSACTION").await?;

        debug!("Inserted {} SMTP Send logs", inserted_count);
        Ok(inserted_count)
    }

    async fn insert_message_tracking_logs(&self, logs: Vec<MessageTrackingLog>) -> Result<u64> {
        if logs.is_empty() {
            debug!("Нет Message Tracking логов для вставки");
            return Ok(0);
        }

        let mut client = self.pool.get().await?;
        let mut inserted_count = 0;

        client.simple_query("BEGIN TRANSACTION").await?;

        for log in logs {
            let sql = format!(
                r#"
                INSERT INTO [dbo].[{prefix}message_tracking_logs]
                (date_time, client_ip, client_hostname, server_ip, server_hostname, source_context,
                connector_id, source, event_id, internal_message_id, message_id, network_message_id,
                recipient_address, recipient_status, total_bytes, recipient_count, related_recipient_address,
                reference, message_subject, sender_address, return_path, message_info, directionality,
                tenant_id, original_client_ip, original_server_ip, custom_data, transport_traffic_type,
                log_id, schema_version)
                VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10, @P11, @P12, @P13, @P14,
                        @P15, @P16, @P17, @P18, @P19, @P20, @P21, @P22, @P23, @P24, @P25, @P26,
                        @P27, @P28, @P29, @P30)
                "#,
                prefix = self.table_prefix
            );
            let mut query = Query::new(sql.as_str());

            query.bind(log.date_time);
            query.bind(log.client_ip.as_deref());
            query.bind(log.client_hostname.as_deref());
            query.bind(log.server_ip.as_deref());
            query.bind(&log.server_hostname);
            query.bind(log.source_context.as_deref());
            query.bind(log.connector_id.as_deref());
            query.bind(log.source.as_deref());
            query.bind(&log.event_id);
            query.bind(&log.internal_message_id);
            query.bind(&log.message_id);
            query.bind(&log.network_message_id);
            query.bind(&log.recipient_address);
            query.bind(log.recipient_status.as_deref());
            query.bind(log.total_bytes);
            query.bind(log.recipient_count);
            query.bind(log.related_recipient_address.as_deref());
            query.bind(log.reference.as_deref());
            query.bind(log.message_subject.as_deref());
            query.bind(&log.sender_address);
            query.bind(log.return_path.as_deref());
            query.bind(log.message_info.as_deref());
            query.bind(log.directionality.as_deref());
            query.bind(log.tenant_id.as_deref());
            query.bind(log.original_client_ip.as_deref());
            query.bind(log.original_server_ip.as_deref());
            query.bind(log.custom_data.as_deref());
            query.bind(log.transport_traffic_type.as_deref());
            query.bind(log.log_id.as_deref());
            query.bind(log.schema_version.as_deref());

            let result = query.execute(&mut client).await?;
            if let Some(rows) = result.rows_affected().first() {
                inserted_count += *rows as u64;
            }
        }

        client.simple_query("COMMIT TRANSACTION").await?;

        debug!("Inserted {} Message Tracking logs", inserted_count);
        Ok(inserted_count)
    }
}
