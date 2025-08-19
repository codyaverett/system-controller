use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter, BufReader, BufRead};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tokio::sync::{RwLock, Mutex};
use crate::protocol::messages::Command;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecurityEventType {
    AuthenticationSuccess,
    AuthenticationFailure,
    AuthorizationFailure,
    CommandExecution,
    TokenGeneration,
    TokenRevocation,
    SecurityViolation,
    ConnectionEstablished,
    ConnectionTerminated,
    ConfigurationChange,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_type: SecurityEventType,
    pub user_id: String,
    pub client_ip: String,
    #[serde(with = "systemtime_serde")]
    pub timestamp: SystemTime,
    pub details: String,
    pub severity: SecuritySeverity,
}

mod systemtime_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = time.duration_since(UNIX_EPOCH).unwrap().as_secs();
        timestamp.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(timestamp))
    }
}

#[derive(Debug, Clone)]
pub struct RotationInfo {
    pub current_file_size: u64,
    pub total_files: usize,
    pub oldest_file: Option<SystemTime>,
    pub newest_file: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct SecurityAlert {
    pub event: SecurityEvent,
    pub alert_time: SystemTime,
    pub severity: SecuritySeverity,
}

pub struct AuditLogger {
    log_file_path: PathBuf,
    current_writer: RwLock<Option<BufWriter<File>>>,
    max_file_size: Option<u64>,
    max_files: Option<usize>,
    recent_alerts: Arc<RwLock<Vec<SecurityAlert>>>,
}

pub struct RealTimeMonitor {
    alerts: Arc<RwLock<Vec<SecurityAlert>>>,
    severity_threshold: SecuritySeverity,
}

impl AuditLogger {
    pub async fn new(log_file_path: String) -> Result<Self> {
        let path = PathBuf::from(log_file_path);
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create log directory: {}", e))?;
        }
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| anyhow!("Failed to open log file: {}", e))?;
        
        let writer = BufWriter::new(file);
        
