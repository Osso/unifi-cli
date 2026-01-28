# UniFi CLI

Rust CLI tool to access UniFi router API settings.

## Structure

```
src/
  main.rs       - CLI args (clap), command dispatch
  api.rs        - HTTP client (reqwest), base GET helpers (get_rest, get_v2, get_setting, get_stat)
  config.rs     - Config file (~/.config/unifi/config.json)
  firewall.rs   - Firewall rules CRUD, groups, traffic rules
  dns.rs        - Static DNS records CRUD
  clients.rs    - Online/offline/all clients
  devices.rs    - UniFi devices
  internet.rs   - WAN/DNS settings
  networks.rs   - Networks/VLANs
  security.rs   - IPS, ad blocking, DNS filtering
  vpn.rs        - Teleport, site-to-site, WireGuard, clients
  wifi.rs       - WLAN configurations
```

## Architecture

- `api::Client` holds reqwest client, base URL, API key
- Domain modules (firewall.rs, dns.rs, etc.) add methods to `Client` via `impl Client` blocks
- All commands output JSON to stdout
- REST v1 endpoint: `/proxy/network/api/s/default/rest/{resource}`
- REST v2 endpoint: `/proxy/network/v2/api/site/default/{resource}`

## Firewall

`firewall.rs` handles CRUD for firewall rules:
- `get_firewall_rules` / `get_firewall_groups` / `get_traffic_rules` — GET
- `create_firewall_rule` — POST, merges caller fields over required defaults (NETv4, empty arrays, etc.)
- `update_firewall_rule` — PUT, partial update (only sends provided fields)
- `delete_firewall_rule` — DELETE by ID

CLI commands: `rules`, `groups`, `traffic`, `add`, `update <id>`, `delete <id>`

## Adding a new command

1. Add subcommand variant to the relevant enum in `main.rs`
2. Add the API method in the domain module (e.g., `firewall.rs`)
3. Wire up the match arm in `main.rs`

## Build

```bash
cargo build --release
```
