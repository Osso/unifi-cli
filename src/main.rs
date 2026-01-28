mod api;
mod clients;
mod config;
mod devices;
mod dns;
mod firewall;
mod internet;
mod networks;
mod security;
mod vpn;
mod wifi;

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
    /// Static DNS records
    Dns {
        #[command(subcommand)]
        command: DnsCommands,
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
    /// Create a firewall rule
    Add {
        /// Rule name
        #[arg(long)]
        name: String,
        /// Action: accept, drop, reject
        #[arg(long)]
        action: String,
        /// Ruleset: LAN_IN, LAN_OUT, LAN_LOCAL, WAN_IN, WAN_OUT, WAN_LOCAL, etc.
        #[arg(long)]
        ruleset: String,
        /// Rule index (priority order)
        #[arg(long)]
        rule_index: u32,
        /// Source address (CIDR or IP)
        #[arg(long)]
        src_address: Option<String>,
        /// Destination address (CIDR or IP)
        #[arg(long)]
        dst_address: Option<String>,
        /// Protocol: tcp, udp, tcp_udp, all, etc.
        #[arg(long)]
        protocol: Option<String>,
        /// Source port
        #[arg(long)]
        src_port: Option<String>,
        /// Destination port
        #[arg(long)]
        dst_port: Option<String>,
        /// Source firewall group IDs (comma-separated)
        #[arg(long, value_delimiter = ',')]
        src_firewallgroup_ids: Option<Vec<String>>,
        /// Destination firewall group IDs (comma-separated)
        #[arg(long, value_delimiter = ',')]
        dst_firewallgroup_ids: Option<Vec<String>>,
        /// Enable the rule (default: true)
        #[arg(long, default_value_t = true)]
        enabled: bool,
        /// Enable logging
        #[arg(long)]
        logging: bool,
    },
    /// Update a firewall rule by ID
    Update {
        /// Rule ID
        id: String,
        /// Rule name
        #[arg(long)]
        name: Option<String>,
        /// Action: accept, drop, reject
        #[arg(long)]
        action: Option<String>,
        /// Rule index (priority order)
        #[arg(long)]
        rule_index: Option<u32>,
        /// Source address (CIDR or IP)
        #[arg(long)]
        src_address: Option<String>,
        /// Destination address (CIDR or IP)
        #[arg(long)]
        dst_address: Option<String>,
        /// Protocol: tcp, udp, tcp_udp, all, etc.
        #[arg(long)]
        protocol: Option<String>,
        /// Source port
        #[arg(long)]
        src_port: Option<String>,
        /// Destination port
        #[arg(long)]
        dst_port: Option<String>,
        /// Source firewall group IDs (comma-separated)
        #[arg(long, value_delimiter = ',')]
        src_firewallgroup_ids: Option<Vec<String>>,
        /// Destination firewall group IDs (comma-separated)
        #[arg(long, value_delimiter = ',')]
        dst_firewallgroup_ids: Option<Vec<String>>,
        /// Enable or disable the rule
        #[arg(long)]
        enabled: Option<bool>,
        /// Enable or disable logging
        #[arg(long)]
        logging: Option<bool>,
    },
    /// Delete a firewall rule by ID
    Delete {
        /// Rule ID
        id: String,
    },
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
    /// Reconnect a client (kick and let it rejoin)
    Reconnect {
        /// Client MAC address (e.g., aa:bb:cc:dd:ee:ff)
        mac: String,
    },
}

#[derive(Subcommand)]
enum InternetCommands {
    /// Show all WAN settings
    All,
    /// Show DNS settings
    Dns,
}

