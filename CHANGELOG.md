# Changelog

All notable changes to the Global Search Launcher will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned Features
- Plugin system for third-party extensions
- Cloud sync for settings and history
- AI-powered natural language queries
- Custom commands and scripts
- Multi-monitor support
- File preview functionality
- Search history and suggestions

## [0.1.0] - 2025-01-15

### Added
- Initial release of Global Search Launcher
- Global hotkey activation (Ctrl+K by default)
- File search using Everything SDK with Windows Search fallback
- Application search and launcher
- Browser bookmarks search (Chrome, Edge, Firefox)
- Clipboard history tracking and search
- Built-in calculator for mathematical expressions
- Quick actions for system commands (shutdown, restart, lock, etc.)
- Web search fallback for unmatched queries
- Recent files tracking
- Keyboard-first navigation (arrow keys, Enter, Escape)
- System tray integration
- Settings panel with customization options
- Theme support (light, dark, system)
- Auto-start with Windows
- Auto-update functionality
- Fuzzy search matching
- Result grouping by type
- Icon caching for performance
- Comprehensive error handling and logging
- NSIS and MSI installers
- Complete documentation (README, INSTALLATION, TROUBLESHOOTING)

### Features by Category

#### Search Providers
- **FileSearchProvider**: Ultra-fast file search via Everything SDK
- **WindowsSearchProvider**: Fallback file search using Windows Search API
- **AppSearchProvider**: Search and launch installed applications
- **BookmarkProvider**: Search browser bookmarks from multiple browsers
- **ClipboardHistoryProvider**: Track and search clipboard history (last 20 items)
- **CalculatorProvider**: Evaluate mathematical expressions
- **QuickActionProvider**: Execute system commands
- **RecentFilesProvider**: Quick access to recently opened files
- **WebSearchProvider**: Fallback to Google search for unmatched queries

#### User Interface
- Frameless, transparent search window
- Smooth show/hide animations
- Result highlighting for matched characters
- Visual grouping by result type
- Hover and keyboard selection states
- Toast notifications for errors and confirmations
- Settings panel with live preview
- Update notifications

#### Performance
- Sub-50ms search response time
- Sub-100ms UI render time
- <100MB RAM usage while idle
- <2s startup time
- LRU caching for icons and results
- Parallel search execution across providers
- Virtual scrolling for large result sets

#### System Integration
- Global hotkey registration
- System tray with menu
- Windows registry integration for auto-start
- Windows theme detection
- Clipboard monitoring
- Browser profile detection
- Everything SDK integration

### Technical Details
- Built with Tauri 2.x (Rust + React)
- React 18 with TypeScript
- TailwindCSS for styling
- Tokio for async runtime
- Tracing for logging
- SQLite for data persistence
- Everything SDK for file indexing
- Windows API integration

### Known Issues
- Clipboard history only supports text content (images planned for future release)
- Everything SDK must be installed separately for optimal file search
- Some antivirus software may flag the installer (false positive)
- Hotkey conflicts with other applications require manual resolution

### Installation
- Windows 10 (1809+) or Windows 11 required
- NSIS installer: `better-finder_0.1.0_x64-setup.exe`
- MSI installer: `better-finder_0.1.0_x64_en-US.msi`
- Silent install support for enterprise deployment

### Documentation
- Comprehensive README with usage instructions
- Detailed INSTALLATION guide
- TROUBLESHOOTING guide for common issues
- Inline code documentation
- API documentation for developers

---

## Version History

### Version Numbering

We use Semantic Versioning (MAJOR.MINOR.PATCH):
- **MAJOR**: Incompatible API changes or major feature overhauls
- **MINOR**: New features in a backwards-compatible manner
- **PATCH**: Backwards-compatible bug fixes

### Release Types

- **Stable**: Production-ready releases (e.g., 1.0.0)
- **Beta**: Feature-complete but may have bugs (e.g., 1.0.0-beta.1)
- **Alpha**: Early testing releases (e.g., 1.0.0-alpha.1)
- **RC**: Release candidates (e.g., 1.0.0-rc.1)

### Support Policy

- **Latest Major Version**: Full support with new features and bug fixes
- **Previous Major Version**: Security updates and critical bug fixes for 6 months
- **Older Versions**: No support (upgrade recommended)

---

## How to Update

### Automatic Updates
The application checks for updates automatically on startup and notifies you when an update is available. Updates are downloaded and installed in the background.

### Manual Updates
1. Download the latest installer from the [Releases page](https://github.com/yourusername/global-search-launcher/releases)
2. Run the installer (it will detect and upgrade the existing installation)
3. Restart the application

### Checking Your Version
1. Right-click the system tray icon
2. Select "Settings"
3. Version number is displayed at the bottom of the settings panel

---

## Migration Guides

### Migrating from 0.x to 1.0 (Future)
When version 1.0 is released, migration instructions will be provided here.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for information on how to contribute to this project.

---

## Links

- [GitHub Repository](https://github.com/yourusername/global-search-launcher)
- [Issue Tracker](https://github.com/yourusername/global-search-launcher/issues)
- [Discussions](https://github.com/yourusername/global-search-launcher/discussions)
- [Releases](https://github.com/yourusername/global-search-launcher/releases)

---

[Unreleased]: https://github.com/yourusername/global-search-launcher/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/global-search-launcher/releases/tag/v0.1.0
