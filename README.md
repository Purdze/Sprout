# Sprout

A desktop server management panel for [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) Minecraft servers.

## Features

- **Multi-server management** — manage multiple Pumpkin servers from a single window, or drag tabs into separate windows
- **Console** — view server logs and send commands in real time
- **Performance graphs** — monitor CPU, memory, and TPS with live charts
- **Player management** — see online players and run commands on them
- **Config editor** — browse and edit server configuration files
- **Cloudflare DNS** — create and manage SRV records for your server domains

## Download

Grab the latest release from the [Releases](https://github.com/Purdze/Sprout/releases) page. Available for Windows, Linux, and macOS.

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (LTS)
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)

### Setup

```bash
pnpm install
pnpm tauri dev
```

### Build

```bash
pnpm tauri build
```

### Linting

```bash
pnpm lint        # ESLint
pnpm format      # Prettier
cd src-tauri && cargo clippy  # Clippy
```

## Support

Join the [Discord](https://discord.gg/qsRhJUP4q5) for help and discussion.
