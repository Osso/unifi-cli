mod api;
mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "unifi")]
#[command(about = "CLI tool to access UniFi router API")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure host and API key
    Config {
        /// UniFi controller/UDM host (e.g., 192.168.2.1)
        #[arg(short = 'H', long)]
        host: Option<String>,
        /// API key
        #[arg(short, long)]
        api_key: Option<String>,
    },
    /// Internet/WAN settings
    Internet {
        #[command(subcommand)]
        command: InternetCommands,
    },
    /// Security settings (IPS, ad blocking, DNS filtering)
    Security,
    /// Firewall rules and policies
    Firewall {
        #[command(subcommand)]
        command: FirewallCommands,
    },
    /// VPN settings (Teleport, WireGuard)
    Vpn {
        #[command(subcommand)]
        command: VpnCommands,
    },
    /// Network/VLAN settings
    Networks,
    /// WiFi/WLAN settings
    Wifi,
    /// UniFi devices (APs, switches, gateways)
    Devices,
    /// Connected clients
    Clients {
        #[command(subcommand)]
        command: ClientsCommands,
    },
}

#[derive(Subcommand)]
enum FirewallCommands {
    /// List firewall rules
    Rules,
    /// List firewall groups (IP groups, port groups)
    Groups,
    /// List traffic rules
    Traffic,
}

#[derive(Subcommand)]
enum VpnCommands {
    /// Show Teleport VPN settings
    Teleport,
    /// Show Site-to-Site VPN settings
    SiteToSite,
    /// List VPN servers
    Servers,
    /// List VPN clients
    Clients,
}

#[derive(Subcommand)]
enum ClientsCommands {
    /// All known clients
    All,
    /// Currently online clients
    Online,
    /// Offline clients
    Offline,
}

#[derive(Subcommand)]
enum InternetCommands {
    /// Show all WAN settings
    All,
    /// Show DNS settings
    Dns,
}

fn get_client() -> Result<api::Client> {
    let cfg = config::load_config()?;
    let host = cfg
        .host
        .ok_or_else(|| anyhow::anyhow!("Not configured. Run 'unifi config' first"))?;
    let api_key = cfg
        .api_key
        .ok_or_else(|| anyhow::anyhow!("API key not configured. Run 'unifi config' first"))?;
    api::Client::new(&host, &api_key)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { host, api_key } => {
            let mut cfg = config::load_config().unwrap_or_default();

            if let Some(h) = host {
                cfg.host = Some(h);
            }
            if let Some(k) = api_key {
                cfg.api_key = Some(k);
            }

            config::save_config(&cfg)?;
            println!("Config saved to ~/.config/unifi/config.json");
        }
        Commands::Internet { command } => match command {
            InternetCommands::All => {
                let client = get_client()?;
                let wan = client.get_wan_settings().await?;
                println!("{}", serde_json::to_string_pretty(&wan)?);
            }
            InternetCommands::Dns => {
                let client = get_client()?;
                let dns = client.get_dns_settings().await?;
                println!("{}", serde_json::to_string_pretty(&dns)?);
            }
        },
        Commands::Security => {
            let client = get_client()?;
            let security = client.get_security_settings().await?;
            println!("{}", serde_json::to_string_pretty(&security)?);
        }
        Commands::Firewall { command } => match command {
            FirewallCommands::Rules => {
                let client = get_client()?;
                let rules = client.get_firewall_rules().await?;
                println!("{}", serde_json::to_string_pretty(&rules)?);
            }
            FirewallCommands::Groups => {
                let client = get_client()?;
                let groups = client.get_firewall_groups().await?;
                println!("{}", serde_json::to_string_pretty(&groups)?);
            }
            FirewallCommands::Traffic => {
                let client = get_client()?;
                let traffic = client.get_traffic_rules().await?;
                println!("{}", serde_json::to_string_pretty(&traffic)?);
            }
        },
        Commands::Vpn { command } => match command {
            VpnCommands::Teleport => {
                let client = get_client()?;
                let teleport = client.get_vpn_teleport().await?;
                println!("{}", serde_json::to_string_pretty(&teleport)?);
            }
            VpnCommands::SiteToSite => {
                let client = get_client()?;
                let s2s = client.get_vpn_site_to_site().await?;
                println!("{}", serde_json::to_string_pretty(&s2s)?);
            }
            VpnCommands::Servers => {
                let client = get_client()?;
                let servers = client.get_vpn_servers().await?;
                println!("{}", serde_json::to_string_pretty(&servers)?);
            }
            VpnCommands::Clients => {
                let client = get_client()?;
                let clients = client.get_vpn_clients().await?;
                println!("{}", serde_json::to_string_pretty(&clients)?);
            }
        },
        Commands::Networks => {
            let client = get_client()?;
            let networks = client.get_networks().await?;
            println!("{}", serde_json::to_string_pretty(&networks)?);
        }
        Commands::Wifi => {
            let client = get_client()?;
            let wifi = client.get_wifi().await?;
            println!("{}", serde_json::to_string_pretty(&wifi)?);
        }
        Commands::Devices => {
            let client = get_client()?;
            let devices = client.get_devices().await?;
            println!("{}", serde_json::to_string_pretty(&devices)?);
        }
        Commands::Clients { command } => match command {
            ClientsCommands::All => {
                let client = get_client()?;
                let clients = client.get_clients_all().await?;
                println!("{}", serde_json::to_string_pretty(&clients)?);
            }
            ClientsCommands::Online => {
                let client = get_client()?;
                let clients = client.get_clients_online().await?;
                println!("{}", serde_json::to_string_pretty(&clients)?);
            }
            ClientsCommands::Offline => {
                let client = get_client()?;
                let clients = client.get_clients_offline().await?;
                println!("{}", serde_json::to_string_pretty(&clients)?);
            }
        }
    }

    Ok(())
}
