use crate::models::{LogType, MessageTrackingLog, PgDateTime, SmtpReceiveLog, SmtpSendLog};
use chrono::{DateTime, Utc};
use color_eyre::eyre::{Result, eyre};
use encoding_rs::WINDOWS_1251;
use lazy_static::lazy_static;
use log::info;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

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

/// Log parser implementation
///
/// This struct is used to parse the log files.
///
/// ### Examples
///
/// ```
/// let parser = LogParser;
/// let logs = parser.parse_smtp_receive_log("path/to/log/file")?;
/// ```
pub struct LogParser;

impl LogParser {
    pub fn detect_log_type(file_path: &Path) -> Result<LogType> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        let (cow, _, had_errors) = WINDOWS_1251.decode(&buffer);
        let content = if had_errors {
            String::from_utf8_lossy(&buffer).into_owned()
        } else {
            cow.into_owned()
        };

        for line in content.lines() {
            if line.starts_with("#Log-type:") {
                match line.trim() {
                    "#Log-type: SMTP Receive Protocol Log" => return Ok(LogType::SmtpReceive),
                    "#Log-type: SMTP Send Protocol Log" => return Ok(LogType::SmtpSend),
                    "#Log-type: Message Tracking Log" => return Ok(LogType::MessageTracking),
                    _ => return Ok(LogType::Unknown),
                }
            }
        }

