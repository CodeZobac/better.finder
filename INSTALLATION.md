# Installation Guide

Complete installation instructions for the Global Search Launcher.

## Table of Contents

- [System Requirements](#system-requirements)
- [Pre-Installation](#pre-installation)
- [Installation Methods](#installation-methods)
- [Post-Installation](#post-installation)
- [Optional Components](#optional-components)
- [Uninstallation](#uninstallation)

## System Requirements

### Minimum Requirements

- **Operating System**: Windows 10 (version 1809 or later)
- **Processor**: 1 GHz or faster
- **RAM**: 4 GB
- **Disk Space**: 150 MB free space
- **Display**: 1024x768 resolution or higher
- **Internet**: Required for updates and web search

### Recommended Requirements

- **Operating System**: Windows 11
- **Processor**: 2 GHz dual-core or faster
- **RAM**: 8 GB or more
- **Disk Space**: 500 MB free space (for cache and logs)
- **Display**: 1920x1080 resolution or higher
- **Internet**: Broadband connection

### Optional Components

- **Everything 1.4+**: For ultra-fast file search (highly recommended)
- **.NET Runtime**: May be required for some features (usually pre-installed)

## Pre-Installation

### 1. Check Windows Version

1. Press `Win + R`
2. Type `winver` and press Enter
3. Verify you have Windows 10 (1809+) or Windows 11

### 2. Check Available Disk Space

1. Open File Explorer
2. Right-click on your C: drive
3. Select "Properties"
4. Ensure you have at least 150 MB free space

### 3. Download the Installer

1. Visit the [Releases page](https://github.com/yourusername/global-search-launcher/releases)
2. Download the latest version:
   - **Recommended**: `better-finder_x.x.x_x64-setup.exe` (NSIS installer)
   - **Alternative**: `better-finder_x.x.x_x64_en-US.msi` (MSI installer)

### 4. Verify Download (Optional but Recommended)

1. Check the file size matches the expected size on the releases page
2. Verify the SHA256 checksum if provided

## Installation Methods

### Method 1: NSIS Installer (Recommended)

The NSIS installer provides the most user-friendly installation experience.

#### Steps:

1. **Locate the Installer**:
   - Navigate to your Downloads folder
   - Find `better-finder_x.x.x_x64-setup.exe`

2. **Run the Installer**:
   - Double-click the installer
   - If Windows SmartScreen appears:
     - Click "More info"
     - Click "Run anyway"

3. **Choose Installation Type**:
   - **For current user only** (recommended):
     - No administrator privileges required
     - Installs to `%LOCALAPPDATA%\Programs\`
   - **For all users**:
     - Requires administrator privileges
     - Installs to `C:\Program Files\`

4. **Select Installation Location** (optional):
   - Default location is recommended
   - Click "Browse" to change if needed

5. **Choose Components** (optional):
   - Desktop shortcut
   - Start Menu folder
   - Auto-start with Windows

6. **Install**:
   - Click "Install"
   - Wait for installation to complete
   - Click "Finish"

7. **First Launch**:
   - The application will start automatically
   - Look for the tray icon in the system tray
   - Press `Ctrl+K` to test the search bar

### Method 2: MSI Installer

The MSI installer is suitable for enterprise deployments and Group Policy.

#### Steps:

1. **Locate the Installer**:
   - Navigate to your Downloads folder
   - Find `better-finder_x.x.x_x64_en-US.msi`

2. **Run the Installer**:
   - Double-click the MSI file
   - Click "Next" on the welcome screen

3. **Accept License Agreement**:
   - Read the license terms
   - Check "I accept the terms"
   - Click "Next"

4. **Choose Installation Folder**:
   - Default: `C:\Program Files\Global Search Launcher\`
   - Click "Change" to modify
   - Click "Next"

5. **Ready to Install**:
   - Review your choices
   - Click "Install"
   - Provide administrator credentials if prompted

6. **Complete Installation**:
   - Wait for installation to finish
   - Click "Finish"
   - Launch the application from the Start Menu

### Method 3: Silent Installation (Enterprise)

For automated or silent installations:

#### NSIS Silent Install:
```cmd
better-finder_x.x.x_x64-setup.exe /S
```

#### MSI Silent Install:
```cmd
msiexec /i better-finder_x.x.x_x64_en-US.msi /quiet /qn
```

#### MSI Silent Install with Logging:
```cmd
msiexec /i better-finder_x.x.x_x64_en-US.msi /quiet /qn /l*v install.log
```

#### MSI Install for All Users:
```cmd
msiexec /i better-finder_x.x.x_x64_en-US.msi ALLUSERS=1 /quiet
```

## Post-Installation

### 1. Verify Installation

1. **Check System Tray**:
   - Look for the application icon in the system tray
   - If not visible, click the up arrow to show hidden icons

2. **Test Hotkey**:
   - Press `Ctrl+K`
   - The search bar should appear
   - Press `Esc` to close

3. **Check Version**:
   - Right-click the tray icon
   - Select "Settings"
   - Verify the version number at the bottom

### 2. Configure Settings

1. **Open Settings**:
   - Right-click the tray icon
   - Select "Settings"

2. **Customize Hotkey** (optional):
   - Change from `Ctrl+K` if needed
   - Recommended alternatives:
     - `Ctrl+Space`
     - `Alt+Space`
     - `Win+K`

3. **Choose Theme**:
   - Light
   - Dark
   - System (matches Windows theme)

4. **Enable Auto-Start** (recommended):
   - Check "Start with Windows"
   - Click "Save"

5. **Configure Providers**:
   - Enable/disable search providers as needed
   - Disable unused providers to improve performance

### 3. First Search

1. Press `Ctrl+K` to open the search bar
2. Try these example searches:
   - Type a filename: `document`
   - Type an app name: `notepad`
   - Type a calculation: `2+2`
   - Type a system command: `lock`

## Optional Components

### Installing Everything (Highly Recommended)

Everything provides ultra-fast file search capabilities.

#### Steps:

1. **Download Everything**:
   - Visit https://www.voidtools.com/downloads/
   - Download Everything 1.4 or later (64-bit recommended)

2. **Install Everything**:
   - Run the installer
   - Choose installation options
   - Complete installation

3. **Configure Everything**:
   - Launch Everything
   - Wait for initial indexing (usually very fast)
   - Go to Tools > Options > General
   - Enable "Start Everything on system startup"

4. **Verify Integration**:
   - Restart Global Search Launcher
   - Press `Ctrl+K`
   - Type a filename
   - Results should appear instantly

### Configuring Windows Search (Fallback)

If you don't install Everything, Windows Search will be used as a fallback.

#### Steps:

1. **Open Windows Settings**:
   - Press `Win + I`
   - Go to "Privacy & Security" > "Searching Windows"

2. **Configure Search Locations**:
   - Enable "Enhanced" search mode
   - Or add specific folders to index

3. **Wait for Indexing**:
   - Initial indexing may take several hours
   - Check indexing status in Settings

## Uninstallation

### Method 1: Windows Settings

1. **Open Settings**:
   - Press `Win + I`
   - Go to "Apps" > "Installed apps"

2. **Find Application**:
   - Search for "Global Search Launcher" or "better-finder"
   - Click the three dots (⋯)
   - Select "Uninstall"

3. **Confirm Uninstallation**:
   - Click "Uninstall" again
   - Wait for completion

4. **Remove User Data** (optional):
   - Press `Win + R`
   - Type `%APPDATA%\better-finder`
   - Delete the folder

### Method 2: Control Panel

1. **Open Control Panel**:
   - Press `Win + R`
   - Type `control`
   - Press Enter

2. **Programs and Features**:
   - Click "Programs and Features"
   - Find "Global Search Launcher"
   - Right-click and select "Uninstall"

3. **Follow Uninstaller**:
   - Complete the uninstallation wizard
   - Restart if prompted

### Method 3: Silent Uninstallation

#### NSIS Silent Uninstall:
```cmd
"%LOCALAPPDATA%\Programs\better-finder\uninstall.exe" /S
```

#### MSI Silent Uninstall:
```cmd
msiexec /x {PRODUCT-CODE} /quiet
```

To find the product code:
```cmd
wmic product where name="Global Search Launcher" get IdentifyingNumber
```

## Troubleshooting Installation

### Installation Fails

**Error: "Installation failed" or "Access denied"**

Solutions:
1. Run installer as Administrator
2. Disable antivirus temporarily
3. Check disk space
4. Try the alternative installer (MSI vs NSIS)

### Application Won't Start

**Error: Application doesn't launch after installation**

Solutions:
1. Check Task Manager for running process
2. Look for tray icon
3. Check logs at `%APPDATA%\better-finder\logs\`
4. Reinstall the application

### SmartScreen Warning

**Error: "Windows protected your PC"**

Solutions:
1. Click "More info"
2. Click "Run anyway"
3. Or right-click installer > Properties > Unblock

### Antivirus Blocking

**Error: Antivirus quarantines or blocks the installer**

Solutions:
1. Add exception for the installer
2. Add exception for installation directory
3. Temporarily disable antivirus during installation
4. Download from official source only

## Upgrading

### Automatic Updates

The application checks for updates automatically:
1. Update notification appears when available
2. Update downloads in background
3. Notification shows when ready to install
4. Restart application to apply update

### Manual Updates

To manually update:
1. Download the latest installer
2. Run the installer
3. It will detect and upgrade the existing installation
4. No need to uninstall first

### Preserving Settings

Settings are preserved during updates:
- Stored in `%APPDATA%\better-finder\settings.json`
- Automatically migrated to new versions
- Backup recommended before major version updates

## Enterprise Deployment

### Group Policy Deployment

1. **Create MSI Package**:
   - Use the MSI installer
   - Test on a reference machine

2. **Create GPO**:
   - Open Group Policy Management
   - Create new GPO
   - Edit GPO

3. **Add Software Package**:
   - Computer Configuration > Policies > Software Settings
   - Right-click "Software installation"
   - New > Package
   - Browse to MSI file on network share

4. **Configure Deployment**:
   - Choose "Assigned" or "Published"
   - Configure options as needed
   - Link GPO to appropriate OU

### SCCM/Intune Deployment

1. **Prepare Package**:
   - Copy MSI to package source
   - Create detection method
   - Define install/uninstall commands

2. **Create Application**:
   - Add MSI as deployment type
   - Configure requirements
   - Set detection rules

3. **Deploy**:
   - Deploy to device collection
   - Choose deployment type (Available/Required)
   - Set schedule

### Configuration Management

Default settings can be pre-configured:

1. Create `settings.json` in `%APPDATA%\better-finder\`
2. Deploy via GPO or configuration management
3. Example:
```json
{
  "hotkey": "Ctrl+K",
  "theme": "system",
  "max_results": 8,
  "start_with_windows": true,
  "enabled_providers": {
    "files": true,
    "applications": true,
    "quick_actions": true,
    "calculator": true,
    "clipboard": false,
    "bookmarks": true,
    "recent_files": true
  }
}
```

## Support

For installation issues:
- Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
- Visit [GitHub Issues](https://github.com/yourusername/global-search-launcher/issues)
- Email: support@example.com

---

Last updated: January 2025