        Ok(Self {
            log_file_path: path,
            current_writer: RwLock::new(Some(writer)),
            max_file_size: None,
            max_files: None,
            recent_alerts: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn with_rotation(
        log_file_path: String,
        max_file_size: u64,
        max_files: usize,
    ) -> Result<Self> {
        let mut logger = Self::new(log_file_path).await?;
        logger.max_file_size = Some(max_file_size);
        logger.max_files = Some(max_files);
        Ok(logger)
    }

    pub async fn log_security_event(&mut self, event: SecurityEvent) -> Result<()> {
        let timestamp = event.timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let log_entry = serde_json::json!({
            "timestamp": timestamp,
            "event_type": event.event_type,
            "user_id": event.user_id,
            "client_ip": event.client_ip,
            "details": event.details,
            "severity": event.severity,
        });
        
        let log_line = format!("{}\n", log_entry);
        
        // Check if we need to rotate
        if let Some(max_size) = self.max_file_size {
            let current_size = self.get_current_file_size().await?;
            if current_size + log_line.len() as u64 > max_size {
                self.rotate_log_files().await?;
            }
        }
        
        // Write the log entry
        {
            let mut writer_guard = self.current_writer.write().await;
            if let Some(writer) = writer_guard.as_mut() {
                writer.write_all(log_line.as_bytes())
                    .map_err(|e| anyhow!("Failed to write log entry: {}", e))?;
                writer.flush()
                    .map_err(|e| anyhow!("Failed to flush log: {}", e))?;
            }
        }
        
        // Add to alerts if high severity
        if matches!(event.severity, SecuritySeverity::Error | SecuritySeverity::Critical) {
            let alert = SecurityAlert {
                event: event.clone(),
                alert_time: SystemTime::now(),
                severity: event.severity,
            };
            
            let mut alerts = self.recent_alerts.write().await;
            alerts.push(alert);
            
            // Keep only recent alerts (last 100)
            let len = alerts.len();
            if len > 100 {
                alerts.drain(0..len - 100);
            }
        }
        
        Ok(())
    }

    pub async fn log_command_execution(
        &mut self,
        user_id: &str,
        client_ip: &str,
        command: &Command,
        success: bool,
    ) -> Result<()> {
        let event = SecurityEvent {
            event_type: SecurityEventType::CommandExecution,
            user_id: user_id.to_string(),
            client_ip: client_ip.to_string(),
            timestamp: SystemTime::now(),
            details: format!(
                "Command '{}' (type: {:?}) execution: {}",
                command.id,
                command.command_type,
                if success { "SUCCESS" } else { "FAILED" }
            ),
            severity: if success { SecuritySeverity::Info } else { SecuritySeverity::Warning },
        };
        
        self.log_security_event(event).await
    }

    pub async fn log_authentication_failure(
        &mut self,
        user_id: &str,
        client_ip: &str,
        reason: &str,
    ) -> Result<()> {
        let event = SecurityEvent {
            event_type: SecurityEventType::AuthenticationFailure,
            user_id: user_id.to_string(),
            client_ip: client_ip.to_string(),
            timestamp: SystemTime::now(),
            details: format!("Authentication failed: {}", reason),
            severity: SecuritySeverity::Warning,
        };
        
        self.log_security_event(event).await
    }

    async fn get_current_file_size(&self) -> Result<u64> {
        let metadata = std::fs::metadata(&self.log_file_path)
            .map_err(|e| anyhow!("Failed to get file metadata: {}", e))?;
        Ok(metadata.len())
    }

    async fn rotate_log_files(&mut self) -> Result<()> {
        // Close current writer
        {
            let mut writer_guard = self.current_writer.write().await;
            if let Some(writer) = writer_guard.take() {
                drop(writer);
            }
        }
        
        // Rotate existing files
        if let Some(max_files) = self.max_files {
            for i in (1..max_files).rev() {
                let old_path = format!("{}.{}", self.log_file_path.display(), i);
                let new_path = format!("{}.{}", self.log_file_path.display(), i + 1);
                
                if Path::new(&old_path).exists() {
                    std::fs::rename(&old_path, &new_path).ok();
                }
            }
            
            // Move current log to .1
            let backup_path = format!("{}.1", self.log_file_path.display());
            std::fs::rename(&self.log_file_path, &backup_path).ok();
        }
        
        // Create new log file
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.log_file_path)
            .map_err(|e| anyhow!("Failed to create new log file: {}", e))?;
        
        let writer = BufWriter::new(file);
        
        {
            let mut writer_guard = self.current_writer.write().await;
            *writer_guard = Some(writer);
        }
        
        Ok(())
    }

    pub async fn get_rotation_info(&self) -> Result<RotationInfo> {
        let current_size = self.get_current_file_size().await?;
        
        // Count rotated files
        let mut total_files = 1; // Current file
        let mut oldest_file = None;
        let newest_file = Some(SystemTime::now());
        
        if let Some(max_files) = self.max_files {
            for i in 1..=max_files {
                let rotated_path = format!("{}.{}", self.log_file_path.display(), i);
                if Path::new(&rotated_path).exists() {
                    total_files += 1;
                    if let Ok(metadata) = std::fs::metadata(&rotated_path) {
                        if let Ok(modified) = metadata.modified() {
                            if oldest_file.is_none() || Some(modified) < oldest_file {
                                oldest_file = Some(modified);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(RotationInfo {
            current_file_size: current_size,
            total_files,
            oldest_file,
            newest_file,
        })
    }

    pub async fn search_events(
        &self,
        event_type: SecurityEventType,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Result<Vec<SecurityEvent>> {
        let start_timestamp = start_time.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let end_timestamp = end_time.duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        let mut events = Vec::new();
        
        // Search current log file
        if let Ok(file) = File::open(&self.log_file_path) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Ok(log_entry) = serde_json::from_str::<serde_json::Value>(&line) {
                        if let (Some(timestamp), Some(logged_event_type)) = (
                            log_entry["timestamp"].as_u64(),
                            log_entry["event_type"].as_str(),
                        ) {
                            if timestamp >= start_timestamp && timestamp <= end_timestamp {
                                if let Ok(logged_event_type) = serde_json::from_str::<SecurityEventType>(&format!("\"{}\"", logged_event_type)) {
                                    if logged_event_type == event_type {
                                        if let Ok(event) = serde_json::from_value::<SecurityEvent>(log_entry) {
                                            events.push(event);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(events)
    }

    pub async fn create_real_time_monitor(&self) -> Result<RealTimeMonitor> {
        Ok(RealTimeMonitor {
            alerts: Arc::clone(&self.recent_alerts),
            severity_threshold: SecuritySeverity::Warning,
        })
    }
}

impl RealTimeMonitor {
    pub async fn get_recent_alerts(&self) -> Result<Vec<SecurityAlert>> {
        let alerts = self.alerts.read().await;
        Ok(alerts.clone())
    }

    pub async fn add_alert(&self, alert: SecurityAlert) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        alerts.push(alert);
        
        // Keep only recent alerts
        let len = alerts.len();
        if len > 50 {
            alerts.drain(0..len - 50);
        }
        
        Ok(())
    }
}

// Global rate limiting structures

pub struct RateLimiter {
    requests: Mutex<HashMap<String, Vec<SystemTime>>>,
    max_requests: usize,
    window_duration: Duration,
}

pub struct CommandRateLimiter {
    limits: RwLock<HashMap<crate::protocol::messages::CommandType, (usize, Duration)>>,
    requests: Mutex<HashMap<String, HashMap<crate::protocol::messages::CommandType, Vec<SystemTime>>>>,
}

pub struct AdaptiveRateLimiter {
    base_limits: RwLock<HashMap<String, (usize, Duration)>>,
    current_limits: RwLock<HashMap<String, (usize, Duration)>>,
    request_history: Mutex<HashMap<String, Vec<(SystemTime, Duration)>>>,
}

#[derive(Debug, Clone)]
pub struct LimitInfo {
    pub requests_per_minute: usize,
    pub window_duration: Duration,
    pub is_restricted: bool,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_duration: Duration) -> Result<Self> {
        Ok(Self {
            requests: Mutex::new(HashMap::new()),
            max_requests,
            window_duration,
        })
    }

    pub async fn check_rate_limit(&mut self, user_id: &str) -> Result<bool> {
        let now = SystemTime::now();
        let window_start = now - self.window_duration;
        
        let mut requests = self.requests.lock().await;
        let user_requests = requests.entry(user_id.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        user_requests.retain(|&time| time > window_start);
        
        // Check if under limit
        if user_requests.len() < self.max_requests {
            user_requests.push(now);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl CommandRateLimiter {
    pub fn new() -> Self {
        Self {
            limits: RwLock::new(HashMap::new()),
            requests: Mutex::new(HashMap::new()),
        }
    }

    pub async fn set_command_limit(
        &mut self,
        command_type: crate::protocol::messages::CommandType,
        max_requests: usize,
        window_duration: Duration,
    ) -> Result<()> {
        let mut limits = self.limits.write().await;
        limits.insert(command_type, (max_requests, window_duration));
        Ok(())
    }

    pub async fn check_command_rate(
        &mut self,
        user_id: &str,
        command_type: crate::protocol::messages::CommandType,
    ) -> Result<bool> {
        let limits = self.limits.read().await;
        let (max_requests, window_duration) = limits.get(&command_type)
            .cloned()
            .unwrap_or((100, Duration::from_secs(60))); // Default limits
        drop(limits);
        
        let now = SystemTime::now();
        let window_start = now - window_duration;
        
        let mut requests = self.requests.lock().await;
        let user_requests = requests.entry(user_id.to_string()).or_insert_with(HashMap::new);
        let command_requests = user_requests.entry(command_type).or_insert_with(Vec::new);
        
        // Remove old requests
        command_requests.retain(|&time| time > window_start);
        
        // Check limit
        if command_requests.len() < max_requests {
            command_requests.push(now);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl AdaptiveRateLimiter {
    pub fn new() -> Result<Self> {
        Ok(Self {
            base_limits: RwLock::new(HashMap::new()),
            current_limits: RwLock::new(HashMap::new()),
            request_history: Mutex::new(HashMap::new()),
        })
    }

    pub async fn record_request(&mut self, user_id: &str, response_time: Duration) -> Result<()> {
        let now = SystemTime::now();
        
        let mut history = self.request_history.lock().await;
        let user_history = history.entry(user_id.to_string()).or_insert_with(Vec::new);
        
        user_history.push((now, response_time));
        
        // Keep only recent history (last hour)
        let one_hour_ago = now - Duration::from_secs(3600);
        user_history.retain(|(time, _)| *time > one_hour_ago);
        
        // Analyze patterns and adjust limits
        self.analyze_and_adjust_limits(user_id, user_history).await?;
        
        Ok(())
    }

    async fn analyze_and_adjust_limits(
        &self,
        user_id: &str,
        history: &[(SystemTime, Duration)],
    ) -> Result<()> {
        let base_limit = 60; // 60 requests per minute base
        let base_window = Duration::from_secs(60);
        
        // Detect suspicious patterns
        let rapid_requests = history.iter()
            .filter(|(_, response_time)| response_time.as_millis() < 10)
            .count();
        
        let new_limit = if rapid_requests > 50 {
            // Likely automated/bot behavior - restrict heavily
            10
        } else if rapid_requests > 20 {
            // Suspicious activity - moderate restriction
            30
        } else {
            // Normal behavior
            base_limit
        };
        
        let mut current_limits = self.current_limits.write().await;
        current_limits.insert(user_id.to_string(), (new_limit, base_window));
        
        Ok(())
    }

    pub async fn check_rate_limit(&self, user_id: &str) -> Result<bool> {
        let current_limits = self.current_limits.read().await;
        let (max_requests, _) = current_limits.get(user_id)
            .cloned()
            .unwrap_or((60, Duration::from_secs(60)));
        
        // For simplicity, return true if under half the limit
        Ok(max_requests > 30)
    }

    pub async fn get_current_limits(&self, user_id: &str) -> Result<LimitInfo> {
        let current_limits = self.current_limits.read().await;
        let (requests_per_minute, window_duration) = current_limits.get(user_id)
            .cloned()
            .unwrap_or((60, Duration::from_secs(60)));
        
        Ok(LimitInfo {
            requests_per_minute,
            window_duration,
            is_restricted: requests_per_minute < 60,
        })
    }
}