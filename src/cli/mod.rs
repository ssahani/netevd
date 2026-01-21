pub mod handler;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "netevd")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "/etc/netevd/netevd.yaml")]
    pub config: PathBuf,

    /// Enable dry-run mode (don't execute scripts or make changes)
    #[arg(long)]
    pub dry_run: bool,

    /// Enable verbose logging
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the daemon (default)
    Start {
        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,
    },

    /// Show daemon status
    Status {
        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,

        /// API endpoint
        #[arg(long, default_value = "http://localhost:9090")]
        endpoint: String,
    },

    /// List network interfaces
    List {
        /// Resource type to list
        #[arg(value_enum)]
        resource: ListResource,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,

        /// API endpoint
        #[arg(long, default_value = "http://localhost:9090")]
        endpoint: String,
    },

    /// Show details about a resource
    Show {
        /// Resource type
        #[arg(value_enum)]
        resource: ShowResource,

        /// Resource name/identifier
        name: String,

        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,

        /// API endpoint
        #[arg(long, default_value = "http://localhost:9090")]
        endpoint: String,
    },

    /// Watch events in real-time
    Events {
        /// Follow events (like tail -f)
        #[arg(short, long)]
        follow: bool,

        /// Filter by interface name
        #[arg(short, long)]
        interface: Option<String>,

        /// Filter by event type
        #[arg(short = 't', long)]
        event_type: Option<String>,

        /// Number of recent events to show
        #[arg(short, long, default_value = "10")]
        tail: usize,

        /// Output format
        #[arg(short = 'f', long, default_value = "text")]
        format: OutputFormat,

        /// API endpoint
        #[arg(long, default_value = "http://localhost:9090")]
        endpoint: String,
    },

    /// Reload configuration
    Reload {
        /// API endpoint
        #[arg(long, default_value = "http://localhost:9090")]
        endpoint: String,
    },

    /// Validate configuration file
    Validate {
        /// Configuration file to validate
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Test a script execution
    Test {
        /// Path to script to test
        script: PathBuf,

        /// Interface name for testing
        #[arg(short, long, default_value = "eth0")]
        interface: String,

        /// Event type for testing
        #[arg(short = 't', long, default_value = "routable")]
        event_type: String,

        /// IP address for testing
        #[arg(long)]
        ip: Option<String>,
    },

    /// Show version information
    Version {
        /// Show detailed version info
        #[arg(short, long)]
        detailed: bool,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
    Table,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ListResource {
    Interfaces,
    Routes,
    Rules,
    Scripts,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ShowResource {
    Interface,
    Route,
    Rule,
    Status,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}
