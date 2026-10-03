use clap::Parser;
use log::Level;
use logforth::{
    Diagnostic, Filter, append,
    filter::FilterResult,
    record::{self, FilterCriteria, Record},
};
use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr},
};
use vetis::{VetisServer, host::HostConfig};
use vetis_static::{StaticPathConfig, tokio::StaticPath};
use vetis_tokio::{Vetis, host::Host};

#[derive(Parser)]
#[command(
    name = "front",
    about = "front - a very tiny frontend server",
    long_about = r#"
front - a very tiny frontend server

Usage:
    front [OPTIONS]

Options:
    -h, --help       Print help information
    -V, --version    Print version information
    -i, --interface  <INTERFACE>
                     Interface to bind to
                     Default: 0.0.0.0
    -p, --port       <PORT>
                     Port to bind to
                     Default: 4444
    -r, --root       <ROOT>
                     Root directory to serve
                     Default: .
"#
)]
struct Args {
    #[arg(short, long, required = false, help = "Root directory to serve.")]
    root: Option<String>,
    #[arg(short, long, required = false, help = "Interface to bind to.")]
    interface: Option<String>,
    #[arg(short, long, required = false, help = "Port to bind to.")]
    port: Option<u16>,
}

#[derive(Debug)]
/// TargetFilter is used to dispatch by target
pub struct TargetFilter {
    target: String,
    level: Level,
}

impl TargetFilter {
    /// Create a net TargetFilter
    pub fn new(target: &str, level: Level) -> Self {
        Self { target: target.into(), level }
    }
}

impl Filter for TargetFilter {
    fn enabled(&self, criteria: &FilterCriteria, _: &[Box<dyn Diagnostic>]) -> FilterResult {
        let log_level = match self.level {
            Level::Error => record::Level::Error,
            Level::Warn => record::Level::Warn,
            Level::Info => record::Level::Info,
            Level::Debug => record::Level::Debug,
            Level::Trace => record::Level::Trace,
        };

        if criteria.target() == self.target && criteria.level() >= log_level {
            FilterResult::Accept
        } else {
            FilterResult::Reject
        }
    }

    fn matches(&self, record: &Record, diags: &[Box<dyn Diagnostic>]) -> FilterResult {
        let criteria = FilterCriteria::builder()
            .level(record.level())
            .target(record.target())
            .build();

        self.enabled(&criteria, diags)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut log = logforth::starter_log::builder()
        .dispatch(|d| {
            d.filter(TargetFilter::new("vetis", Level::Info))
                .append(append::Stdout::default())
        })
        .apply();

    let root = &args
        .root
        .unwrap_or(".".to_string());
    let port = args
        .port
        .unwrap_or(4444);

    let host_config = HostConfig::builder()
        .hostname("localhost")
        .bind_addresses(&[(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)])
        .root_directory(root)
        .build()?;

    let mut virtual_host = Host::new(host_config).await?;
    virtual_host.add_path(StaticPath::new(
        StaticPathConfig::builder()
            .uri("/")
            .directory(root)
            .extensions("\\.(html|js|css|svg|png|ico|jsx|ts|tsx|json|mjs)$")
            .index_files(vec!["index.html".into()])
            .build()?,
    ));

    let mut server = Vetis::builder()
        .add_host(virtual_host)
        .await?
        .build();

    server.run().await?;

    Ok(())
}
