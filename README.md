# Sprout

A desktop server management panel for [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) Minecraft servers. Built with [Tauri v2](https://v2.tauri.app/), Vue 3, and Rust.

![Console](<img width="608" height="440" alt="image" src="https://github.com/user-attachments/assets/281f9406-60db-4d03-acc4-79295f60a2fa" />
)

## Features

### Multi-Server Management

Manage multiple Pumpkin servers from a single window. Each server gets its own tab with a live status indicator (running, starting, stopped). Drag a tab out of the window to pop it into its own separate window — drag it back to re-dock it.

![Tabs](<img width="216" height="67" alt="image" src="https://github.com/user-attachments/assets/1affccc3-205a-4ad0-ab26-063543b65ad3" />
)

### Performance Graphs

Monitor server health with live SVG graphs for CPU usage, memory usage, and TPS. Graphs auto-scale to your data and keep a rolling 2-minute history.

![Graphs](<img width="748" height="443" alt="image" src="https://github.com/user-attachments/assets/17bfee66-abd2-4b37-acaa-a0aaec2a62ee" />
)

### Player Management

See all online and previously joined players at a glance. Search and filter by name. Switch between grid and list views. Quick actions let you kick, ban, or ban-ip players directly from the list.

![Players](screenshots/players.png)

### Player Inventory Inspector

Click any player to open a detailed inventory view with:

- **Inventory, hotbar, armor, and offhand** displayed with Minecraft item icons
- **Ender chest** tab with full 27-slot grid
- **Health, food, and XP** shown with authentic Minecraft HUD icons
- **Player stats** — world, position, gamemode, last slept
- **Quick actions** — whitelist, op/deop (conditional), kick, ban, ban-ip
- **Last refreshed** timestamp with manual refresh

![Inventory](screenshots/inventory.png)

### Config Editor

Browse and edit server configuration files across the `config`, `data`, `worlds`, `logs`, and `plugins` directories. Files are editable when the server is stopped, with unsaved change protection. Log and compressed files open in read-only mode with a copy-to-clipboard button.

![Config](screenshots/config.png)

### Cloudflare DNS

Connect your Cloudflare account with an API token to manage SRV records for your server domains. Create new records with a subdomain, target, and port — or delete existing ones. Supports multiple zones.

![Domains](screenshots/domains.png)

### Commands

Save frequently used server commands with custom names for quick access. Run any saved command with a single click.

### Plugin Manager

Browse and manage plugins in the `plugins` directory. Enable or disable plugins by toggling their file extension. Click a plugin to browse its config files and edit them with the visual config editor.

### Visual Config Editor

Edit TOML and JSON config files with a form-based UI. Booleans render as toggles, numbers get sliders where appropriate, and nested objects are collapsible sections. Switch between visual and raw text editing at any time.

### Auto-Updater

Sprout checks for updates on launch and prompts you to install when a new version is available. Updates are downloaded, verified, and installed automatically — then the app restarts. You can also manually check from the About dialog (? icon in the titlebar).

### Server Downloader

When adding a new server, Sprout can automatically download the latest Pumpkin binary for your platform (Windows, Linux, macOS) directly from GitHub releases, with a progress bar.

### Window State Persistence

Sprout remembers your window size and position between sessions.

## Download

Grab the latest release from the [Releases](https://github.com/Purdze/Sprout/releases) page. Available for Windows, Linux, and macOS.

## Support

Join the [Discord](https://discord.gg/qsRhJUP4q5) for help and discussion.
