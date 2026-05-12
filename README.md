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

#### Configuration File Structure

The configuration file uses TOML format. Here's a complete example with all available options:

```toml
# Last.fm API credentials
# Get your API key and secret from: https://www.last.fm/api/account/create
[lastfm]
api_key = "your_api_key_here"
api_secret = "your_api_secret_here"
session_key = "auto-generated-after-auth"  # Set after authentication
username = "your_lastfm_username"          # Set after authentication

# UI configuration
[ui]
show_notifications = true  # Enable desktop notifications on track change

# Color scheme (Catppuccin-inspired)
# All colors should be valid hex strings
[ui.color_scheme]
base = "#11111b"           # Main background color
slightly_lighter = "#1e1e2e"  # Secondary elements
accent_grey = "#6C7086"    # Subtle text/borders
bright = "#BAC2DE"         # Highlight color
text = "#cdd6f4"           # Primary text color

# Window position
[ui.position]
anchor = "BottomRight"  # Options: TopLeft, TopRight, BottomLeft, BottomRight
x = 20                   # Horizontal margin (pixels from edge)
y = 20                   # Vertical margin (pixels from edge)

# General settings
[general]
scrobble_enabled = true # Enable/disable scrobbling
poll_interval_secs = 5 # How often to check for track changes
ignored_players = [] # List of MPRIS player names to ignore (e.g., ["vlc", "firefox"])
```

#### Configuration Options Explained

**Last.fm Section:**
- `api_key`: Your Last.fm API key (required for authentication)
- `api_secret`: Your Last.fm API secret (required for authentication)
- `session_key`: Automatically populated after successful authentication
- `username`: Your Last.fm username, populated after authentication

**UI Section:**
- `show_notifications`: Whether to show desktop notifications when tracks change

**UI Color Scheme:**
- All five Catppuccin-inspired colors are used throughout the UI
- Customize to match your desktop theme

**UI Position:**
- `anchor`: Which corner of the screen to place the window
- `x`: Distance from left/right edge (depending on anchor)
- `y`: Distance from top/bottom edge (depending on anchor)

**General:**
- `scrobble_enabled`: Toggle scrobbling on/off without removing credentials
- `poll_interval_secs`: How frequently to poll MPRIS for track updates (default: 5)
- `ignored_players`: List of MPRIS player names to ignore. Supports glob patterns (e.g., `["vlc", "firefox.*", "chromium.*"]`). Use `*` to match any characters in player names (useful for instances like `firefox.instance_1_1168`).

#### Getting Last.fm Credentials

1. Visit https://www.last.fm/api/account/create
2. Create a new application (you can use any callback URL)
3. Copy your **API Key** and **API Secret** into the config file
4. Run traac and follow the authentication flow to get your session key

## License

GNU GPLv3

## Notes

The majority of this code is AI-generated. Do you trust traac to execute code on your machine? If not, this isn't for you.

No Tauri. Seriously. All my homies hate webapps.
