use crate::models::{LogType, MessageTrackingLog, SmtpReceiveLog, SmtpSendLog};
use chrono::{DateTime, Utc};
use color_eyre::eyre::{Result, eyre};
use encoding_rs::WINDOWS_1251;
use lazy_static::lazy_static;
use log::info;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

lazy_static! {
    static ref SIZE_REGEX: Regex = Regex::new(r"SIZE=(\d+)").unwrap();
    static ref MAIL_FROM_REGEX: Regex = Regex::new(r"MAIL FROM:<([^>]+)>").unwrap();
    static ref RCPT_TO_REGEX: Regex = Regex::new(r"RCPT TO:<([^>]+)>").unwrap();
    static ref MESSAGE_ID_REGEX: Regex = Regex::new(r"<([^>]+)>").unwrap();
    static ref PROXY_SESSION_REGEX: Regex = Regex::new(r"session id (\w+)").unwrap();
    static ref RECORD_ID_REGEX: Regex = Regex::new(r"RecordId (\d+)").unwrap();
    static ref INTERNET_MESSAGE_ID_REGEX: Regex =
        Regex::new(r"InternetMessageId <([^>]+)>").unwrap();
}

pub struct LogParser;

#[derive(Debug)]
pub enum ParsedLog {
    SmtpReceive(Vec<SmtpReceiveLog>),
    SmtpSend(Vec<SmtpSendLog>),
    MessageTracking(Vec<MessageTrackingLog>),
}

impl LogParser {
    /// Reads and decodes a file with proper Windows-1251 handling
    async fn read_and_decode_file(file_path: &Path) -> Result<String> {
        let mut file = File::open(file_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        // Try Windows-1251 first, fallback to UTF-8
        let (cow, _, had_errors) = WINDOWS_1251.decode(&buffer);
        if had_errors {
            Ok(String::from_utf8_lossy(&buffer).into_owned())
        } else {
            Ok(cow.into_owned())
        }
    }

    pub async fn detect_log_type(file_path: &Path) -> Result<LogType> {
        let content = Self::read_and_decode_file(file_path).await?;

        for line in content.lines() {
            if line.starts_with("#Log-type:") {
                return match line.trim() {
                    "#Log-type: SMTP Receive Protocol Log" => Ok(LogType::SmtpReceive),
                    "#Log-type: SMTP Send Protocol Log" => Ok(LogType::SmtpSend),
                    "#Log-type: Message Tracking Log" => Ok(LogType::MessageTracking),
                    _ => Ok(LogType::Unknown),
                };
            }
        }

        Ok(LogType::Unknown)
    }

    pub async fn parse_log_file(file_path: &Path) -> Result<ParsedLog> {
        let log_type = Self::detect_log_type(file_path).await?;
        match log_type {
            LogType::SmtpReceive => {
                let logs = Self::parse_smtp_receive_log(file_path).await?;
                Ok(ParsedLog::SmtpReceive(logs))
            }
            LogType::SmtpSend => {
                let logs = Self::parse_smtp_send_log(file_path).await?;
                Ok(ParsedLog::SmtpSend(logs))
            }
            LogType::MessageTracking => {
                let logs = Self::parse_message_tracking_log(file_path).await?;
                Ok(ParsedLog::MessageTracking(logs))
            }
            LogType::Unknown => Err(eyre!("Unknown log type in file: {}", file_path.display())),
        }
    }

    fn parse_common_fields(
        line: &str,
        indices: &HashMap<String, usize>,
    ) -> Result<(
        DateTime<Utc>,
        String,
        String,
        i32,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
    )> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < indices.len() {
            return Err(eyre!("Line has fewer parts than expected fields"));
        }

        let date_time = DateTime::parse_from_rfc3339(parts[indices["date-time"]])
            .map_err(|e| eyre!("Failed to parse date: {}", e))?
            .with_timezone(&Utc);

        let connector_id = parts[indices["connector-id"]].to_string();
        let session_id = parts[indices["session-id"]].to_string();
        let sequence_number = parts[indices["sequence-number"]].parse::<i32>()?;
        let local_endpoint = parts[indices["local-endpoint"]].to_string();
        let remote_endpoint = parts[indices["remote-endpoint"]].to_string();
        let event = parts[indices["event"]].to_string();
        let data = if parts[indices["data"]].is_empty() {
            None
        } else {
            Some(parts[indices["data"]].to_string())
        };
        let context = if parts.get(indices["context"]).map_or(true, |s| s.is_empty()) {
            None
        } else {
            Some(parts[indices["context"]].to_string())
        };

        Ok((
            date_time,
            connector_id,
            session_id,
            sequence_number,
            local_endpoint,
            remote_endpoint,
            event,
            data,
            context,
        ))
    }

