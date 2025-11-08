# Global Search Launcher

A powerful Windows desktop application that provides instant, system-wide search functionality accessible via the `Ctrl+K` hotkey. Search files, applications, bookmarks, clipboard history, perform calculations, and more—all from a single, elegant interface.

![Global Search Launcher](docs/screenshot.png)

## Features

- 🔍 **Lightning-Fast File Search** - Powered by Everything SDK for instant file indexing
- 🚀 **Application Launcher** - Quick access to all installed applications
- 📋 **Clipboard History** - Search and restore recent clipboard items
- 🔖 **Browser Bookmarks** - Search bookmarks from Chrome, Edge, and Firefox
- 🧮 **Built-in Calculator** - Evaluate mathematical expressions on the fly
- ⚡ **Quick Actions** - System commands (shutdown, restart, lock, sleep, etc.)
- 🌐 **Web Search Fallback** - Automatically search the web when no local results match
- 📁 **Recent Files** - Quick access to recently opened files
- 🎨 **Theme Support** - Light, dark, and system theme options
- ⌨️ **Keyboard-First** - Fully operable without a mouse
- 🔄 **Auto-Updates** - Automatic background updates

## System Requirements

- **Operating System**: Windows 10 (version 1809 or later) or Windows 11
- **RAM**: 4GB minimum, 8GB recommended
- **Disk Space**: 100MB for application, 50MB for cache
- **Optional**: [Everything](https://www.voidtools.com/) 1.4+ for faster file search (recommended)

## Installation

### Option 1: Download Installer (Recommended)

1. Download the latest installer from the [Releases](https://github.com/yourusername/global-search-launcher/releases) page
2. Choose either:
   - `better-finder_x.x.x_x64_en-US.msi` - Windows Installer package
   - `better-finder_x.x.x_x64-setup.exe` - NSIS installer (recommended)
3. Run the installer and follow the on-screen instructions
4. The application will start automatically after installation

### Option 2: Build from Source

#### Prerequisites

- [Node.js](https://nodejs.org/) 18+ and npm
- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

#### Build Steps

```bash
# Clone the repository
git clone https://github.com/yourusername/global-search-launcher.git
cd global-search-launcher/better-finder

# Install dependencies
npm install

# Build the application
npm run tauri:build

# The installer will be in src-tauri/target/release/bundle/
```

## Usage

### Basic Usage

1. **Open the Search Bar**: Press `Ctrl+K` from anywhere in Windows
2. **Type Your Query**: Start typing to search files, apps, and more
3. **Navigate Results**: Use `↑` and `↓` arrow keys to navigate
4. **Execute**: Press `Enter` to open/execute the selected result
5. **Close**: Press `Esc` or click outside the window to close

### Search Types

#### File Search
Simply type part of a filename:
```
document.pdf
report 2024
```

#### Application Search
Type the name of an application:
```
chrome
notepad
visual studio
```

#### Calculator
Type a mathematical expression:
```
2 + 2
15 * 8
(100 + 50) / 3
```

#### Clipboard History
Search clipboard history with the `clip:` prefix:
```
clip:password
clip:email
```

#### Quick Actions
Type system commands:
```
shutdown
restart
lock
sleep
```

#### Web Search
Type any query that doesn't match local results, and press `Enter` to search the web:
```
how to use keyboard shortcuts
weather today
```

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+K` | Open/close search bar |
| `↑` / `↓` | Navigate results |
| `Enter` | Execute selected result |
| `Esc` | Close search bar |

### Settings

Access settings by:
1. Right-clicking the system tray icon
2. Selecting "Settings"

Available settings:
- **Hotkey**: Customize the global keyboard shortcut
- **Theme**: Choose between light, dark, or system theme
- **Max Results**: Set the maximum number of results to display
- **Enabled Providers**: Enable/disable specific search providers
- **Start with Windows**: Launch automatically on system startup

## Configuration

### Everything SDK Setup (Optional but Recommended)

For the fastest file search experience, install Everything:

1. Download [Everything](https://www.voidtools.com/downloads/) (1.4 or later)
2. Install and run Everything
3. Global Search Launcher will automatically detect and use it

If Everything is not installed, the application will fall back to Windows Search.

### Auto-Start

To enable auto-start:
1. Open Settings from the system tray
2. Enable "Start with Windows"

Or manually add to Windows startup:
1. Press `Win+R`
2. Type `shell:startup` and press Enter
3. Create a shortcut to the application in this folder

## Troubleshooting

### Search Bar Doesn't Appear

**Problem**: Pressing `Ctrl+K` doesn't show the search bar.

**Solutions**:
- Check if another application is using the same hotkey
- Try changing the hotkey in Settings
- Restart the application from the system tray
- Check if the application is running (look for the tray icon)

### File Search is Slow

**Problem**: File search takes several seconds to return results.

**Solutions**:
- Install [Everything](https://www.voidtools.com/) for instant file indexing
- Ensure Everything is running in the background
- Check Windows Search indexing settings

### Application Not Starting

**Problem**: The application doesn't start after installation.

**Solutions**:
- Check if Windows Defender or antivirus is blocking the application
- Run the installer as Administrator
- Check the log file at `%APPDATA%\better-finder\logs\app.log`

### Hotkey Conflicts

**Problem**: The global hotkey conflicts with another application.

**Solutions**:
- Open Settings and change the hotkey to a different combination
- Common alternatives: `Ctrl+Space`, `Alt+Space`, `Win+K`

### Updates Not Working

**Problem**: Auto-updates fail to download or install.

**Solutions**:
- Check your internet connection
- Ensure the application has permission to access the network
- Try manually downloading the latest version from the Releases page
- Check firewall settings

### High Memory Usage

**Problem**: The application uses more than 100MB of RAM.

**Solutions**:
- Clear the cache by restarting the application
- Reduce the number of enabled search providers in Settings
- Reduce the "Max Results" setting
- Check for memory leaks in the log file

## Development

### Project Structure

```
better-finder/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── hooks/             # React hooks
│   └── App.tsx            # Main app component
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── search/        # Search engine and providers
│   │   ├── hotkey.rs      # Global hotkey manager
│   │   ├── settings.rs    # Settings management
│   │   └── main.rs        # Entry point
│   └── Cargo.toml         # Rust dependencies
├── package.json           # Node dependencies
└── README.md             # This file
```

### Development Commands

```bash
# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build

# Run tests
npm test

# Run Rust tests
cd src-tauri && cargo test

# Format code
npm run format
cargo fmt

# Lint
npm run lint
cargo clippy
```

### Adding a New Search Provider

1. Create a new provider in `src-tauri/src/search/providers/`
2. Implement the `SearchProvider` trait
3. Register the provider in `src-tauri/src/lib.rs`

Example:
```rust
pub struct MyProvider;

#[async_trait]
impl SearchProvider for MyProvider {
    fn name(&self) -> &str { "my_provider" }
    fn priority(&self) -> u8 { 50 }
    
    async fn search(&self, query: &str) -> Vec<SearchResult> {
        // Implementation
    }
    
    async fn execute(&self, result: &SearchResult) -> Result<()> {
        // Implementation
    }
}
```

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) - Framework for building desktop applications
- [Everything SDK](https://www.voidtools.com/support/everything/sdk/) - Ultra-fast file indexing
- [Lucide](https://lucide.dev/) - Beautiful icon library
- [TailwindCSS](https://tailwindcss.com/) - Utility-first CSS framework

## Support

- 📧 Email: support@example.com
- 🐛 Issues: [GitHub Issues](https://github.com/yourusername/global-search-launcher/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/yourusername/global-search-launcher/discussions)

## Roadmap

- [ ] Plugin system for third-party extensions
- [ ] Cloud sync for settings and history
- [ ] AI-powered natural language queries
- [ ] Custom commands and scripts
- [ ] Multi-monitor support
- [ ] File preview functionality
- [ ] Search history and suggestions

---

Made with ❤️ by the Global Search Launcher team
