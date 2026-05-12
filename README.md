# traac

**traac** (no caps) (pronounced like tracy or track) is a lightweight last.fm scrobbler made for Hyprland support. Mainly a small pop-up window (as a shell overlay), with optional notification of track change via notification daemon or window itself.

## Tech Stack

- **Language:** Rust
- **UI:** Iced (with iced_layershell for wlroots-layer-shell support)
- **Scrobbling:** last-fm-rs
- **Track info:** mpris (D-Bus MPRIS interface)
- **Config:** serde + toml, stored at `~/.config/traac/config.toml`

## Design Philosophy

Squared, almost TUI-style with few borders and easy configuration via a `.config/` file. In-UI settings should edit that config file as well. Features heavy UI customization and feature settings.

Initial UI color scheme uses the color scheme defined in `eyc.txt`, based on Catppuccin Mocha.

## Building

### Prerequisites

- Rust toolchain (1.88+)
- Wayland development libraries (for iced_layershell)

### Build

```bash
git clone <repo-url>
cd traac
cargo build --release
```

## Running

```bash
cargo run --release
```

The application will appear as a small overlay window on your Wayland desktop (tested on Hyprland). It reads configuration from `~/.config/traac/config.toml` on startup.

### Configuration

A default config will be used if none exists. Create or edit `~/.config/traac/config.toml` to customize:

- Last.fm API credentials
- UI position and color scheme
- Scrobbling behavior (poll interval, enable/disable)

## License

GNU GPLv3

## Notes

The majority of this code is AI-generated. Do you trust traac to execute code on your machine? If not, this isn't for you.

No Tauri. Seriously. All my homies hate webapps.
