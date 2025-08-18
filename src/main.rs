pub mod protocol;
pub mod server;
pub mod platform;

use clap::{Parser, Subcommand};
use tracing::info;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "system-controller")]
#[command(about = "Cross-platform remote system control server")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the system control server
    Server {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Address to bind to
        #[arg(short, long, default_value = "127.0.0.1")]
        address: String,
    },
    /// Test system capabilities
    Test,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Server { port, address } => {
            info!("Starting system controller server on {}:{}", address, port);
            // TODO: Start server
            Ok(())
        }
        Commands::Test => {
            info!("Testing system capabilities");
            // TODO: Run capability tests
            Ok(())
        }
    }
}
