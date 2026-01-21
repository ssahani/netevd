use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const DEFAULT_AUDIT_LOG_PATH: &str = "/var/log/netevd/audit.log";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub result: AuditResult,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    NetworkEvent,
    ScriptExecution,
    ConfigChange,
    ApiRequest,
    RouteChange,
    RuleChange,
    InterfaceChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditResult {
    Success,
    Failure,
    Partial,
}

pub struct AuditLogger {
    log_path: PathBuf,
    enabled: bool,
}

impl AuditLogger {
    pub fn new(log_path: Option<PathBuf>, enabled: bool) -> Self {
        let log_path = log_path.unwrap_or_else(|| PathBuf::from(DEFAULT_AUDIT_LOG_PATH));

        // Ensure directory exists
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        Self { log_path, enabled }
    }

    pub fn log(&self, event: AuditLog) {
        if !self.enabled {
            return;
        }

        if let Err(e) = self.write_log(&event) {
            tracing::error!("Failed to write audit log: {}", e);
        }
    }

    fn write_log(&self, event: &AuditLog) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        let json = serde_json::to_string(event)?;
        writeln!(file, "{}", json)?;

        Ok(())
    }

    pub fn log_network_event(
        &self,
        interface: &str,
        event_type: &str,
        result: AuditResult,
        details: Option<serde_json::Value>,
    ) {
        let event = AuditLog {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: AuditEventType::NetworkEvent,
            actor: "netevd".to_string(),
            action: event_type.to_string(),
            resource: interface.to_string(),
            result,
            details,
        };

        self.log(event);
    }

    pub fn log_script_execution(
        &self,
        script: &Path,
        interface: &str,
        exit_code: i32,
        duration_ms: u64,
    ) {
        let result = if exit_code == 0 {
            AuditResult::Success
        } else {
            AuditResult::Failure
        };

        let details = serde_json::json!({
            "script": script.display().to_string(),
            "interface": interface,
            "exit_code": exit_code,
            "duration_ms": duration_ms,
        });

        let event = AuditLog {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: AuditEventType::ScriptExecution,
            actor: "netevd".to_string(),
            action: "execute_script".to_string(),
            resource: script.display().to_string(),
            result,
            details: Some(details),
        };

        self.log(event);
    }

    pub fn log_config_reload(&self, success: bool) {
        let result = if success {
            AuditResult::Success
        } else {
            AuditResult::Failure
        };

        let event = AuditLog {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: AuditEventType::ConfigChange,
            actor: "system".to_string(),
            action: "reload_config".to_string(),
            resource: "/etc/netevd/netevd.yaml".to_string(),
            result,
            details: None,
        };

        self.log(event);
    }

    pub fn log_api_request(
        &self,
        method: &str,
        path: &str,
        status_code: u16,
        source_ip: Option<String>,
    ) {
        let result = if status_code < 400 {
            AuditResult::Success
        } else {
            AuditResult::Failure
        };

        let details = serde_json::json!({
            "method": method,
            "path": path,
            "status_code": status_code,
            "source_ip": source_ip,
        });

        let event = AuditLog {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: AuditEventType::ApiRequest,
            actor: source_ip.unwrap_or_else(|| "unknown".to_string()),
            action: format!("{} {}", method, path),
            resource: path.to_string(),
            result,
            details: Some(details),
        };

        self.log(event);
    }

    pub fn log_route_change(&self, action: &str, destination: &str, gateway: Option<String>) {
        let details = serde_json::json!({
            "destination": destination,
            "gateway": gateway,
        });

        let event = AuditLog {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: AuditEventType::RouteChange,
            actor: "netevd".to_string(),
            action: action.to_string(),
            resource: destination.to_string(),
            result: AuditResult::Success,
            details: Some(details),
        };

        self.log(event);
    }

    pub fn log_routing_rule_change(&self, action: &str, source: &str, table: u32) {
        let details = serde_json::json!({
            "source": source,
            "table": table,
        });

        let event = AuditLog {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: AuditEventType::RuleChange,
            actor: "netevd".to_string(),
            action: action.to_string(),
            resource: format!("rule-{}", source),
            result: AuditResult::Success,
            details: Some(details),
        };

        self.log(event);
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(None, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_audit_logger() {
        let temp_file = NamedTempFile::new().unwrap();
        let logger = AuditLogger::new(Some(temp_file.path().to_path_buf()), true);

        logger.log_network_event("eth0", "routable", AuditResult::Success, None);
        logger.log_config_reload(true);

        // Verify log was written
        let contents = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(contents.contains("\"event_type\":\"network_event\""));
        assert!(contents.contains("\"event_type\":\"config_change\""));
    }
}
