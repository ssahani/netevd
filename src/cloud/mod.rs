pub mod aws;
pub mod azure;
pub mod gcp;

use anyhow::Result;

#[derive(Debug, Clone)]
pub enum CloudProvider {
    AWS,
    Azure,
    GCP,
    None,
}

impl CloudProvider {
    pub fn detect() -> Self {
        // Try to detect cloud provider from metadata services
        if Self::is_aws() {
            CloudProvider::AWS
        } else if Self::is_azure() {
            CloudProvider::Azure
        } else if Self::is_gcp() {
            CloudProvider::GCP
        } else {
            CloudProvider::None
        }
    }

    fn is_aws() -> bool {
        // Check for AWS metadata service
        std::path::Path::new("/sys/hypervisor/uuid").exists()
            && std::fs::read_to_string("/sys/hypervisor/uuid")
                .map(|s| s.to_lowercase().starts_with("ec2"))
                .unwrap_or(false)
    }

    fn is_azure() -> bool {
        // Check for Azure metadata
        std::path::Path::new("/var/lib/waagent").exists()
    }

    fn is_gcp() -> bool {
        // Check for GCP metadata
        std::path::Path::new("/sys/class/dmi/id/product_name").exists()
            && std::fs::read_to_string("/sys/class/dmi/id/product_name")
                .map(|s| s.contains("Google"))
                .unwrap_or(false)
    }
}
