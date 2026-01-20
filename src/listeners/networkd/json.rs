// SPDX-License-Identifier: LGPL-3.0-or-later

//! LinkDescribe JSON generation

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::api::LinkState;

/// LinkDescribe structure for JSON emission
#[derive(Debug, Serialize, Deserialize)]
pub struct LinkDescribe {
    pub ifindex: u32,
    pub ifname: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_state: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub oper_state: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub carrier_state: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_state: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4_address_state: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_address_state: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub online_state: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dns: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub domains: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub addresses: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway6: Option<String>,
}

/// Build LinkDescribe JSON from link state and network info
pub fn build_link_describe_json(
    ifindex: u32,
    ifname: String,
    link_state: &LinkState,
    addresses: Vec<String>,
) -> Result<Value> {
    let describe = LinkDescribe {
        ifindex,
        ifname,
        admin_state: if !link_state.admin_state.is_empty() {
            Some(link_state.admin_state.clone())
        } else {
            None
        },
        oper_state: if !link_state.oper_state.is_empty() {
            Some(link_state.oper_state.clone())
        } else {
            None
        },
        carrier_state: if !link_state.carrier_state.is_empty() {
            Some(link_state.carrier_state.clone())
        } else {
            None
        },
        address_state: if !link_state.address_state.is_empty() {
            Some(link_state.address_state.clone())
        } else {
            None
        },
        ipv4_address_state: if !link_state.ipv4_address_state.is_empty() {
            Some(link_state.ipv4_address_state.clone())
        } else {
            None
        },
        ipv6_address_state: if !link_state.ipv6_address_state.is_empty() {
            Some(link_state.ipv6_address_state.clone())
        } else {
            None
        },
        online_state: if !link_state.online_state.is_empty() {
            Some(link_state.online_state.clone())
        } else {
            None
        },
        dns: link_state.dns.clone(),
        domains: link_state.domains.clone(),
        addresses,
        gateway: link_state.gateway.clone(),
        gateway6: link_state.gateway6.clone(),
    };

    serde_json::to_value(describe).map_err(Into::into)
}
