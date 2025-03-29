use chrono::{DateTime, Utc};
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use postgres_types::Type;
use bytes::BytesMut;
use color_eyre::eyre::eyre;

/// PostgreSQL date time type
/// 
/// This struct is used to represent a date time in PostgreSQL.
/// 
/// ### Examples
/// 
/// ```
/// let date_time = PgDateTime(DateTime::from_utc(NaiveDateTime::from_timestamp(1717027200, 0), Utc));
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PgDateTime(pub DateTime<Utc>);

impl ToSql for PgDateTime {
    fn to_sql(&self, _ty: &Type, out: &mut BytesMut) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let timestamp = self.0.timestamp();
        out.extend_from_slice(&timestamp.to_be_bytes());
        Ok(postgres_types::IsNull::No)
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }

    fn to_sql_checked(&self, _ty: &Type, out: &mut BytesMut) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        self.to_sql(_ty, out)
    }
}

impl FromSql<'_> for PgDateTime {
    fn from_sql(_ty: &Type, raw: &[u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let timestamp = i64::from_be_bytes(raw.try_into()?);
        let date_time = DateTime::from_timestamp(timestamp, 0)
            .ok_or_else(|| eyre!("Invalid timestamp"))?;
        Ok(PgDateTime(date_time))
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}

/// SMTP Receive log
/// 
/// This struct is used to represent a SMTP Receive log.
/// 
/// ### Examples
///
/// ```
/// let log = SmtpReceiveLog {
///     id: None,
///     date_time: PgDateTime(DateTime::from_utc(NaiveDateTime::from_timestamp(1717027200, 0), Utc)),
///     connector_id: "123".to_string(),
///     session_id: "456".to_string(),
///     sequence_number: 1,
///     local_endpoint: "127.0.0.1:1234".to_string(),
///     remote_endpoint: "127.0.0.1:1235".to_string(),
///     event: "SMTP Receive".to_string(),
///     data: None,
///     context: None,
///     sender: None,
///     recipient: None,
///     message_id: None,
///     subject: None,
///     size: None,
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, ToSql, FromSql, Clone)]
pub struct SmtpReceiveLog {
    pub id: Option<i32>,
    pub date_time: PgDateTime,
    pub connector_id: String,
    pub session_id: String,
    pub sequence_number: i32,
    pub local_endpoint: String,
    pub remote_endpoint: String,
    pub event: String,
    pub data: Option<String>,
    pub context: Option<String>,
    pub sender: Option<String>,
    pub recipient: Option<String>,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub size: Option<i32>,
}

/// SMTP Send log
/// 
/// This struct is used to represent a SMTP Send log.
/// 
/// ### Examples
///
/// ```
/// let log = SmtpSendLog {
///     id: None,
///     date_time: PgDateTime(DateTime::from_utc(NaiveDateTime::from_timestamp(1717027200, 0), Utc)),
///     connector_id: "123".to_string(),
///     session_id: "456".to_string(),
///     sequence_number: 1,
///     local_endpoint: "127.0.0.1:1234".to_string(),
///     remote_endpoint: "127.0.0.1:1235".to_string(),
///     event: "SMTP Send".to_string(),
///     data: None,
///     context: None,
///     proxy_session_id: None,
///     sender: None,
///     recipient: None,
///     message_id: None,
///     record_id: None,
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, ToSql, FromSql, Clone)]
pub struct SmtpSendLog {
    pub id: Option<i32>,
    pub date_time: PgDateTime,
    pub connector_id: String,
    pub session_id: String,
    pub sequence_number: i32,
    pub local_endpoint: String,
    pub remote_endpoint: String,
    pub event: String,
    pub data: Option<String>,
    pub context: Option<String>,
    pub proxy_session_id: Option<String>,
    pub sender: Option<String>,
    pub recipient: Option<String>,
    pub message_id: Option<String>,
    pub record_id: Option<String>,
}

/// Message Tracking log
/// 
/// This struct is used to represent a Message Tracking log.
/// 
/// ### Examples
///
/// ```
/// let log = MessageTrackingLog {
///     id: None,
///     date_time: PgDateTime(DateTime::from_utc(NaiveDateTime::from_timestamp(1717027200, 0), Utc)),
///     client_ip: None,
///     client_hostname: None,
///     server_ip: None,
///     server_hostname: "127.0.0.1".to_string(),
///     source_context: None,
///     connector_id: None,
///     source: None,
///     event_id: "123".to_string(),
///     internal_message_id: "456".to_string(),
///     message_id: "789".to_string(),
///     network_message_id: "101".to_string(),
///     recipient_address: "test@example.com".to_string(),
///     recipient_status: None,
///     total_bytes: None,
///     recipient_count: 1,
///     related_recipient_address: None,
///     reference: None,
///     message_subject: None,
///     sender_address: "test@example.com".to_string(),
///     return_path: None,
///     message_info: None,
///     directionality: None,
///     tenant_id: None,
///     original_client_ip: None,
///     original_server_ip: None,
///     custom_data: None,
///     transport_traffic_type: None,
///     log_id: None,
///     schema_version: None,
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, ToSql, FromSql, Clone)]
pub struct MessageTrackingLog {
    pub id: Option<i32>,
    pub date_time: PgDateTime,
    pub client_ip: Option<String>,
    pub client_hostname: Option<String>,
    pub server_ip: Option<String>,
    pub server_hostname: String,
    pub source_context: Option<String>,
    pub connector_id: Option<String>,
    pub source: Option<String>,
    pub event_id: String,
    pub internal_message_id: String,
    pub message_id: String,
    pub network_message_id: String,
    pub recipient_address: String,
    pub recipient_status: Option<String>,
    pub total_bytes: Option<i32>,
    pub recipient_count: i32,
    pub related_recipient_address: Option<String>,
    pub reference: Option<String>,
    pub message_subject: Option<String>,
    pub sender_address: String,
    pub return_path: Option<String>,
    pub message_info: Option<String>,
    pub directionality: Option<String>,
    pub tenant_id: Option<String>,
    pub original_client_ip: Option<String>,
    pub original_server_ip: Option<String>,
    pub custom_data: Option<String>,
    pub transport_traffic_type: Option<String>,
    pub log_id: Option<String>,
    pub schema_version: Option<String>,
}

/// Log type
/// 
/// This enum is used to represent the type of log.
/// 
/// ### Examples
///
/// ```
/// let log_type = LogType::SmtpReceive;
/// ```
#[derive(Debug)]
pub enum LogType {
    SmtpReceive,
    SmtpSend,
    MessageTracking,
    Unknown,
}
