use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    #[serde(default)]
    pub filters: Vec<Filter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub match_rule: MatchRule,
    pub action: FilterAction,
    #[serde(default)]
    pub scripts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface_pattern: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_family: Option<IpFamily>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IpFamily {
    Ipv4,
    Ipv6,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FilterAction {
    Execute,
    Ignore,
    Log,
}

#[derive(Debug, Clone)]
pub struct NetworkEvent {
    pub interface: String,
    pub event_type: String,
    pub backend: String,
    pub addresses: Vec<IpAddr>,
    pub has_gateway: bool,
    pub dns_servers: Vec<IpAddr>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn from_yaml(yaml: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(yaml)?)
    }

    pub fn should_execute(&self, event: &NetworkEvent) -> bool {
        for filter in &self.filters {
            if filter.matches(event) {
                match filter.action {
                    FilterAction::Execute => return true,
                    FilterAction::Ignore => return false,
                    FilterAction::Log => {
                        tracing::info!("Filter matched (log only): {:?}", event);
                        continue;
                    }
                }
            }
        }
        // Default: execute if no filters match
        true
    }

    pub fn get_scripts_for_event(&self, event: &NetworkEvent) -> Vec<String> {
        let mut scripts = Vec::new();
        for filter in &self.filters {
            if filter.matches(event) && filter.action == FilterAction::Execute {
                scripts.extend(filter.scripts.clone());
            }
        }
        scripts
    }
}

impl Filter {
    pub fn matches(&self, event: &NetworkEvent) -> bool {
        // Check interface exact match
        if let Some(ref interface) = self.match_rule.interface {
            if interface != &event.interface {
                return false;
            }
        }

        // Check interface pattern match
        if let Some(ref pattern) = self.match_rule.interface_pattern {
            if let Ok(regex) = Regex::new(&pattern.replace("*", ".*")) {
                if !regex.is_match(&event.interface) {
                    return false;
                }
            }
        }

        // Check event type
        if let Some(ref event_type) = self.match_rule.event_type {
            if event_type != &event.event_type {
                return false;
            }
        }

        // Check IP family
        if let Some(ref ip_family) = self.match_rule.ip_family {
            match ip_family {
                IpFamily::Ipv4 => {
                    if !event.addresses.iter().any(|addr| addr.is_ipv4()) {
                        return false;
                    }
                }
                IpFamily::Ipv6 => {
                    if !event.addresses.iter().any(|addr| addr.is_ipv6()) {
                        return false;
                    }
                }
                IpFamily::Any => {}
            }
        }

        // Check backend
        if let Some(ref backend) = self.match_rule.backend {
            if backend != &event.backend {
                return false;
            }
        }

        // Check condition (simple expression evaluation)
        if let Some(ref condition) = self.match_rule.condition {
            if !self.evaluate_condition(condition, event) {
                return false;
            }
        }

        true
    }

    fn evaluate_condition(&self, condition: &str, event: &NetworkEvent) -> bool {
        // Simple condition evaluator
        // Supports: has_gateway, dns_count > N, interface == "name"

        if condition.contains("has_gateway") {
            return event.has_gateway;
        }

        if condition.contains("dns_count") {
            if let Some(pos) = condition.find('>') {
                if let Ok(threshold) = condition[pos + 1..].trim().parse::<usize>() {
                    return event.dns_servers.len() > threshold;
                }
            }
            if let Some(pos) = condition.find('<') {
                if let Ok(threshold) = condition[pos + 1..].trim().parse::<usize>() {
                    return event.dns_servers.len() < threshold;
                }
            }
        }

        if condition.contains("interface ==") {
            if let Some(start) = condition.find('"') {
                if let Some(end) = condition[start + 1..].find('"') {
                    let iface = &condition[start + 1..start + 1 + end];
                    return event.interface == iface;
                }
            }
        }

        // Default: condition not recognized, return true
        true
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interface_pattern_match() {
        let filter_yaml = r#"
filters:
  - match_rule:
      interface_pattern: "eth*"
      event_type: "routable"
    action: execute
"#;

        let filter = EventFilter::from_yaml(filter_yaml).unwrap();
        let event = NetworkEvent {
            interface: "eth0".to_string(),
            event_type: "routable".to_string(),
            backend: "systemd-networkd".to_string(),
            addresses: vec![],
            has_gateway: true,
            dns_servers: vec![],
        };

        assert!(filter.should_execute(&event));
    }

    #[test]
    fn test_ignore_action() {
        let filter_yaml = r#"
filters:
  - match_rule:
      interface_pattern: "docker*"
    action: ignore
"#;

        let filter = EventFilter::from_yaml(filter_yaml).unwrap();
        let event = NetworkEvent {
            interface: "docker0".to_string(),
            event_type: "routable".to_string(),
            backend: "systemd-networkd".to_string(),
            addresses: vec![],
            has_gateway: false,
            dns_servers: vec![],
        };

        assert!(!filter.should_execute(&event));
    }

    #[test]
    fn test_condition_evaluation() {
        let filter_yaml = r#"
filters:
  - match_rule:
      interface: "wg0"
      condition: "has_gateway && dns_count > 0"
    action: execute
"#;

        let filter = EventFilter::from_yaml(filter_yaml).unwrap();
        let event = NetworkEvent {
            interface: "wg0".to_string(),
            event_type: "routable".to_string(),
            backend: "systemd-networkd".to_string(),
            addresses: vec![],
            has_gateway: true,
            dns_servers: vec!["8.8.8.8".parse().unwrap()],
        };

        assert!(filter.should_execute(&event));
    }
}