    pub async fn parse_smtp_receive_log(file_path: &Path) -> Result<Vec<SmtpReceiveLog>> {
        let content = Self::read_and_decode_file(file_path).await?;
        let mut fields_indices: Option<HashMap<String, usize>> = None;
        let mut session_data: HashMap<String, SmtpReceiveLog> = HashMap::new();

        for line in content.lines() {
            if line.starts_with("#Fields:") {
                let fields: Vec<&str> = line
                    .trim_start_matches("#Fields:")
                    .split(',')
                    .map(|s| s.trim())
                    .collect();
                let indices = fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| (field.to_string(), i))
                    .collect();
                fields_indices = Some(indices);
                continue;
            }

            if line.starts_with("#") || line.trim().is_empty() {
                continue;
            }

            if let Some(indices) = &fields_indices {
                let (
                    date_time,
                    connector_id,
                    session_id,
                    sequence_number,
                    local_endpoint,
                    remote_endpoint,
                    event,
                    data,
                    context,
                ) = Self::parse_common_fields(line, indices)?;

                // Create or get existing session log
                let log =
                    session_data
                        .entry(session_id.clone())
                        .or_insert_with(|| SmtpReceiveLog {
                            id: None,
                            date_time,
                            connector_id: connector_id.clone(),
                            session_id: session_id.clone(),
                            sequence_number,
                            local_endpoint: local_endpoint.clone(),
                            remote_endpoint: remote_endpoint.clone(),
                            event: event.clone(),
                            data: data.clone(),
                            context: context.clone(),
                            sender: None,
                            recipient: None,
                            message_id: None,
                            subject: None,
                            size: None,
                        });

                // Extract additional information from data field
                if let Some(data_str) = &data {
                    if let Some(captures) = MAIL_FROM_REGEX.captures(data_str) {
                        log.sender = captures.get(1).map(|m| m.as_str().to_string());
                    }

                    if let Some(captures) = RCPT_TO_REGEX.captures(data_str) {
                        log.recipient = captures.get(1).map(|m| m.as_str().to_string());
                    }

                    if let Some(captures) = MESSAGE_ID_REGEX.captures(data_str) {
                        log.message_id = captures.get(1).map(|m| m.as_str().to_string());
                    }

                    if let Some(captures) = SIZE_REGEX.captures(data_str) {
                        log.size = captures.get(1).and_then(|m| m.as_str().parse::<i32>().ok());
                    }
                }
            }
        }