        Ok(LogType::Unknown)
    }

    // pub fn parse_log_file(file_path: &Path) -> Result<Vec<Box<dyn std::any::Any>>> {
    //     let log_type = Self::detect_log_type(file_path)?;
    //     match log_type {
    //         LogType::SmtpReceive => {
    //             let logs = Self::parse_smtp_receive_log(file_path)?;
    //             Ok(logs
    //                 .into_iter()
    //                 .map(|log| Box::new(log) as Box<dyn std::any::Any>)
    //                 .collect())
    //         }
    //         LogType::SmtpSend => {
    //             let logs = Self::parse_smtp_send_log(file_path)?;
    //             Ok(logs
    //                 .into_iter()
    //                 .map(|log| Box::new(log) as Box<dyn std::any::Any>)
    //                 .collect())
    //         }
    //         LogType::MessageTracking => {
    //             let logs = Self::parse_message_tracking_log(file_path)?;
    //             Ok(logs
    //                 .into_iter()
    //                 .map(|log| Box::new(log) as Box<dyn std::any::Any>)
    //                 .collect())
    //         }
    //         LogType::Unknown => Err(eyre!("Unknown log type in file: {}", file_path.display())),
    //     }
    // }

    pub fn parse_smtp_receive_log(file_path: &Path) -> Result<Vec<SmtpReceiveLog>> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        let (cow, _, had_errors) = WINDOWS_1251.decode(&buffer);
        let content = if had_errors {
            String::from_utf8_lossy(&buffer).into_owned()
        } else {
            cow.into_owned()
        };

        let mut fields_indices: Option<HashMap<String, usize>> = None;
        let mut session_data: HashMap<String, SmtpReceiveLog> = HashMap::new();

        for line in content.lines() {
            if line.starts_with("#Fields:") {
                let fields: Vec<&str> = line
                    .trim_start_matches("#Fields:")
                    .split(',')
                    .map(|s| s.trim())
                    .collect();
                let mut indices = HashMap::new();
                for (i, field) in fields.iter().enumerate() {
                    indices.insert(field.to_string(), i);
                }
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

                let date_time_str = parts[indices["date-time"]];
                let date_time = DateTime::parse_from_rfc3339(date_time_str)
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

                // Create or get existing session log
                let log =
                    session_data
                        .entry(session_id.clone())
                        .or_insert_with(|| SmtpReceiveLog {
                            id: None,
                            date_time: PgDateTime(date_time),
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
                    // Extract sender
                    if let Some(captures) = MAIL_FROM_REGEX.captures(data_str) {
                        if let Some(sender) = captures.get(1) {
                            log.sender = Some(sender.as_str().to_string());
                        }
                    }

                    // Extract recipient
                    if let Some(captures) = RCPT_TO_REGEX.captures(data_str) {
                        if let Some(recipient) = captures.get(1) {
                            log.recipient = Some(recipient.as_str().to_string());
                        }
                    }

                    // Extract message ID
                    if let Some(captures) = MESSAGE_ID_REGEX.captures(data_str) {
                        if let Some(message_id) = captures.get(1) {
                            log.message_id = Some(message_id.as_str().to_string());
                        }
                    }

                    // Extract size
                    if let Some(captures) = SIZE_REGEX.captures(data_str) {
                        if let Some(size_str) = captures.get(1) {
                            if let Ok(size) = size_str.as_str().parse::<i32>() {
                                log.size = Some(size);
                            }
                        }
                    }
                }
            }
        }

        // Convert the session_data hashmap to a vector
        let logs: Vec<SmtpReceiveLog> = session_data.values().cloned().collect();

        info!(
            "Parsed {} SMTP Receive log entries from {}",
            logs.len(),
            file_path.display()
        );
        Ok(logs)
    }

    pub fn parse_smtp_send_log(file_path: &Path) -> Result<Vec<SmtpSendLog>> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        let (cow, _, had_errors) = WINDOWS_1251.decode(&buffer);
        let content = if had_errors {
            String::from_utf8_lossy(&buffer).into_owned()
        } else {
            cow.into_owned()
        };

        let mut fields_indices: Option<HashMap<String, usize>> = None;
        let mut session_data: HashMap<String, SmtpSendLog> = HashMap::new();

        for line in content.lines() {
            if line.starts_with("#Fields:") {
                let fields: Vec<&str> = line
                    .trim_start_matches("#Fields:")
                    .split(',')
                    .map(|s| s.trim())
                    .collect();
                let mut indices = HashMap::new();
                for (i, field) in fields.iter().enumerate() {
                    indices.insert(field.to_string(), i);
                }
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

                let date_time_str = parts[indices["date-time"]];
                let date_time = DateTime::parse_from_rfc3339(date_time_str)
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

                // Create or get existing session log
                let log = session_data
                    .entry(session_id.clone())
                    .or_insert_with(|| SmtpSendLog {
                        id: None,
                        date_time: PgDateTime(date_time),
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

                // Extract additional information
                if let Some(context_str) = &context {
                    if context_str.contains("Proxying inbound session") {
                        if let Some(captures) = PROXY_SESSION_REGEX.captures(context_str) {
                            if let Some(proxy_session_id) = captures.get(1) {
                                log.proxy_session_id = Some(proxy_session_id.as_str().to_string());
                            }
                        }
                    }

                    if context_str.contains("sending message with RecordId") {
                        if let Some(captures) = RECORD_ID_REGEX.captures(context_str) {
                            if let Some(record_id) = captures.get(1) {
                                log.record_id = Some(record_id.as_str().to_string());
                            }
                        }

                        if let Some(captures) = INTERNET_MESSAGE_ID_REGEX.captures(context_str) {
                            if let Some(message_id) = captures.get(1) {
                                log.message_id = Some(message_id.as_str().to_string());
                            }
                        }
                    }
                }

                // Extract sender and recipient from data field
                if let Some(data_str) = &data {
                    if let Some(captures) = MAIL_FROM_REGEX.captures(data_str) {
                        if let Some(sender) = captures.get(1) {
                            log.sender = Some(sender.as_str().to_string());
                        }
                    }

                    if let Some(captures) = RCPT_TO_REGEX.captures(data_str) {
                        if let Some(recipient) = captures.get(1) {
                            log.recipient = Some(recipient.as_str().to_string());
                        }
                    }
                }
            }
        }

        // Convert the session_data hashmap to a vector
        let logs: Vec<SmtpSendLog> = session_data.values().cloned().collect();

        info!(
            "Parsed {} SMTP Send log entries from {}",
            logs.len(),
            file_path.display()
        );
        Ok(logs)
    }

    pub fn parse_message_tracking_log(file_path: &Path) -> Result<Vec<MessageTrackingLog>> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        let (cow, _, had_errors) = WINDOWS_1251.decode(&buffer);
        let content = if had_errors {
            String::from_utf8_lossy(&buffer).into_owned()
        } else {
            cow.into_owned()
        };

        let mut logs = Vec::new();
        let mut fields_indices: Option<HashMap<String, usize>> = None;

        for line in content.lines() {
            if line.starts_with("#Fields:") {
                let fields: Vec<&str> = line
                    .trim_start_matches("#Fields:")
                    .split(',')
                    .map(|s| s.trim())
                    .collect();
                let mut indices = HashMap::new();
                for (i, field) in fields.iter().enumerate() {
                    indices.insert(field.to_string(), i);
                }
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

                let date_time_str = parts[indices["date-time"]];
                let date_time = DateTime::parse_from_rfc3339(date_time_str)
                    .map_err(|e| eyre!("Failed to parse date: {}", e))?
                    .with_timezone(&Utc);

                let get_field = |field: &str| -> Option<String> {
                    if let Some(&idx) = indices.get(field) {
                        if idx < parts.len() && !parts[idx].is_empty() {
                            return Some(parts[idx].to_string());
                        }
                    }
                    None
                };

                let get_required_field = |field: &str| -> String {
                    if let Some(&idx) = indices.get(field) {
                        if idx < parts.len() {
                            return parts[idx].to_string();
                        }
                    }
                    String::new()
                };

                let log = MessageTrackingLog {
                    id: None,
                    date_time: PgDateTime(date_time),
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
                };

                logs.push(log);
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
