# unifi-cli

[![CI](https://github.com/Osso/unifi-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/Osso/unifi-cli/actions/workflows/ci.yml)
[![GitHub release](https://img.shields.io/github/v/release/Osso/unifi-cli)](https://github.com/Osso/unifi-cli/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

CLI for UniFi router API access.

## Installation

```bash
cargo install --path .
```

## Setup

```bash
unifi config
```

## Usage

```bash
unifi internet all    # Show all WAN settings
unifi dns             # Static DNS records
unifi networks        # Network/VLAN settings
unifi wifi            # WiFi/WLAN settings
unifi devices         # UniFi devices (APs, switches, gateways)
unifi clients         # Connected clients
unifi firewall        # Firewall rules
unifi security        # Security settings (IPS, ad blocking)
unifi vpn             # VPN settings (Teleport, WireGuard)
```

## License

MIT