        let logs: Vec<SmtpReceiveLog> = session_data.into_values().collect();
        info!(
            "Parsed {} SMTP Receive log entries from {}",
            logs.len(),
            file_path.display()
        );
        Ok(logs)
    }

    pub async fn parse_smtp_send_log(file_path: &Path) -> Result<Vec<SmtpSendLog>> {
        let content = Self::read_and_decode_file(file_path).await?;
        let mut fields_indices: Option<HashMap<String, usize>> = None;
        let mut session_data: HashMap<String, SmtpSendLog> = HashMap::new();

        for line in content.lines() {
            if line.starts_with("#Fields:") {
                let fields: Vec<&str> = line
                    .trim_start_matches("#Fields:")
                    .split(',')
                    .map(|s| s.trim())
                    .collect();
                let indices = fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| (field.to_string(), i))
                    .collect();
                fields_indices = Some(indices);
                continue;
            }

            if line.starts_with("#") || line.trim().is_empty() {
                continue;
            }

            if let Some(indices) = &fields_indices {
                let (
                    date_time,
                    connector_id,
                    session_id,
                    sequence_number,
                    local_endpoint,
                    remote_endpoint,
                    event,
                    data,
                    context,
                ) = Self::parse_common_fields(line, indices)?;

                let log = session_data
                    .entry(session_id.clone())
                    .or_insert_with(|| SmtpSendLog {
                        id: None,
                        date_time,
                        connector_id: connector_id.clone(),
                        session_id: session_id.clone(),
                        sequence_number,
                        local_endpoint: local_endpoint.clone(),
                        remote_endpoint: remote_endpoint.clone(),
                        event: event.clone(),
                        data: data.clone(),
                        context: context.clone(),
                        proxy_session_id: None,
                        sender: None,
                        recipient: None,
                        message_id: None,
                        record_id: None,
                    });

                if let Some(context_str) = &context {
                    if context_str.contains("Proxying inbound session") {
                        if let Some(captures) = PROXY_SESSION_REGEX.captures(context_str) {
                            log.proxy_session_id = captures.get(1).map(|m| m.as_str().to_string());
                        }
                    }

                    if context_str.contains("sending message with RecordId") {
                        if let Some(captures) = RECORD_ID_REGEX.captures(context_str) {
                            log.record_id = captures.get(1).map(|m| m.as_str().to_string());
                        }

                        if let Some(captures) = INTERNET_MESSAGE_ID_REGEX.captures(context_str) {
                            log.message_id = captures.get(1).map(|m| m.as_str().to_string());
                        }
                    }
                }

                if let Some(data_str) = &data {
                    if let Some(captures) = MAIL_FROM_REGEX.captures(data_str) {
                        log.sender = captures.get(1).map(|m| m.as_str().to_string());
                    }

                    if let Some(captures) = RCPT_TO_REGEX.captures(data_str) {
                        log.recipient = captures.get(1).map(|m| m.as_str().to_string());
                    }
                }
            }
        }

        let logs: Vec<SmtpSendLog> = session_data.into_values().collect();
        info!(
            "Parsed {} SMTP Send log entries from {}",
            logs.len(),
            file_path.display()
        );
        Ok(logs)
    }

    pub async fn parse_message_tracking_log(file_path: &Path) -> Result<Vec<MessageTrackingLog>> {
        let content = Self::read_and_decode_file(file_path).await?;
        let mut logs = Vec::new();
        let mut fields_indices: Option<HashMap<String, usize>> = None;

        for line in content.lines() {
            if line.starts_with("#Fields:") {
                let fields: Vec<&str> = line
                    .trim_start_matches("#Fields:")
                    .split(',')
                    .map(|s| s.trim())
                    .collect();
                let indices = fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| (field.to_string(), i))
                    .collect();
                fields_indices = Some(indices);
                continue;
            }

            if line.starts_with("#") || line.trim().is_empty() {
                continue;
            }

            if let Some(indices) = &fields_indices {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() < indices.len() {
                    continue;
                }

                let date_time = DateTime::parse_from_rfc3339(parts[indices["date-time"]])
                    .map_err(|e| eyre!("Failed to parse date: {}", e))?
                    .with_timezone(&Utc);

                let get_field = |field: &str| -> Option<String> {
                    indices
                        .get(field)
                        .and_then(|&idx| parts.get(idx))
                        .filter(|&&s| !s.is_empty())
                        .map(|s| s.to_string())
                };

                let get_required_field = |field: &str| -> String {
                    indices
                        .get(field)
                        .and_then(|&idx| parts.get(idx))
                        .map(|s| s.to_string())
                        .unwrap_or_default()
                };

                logs.push(MessageTrackingLog {
                    id: None,
                    date_time,
                    client_ip: get_field("client-ip"),
                    client_hostname: get_field("client-hostname"),
                    server_ip: get_field("server-ip"),
                    server_hostname: get_required_field("server-hostname"),
                    source_context: get_field("source-context"),
                    connector_id: get_field("connector-id"),
                    source: get_field("source"),
                    event_id: get_required_field("event-id"),
                    internal_message_id: get_required_field("internal-message-id"),
                    message_id: get_required_field("message-id"),
                    network_message_id: get_required_field("network-message-id"),
                    recipient_address: get_required_field("recipient-address"),
                    recipient_status: get_field("recipient-status"),
                    total_bytes: get_field("total-bytes").and_then(|s| s.parse::<i32>().ok()),
                    recipient_count: get_field("recipient-count")
                        .and_then(|s| s.parse::<i32>().ok())
                        .unwrap_or(0),
                    related_recipient_address: get_field("related-recipient-address"),
                    reference: get_field("reference"),
                    message_subject: get_field("message-subject"),
                    sender_address: get_required_field("sender-address"),
                    return_path: get_field("return-path"),
                    message_info: get_field("message-info"),
                    directionality: get_field("directionality"),
                    tenant_id: get_field("tenant-id"),
                    original_client_ip: get_field("original-client-ip"),
                    original_server_ip: get_field("original-server-ip"),
                    custom_data: get_field("custom-data"),
                    transport_traffic_type: get_field("transport-traffic-type"),
                    log_id: get_field("log-id"),
                    schema_version: get_field("schema-version"),
                });
            }
        }

        info!(
            "Parsed {} Message Tracking log entries from {}",
            logs.len(),
            file_path.display()
        );
        Ok(logs)
    }
}