#[derive(Subcommand)]
enum DnsCommands {
    /// List static DNS records
    List,
    /// Add a static DNS record (A record)
    Add {
        /// Hostname (e.g., git.localdomain)
        name: String,
        /// IP address (e.g., 192.168.2.32)
        ip: String,
    },
    /// Delete a static DNS record by ID
    Delete {
        /// Record ID
        id: String,
    },
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
        Commands::Dns { command } => match command {
            DnsCommands::List => {
                let client = get_client()?;
                let records = client.get_dns_records().await?;
                println!("{}", serde_json::to_string_pretty(&records)?);
            }
            DnsCommands::Add { name, ip } => {
                let client = get_client()?;
                let record = client.create_dns_record(&name, &ip).await?;
                println!("{}", serde_json::to_string_pretty(&record)?);
            }
            DnsCommands::Delete { id } => {
                let client = get_client()?;
                client.delete_dns_record(&id).await?;
                println!("Deleted DNS record {}", id);
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
            FirewallCommands::Add {
                name,
                action,
                ruleset,
                rule_index,
                src_address,
                dst_address,
                protocol,
                src_port,
                dst_port,
                src_firewallgroup_ids,
                dst_firewallgroup_ids,
                enabled,
                logging,
            } => {
                let client = get_client()?;
                let mut rule = serde_json::Map::new();
                rule.insert("name".into(), serde_json::json!(name));
                rule.insert("action".into(), serde_json::json!(action));
                rule.insert("ruleset".into(), serde_json::json!(ruleset));
                rule.insert("rule_index".into(), serde_json::json!(rule_index));
                rule.insert("enabled".into(), serde_json::json!(enabled));
                rule.insert("logging".into(), serde_json::json!(logging));
                rule.insert("protocol".into(), serde_json::json!(protocol.unwrap_or_else(|| "all".to_string())));
                rule.insert("src_address".into(), serde_json::json!(src_address.unwrap_or_default()));
                rule.insert("dst_address".into(), serde_json::json!(dst_address.unwrap_or_default()));
                rule.insert("src_port".into(), serde_json::json!(src_port.unwrap_or_default()));
                rule.insert("dst_port".into(), serde_json::json!(dst_port.unwrap_or_default()));
                rule.insert("src_firewallgroup_ids".into(), serde_json::json!(src_firewallgroup_ids.unwrap_or_default()));
                rule.insert("dst_firewallgroup_ids".into(), serde_json::json!(dst_firewallgroup_ids.unwrap_or_default()));
                let created = client.create_firewall_rule(&rule).await?;
                println!("{}", serde_json::to_string_pretty(&created)?);
            }
            FirewallCommands::Update {
                id,
                name,
                action,
                rule_index,
                src_address,
                dst_address,
                protocol,
                src_port,
                dst_port,
                src_firewallgroup_ids,
                dst_firewallgroup_ids,
                enabled,
                logging,
            } => {
                let client = get_client()?;
                let mut fields = serde_json::Map::new();
                if let Some(v) = name { fields.insert("name".into(), serde_json::json!(v)); }
                if let Some(v) = action { fields.insert("action".into(), serde_json::json!(v)); }
                if let Some(v) = rule_index { fields.insert("rule_index".into(), serde_json::json!(v)); }
                if let Some(v) = src_address { fields.insert("src_address".into(), serde_json::json!(v)); }
                if let Some(v) = dst_address { fields.insert("dst_address".into(), serde_json::json!(v)); }
                if let Some(v) = protocol { fields.insert("protocol".into(), serde_json::json!(v)); }
                if let Some(v) = src_port { fields.insert("src_port".into(), serde_json::json!(v)); }
                if let Some(v) = dst_port { fields.insert("dst_port".into(), serde_json::json!(v)); }
                if let Some(v) = src_firewallgroup_ids { fields.insert("src_firewallgroup_ids".into(), serde_json::json!(v)); }
                if let Some(v) = dst_firewallgroup_ids { fields.insert("dst_firewallgroup_ids".into(), serde_json::json!(v)); }
                if let Some(v) = enabled { fields.insert("enabled".into(), serde_json::json!(v)); }
                if let Some(v) = logging { fields.insert("logging".into(), serde_json::json!(v)); }
                let updated = client.update_firewall_rule(&id, &fields).await?;
                println!("{}", serde_json::to_string_pretty(&updated)?);
            }
            FirewallCommands::Delete { id } => {
                let client = get_client()?;
                client.delete_firewall_rule(&id).await?;
                println!("Deleted firewall rule {}", id);
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
            ClientsCommands::Reconnect { mac } => {
                let client = get_client()?;
                client.kick_client(&mac).await?;
                println!("Kicked client {}, it will reconnect", mac);
            }
        },
    }

    Ok(())
}
