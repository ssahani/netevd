// SPDX-License-Identifier: LGPL-3.0-or-later

//! Network state management and operations

pub mod address;
pub mod link;
pub mod route;
pub mod routing_rule;
pub mod watcher;

use std::collections::HashMap;
use std::net::IpAddr;

/// Represents a routing rule (from/to)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoutingRule {
    pub address: IpAddr,
    pub table: u32,
    pub is_from: bool, // true for "from", false for "to"
}

/// Represents a route entry
#[derive(Debug, Clone)]
pub struct RouteEntry {
    pub ifindex: u32,
    pub gateway: Option<IpAddr>,
    pub table: u32,
}

/// Network state tracking
#[derive(Debug, Default)]
pub struct NetworkState {
    /// Map link name to interface index
    pub links_by_name: HashMap<String, u32>,

    /// Map interface index to link name
    pub links_by_index: HashMap<u32, String>,

    /// Track routes by interface index and table
    pub routes: HashMap<(u32, u32), RouteEntry>,

    /// Track routing rules by address (for cleanup)
    pub routing_rules_from: HashMap<IpAddr, RoutingRule>,
    pub routing_rules_to: HashMap<IpAddr, RoutingRule>,
}

impl NetworkState {
    /// Create a new empty network state
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update a link
    pub fn add_link(&mut self, name: String, index: u32) {
        self.links_by_name.insert(name.clone(), index);
        self.links_by_index.insert(index, name);
    }

    /// Remove a link by index
    pub fn remove_link(&mut self, index: u32) {
        if let Some(name) = self.links_by_index.remove(&index) {
            self.links_by_name.remove(&name);
        }
        // Also remove associated routes and rules
        self.routes.retain(|(idx, _), _| *idx != index);
    }

    /// Get link name by index
    pub fn get_link_name(&self, index: u32) -> Option<&String> {
        self.links_by_index.get(&index)
    }

    /// Get link index by name
    pub fn get_link_index(&self, name: &str) -> Option<u32> {
        self.links_by_name.get(name).copied()
    }

    /// Add a route entry
    pub fn add_route(&mut self, ifindex: u32, table: u32, gateway: Option<IpAddr>) {
        let route = RouteEntry {
            ifindex,
            gateway,
            table,
        };
        self.routes.insert((ifindex, table), route);
    }

    /// Remove a route entry
    pub fn remove_route(&mut self, ifindex: u32, table: u32) {
        self.routes.remove(&(ifindex, table));
    }

    /// Add a routing rule (from)
    pub fn add_routing_rule_from(&mut self, address: IpAddr, table: u32) {
        let rule = RoutingRule {
            address,
            table,
            is_from: true,
        };
        self.routing_rules_from.insert(address, rule);
    }

    /// Add a routing rule (to)
    pub fn add_routing_rule_to(&mut self, address: IpAddr, table: u32) {
        let rule = RoutingRule {
            address,
            table,
            is_from: false,
        };
        self.routing_rules_to.insert(address, rule);
    }

    /// Remove routing rules for an address
    pub fn remove_routing_rules(&mut self, address: &IpAddr) {
        self.routing_rules_from.remove(address);
        self.routing_rules_to.remove(address);
    }

    /// Check if we have routing rules for an address
    pub fn has_routing_rules(&self, address: &IpAddr) -> bool {
        self.routing_rules_from.contains_key(address) || self.routing_rules_to.contains_key(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_add_remove_link() {
        let mut state = NetworkState::new();
        state.add_link("eth0".to_string(), 2);

        assert_eq!(state.get_link_index("eth0"), Some(2));
        assert_eq!(state.get_link_name(2), Some(&"eth0".to_string()));

        state.remove_link(2);
        assert_eq!(state.get_link_index("eth0"), None);
        assert_eq!(state.get_link_name(2), None);
    }

    #[test]
    fn test_routing_rules() {
        let mut state = NetworkState::new();
        let addr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10));

        state.add_routing_rule_from(addr, 10000);
        state.add_routing_rule_to(addr, 10000);

        assert!(state.has_routing_rules(&addr));

        state.remove_routing_rules(&addr);
        assert!(!state.has_routing_rules(&addr));
    }

    #[test]
    fn test_routes() {
        let mut state = NetworkState::new();
        let gateway = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        state.add_route(2, 254, Some(gateway));
        assert!(state.routes.contains_key(&(2, 254)));

        state.remove_route(2, 254);
        assert!(!state.routes.contains_key(&(2, 254)));
    }
}
