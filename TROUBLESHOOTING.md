# Troubleshooting Guide

This guide covers common issues and their solutions for the Global Search Launcher.

## Table of Contents

- [Installation Issues](#installation-issues)
- [Hotkey Issues](#hotkey-issues)
- [Search Issues](#search-issues)
- [Performance Issues](#performance-issues)
- [Update Issues](#update-issues)
- [System Integration Issues](#system-integration-issues)
- [Logging and Diagnostics](#logging-and-diagnostics)

## Installation Issues

### Installer Won't Run

**Symptoms**: Double-clicking the installer does nothing or shows an error.

**Possible Causes**:
- Windows SmartScreen blocking the installer
- Antivirus software blocking the installer
- Corrupted download

**Solutions**:

1. **Bypass SmartScreen**:
   - Right-click the installer
   - Select "Properties"
   - Check "Unblock" at the bottom
   - Click "Apply" and "OK"
   - Run the installer again

2. **Run as Administrator**:
   - Right-click the installer
   - Select "Run as administrator"

3. **Check Antivirus**:
   - Temporarily disable antivirus
   - Run the installer
   - Re-enable antivirus
   - Add the application to antivirus exceptions

4. **Re-download**:
   - Delete the installer
   - Download again from the official source
   - Verify the file size matches the expected size

### Installation Fails Midway

**Symptoms**: Installation starts but fails with an error message.

**Possible Causes**:
- Insufficient disk space
- Insufficient permissions
- Conflicting software

**Solutions**:

1. **Check Disk Space**:
   - Ensure you have at least 200MB free space
   - Clean up temporary files if needed

2. **Install for Current User**:
   - Choose "Install for current user only" option
   - This requires fewer permissions

3. **Close Conflicting Software**:
   - Close any running instances of the application
   - Close other launcher or search applications
   - Restart your computer and try again

### Application Won't Start After Installation

**Symptoms**: Installation completes but the application doesn't launch.

**Solutions**:

1. **Check Task Manager**:
   - Open Task Manager (Ctrl+Shift+Esc)
   - Look for "better-finder.exe" in the Processes tab
   - If found, end the process and try launching again

2. **Check System Tray**:
   - Look for the application icon in the system tray
   - The application runs in the background

3. **Check Logs**:
   - Navigate to `%APPDATA%\better-finder\logs\`
   - Open the latest `app.log` file
   - Look for error messages

4. **Reinstall**:
   - Uninstall the application
   - Restart your computer
   - Install again

## Hotkey Issues

### Hotkey Doesn't Work

**Symptoms**: Pressing Ctrl+K (or your configured hotkey) doesn't show the search bar.

**Possible Causes**:
- Hotkey conflict with another application
- Application not running
- Hotkey registration failed

**Solutions**:

1. **Verify Application is Running**:
   - Check for the tray icon in the system tray
   - If not present, launch the application

2. **Check for Conflicts**:
   - Common conflicting applications:
     - Visual Studio Code (Ctrl+K)
     - Slack (Ctrl+K)
     - Discord (Ctrl+K)
     - Other launcher applications
   - Close these applications temporarily to test
   - Or change the hotkey in Settings

3. **Change Hotkey**:
   - Right-click the tray icon
   - Select "Settings"
   - Change the hotkey to an alternative:
     - `Ctrl+Space`
     - `Alt+Space`
     - `Win+K`
     - `Ctrl+Shift+K`

4. **Restart Application**:
   - Right-click the tray icon
   - Select "Exit"
   - Launch the application again

### Hotkey Works Intermittently

**Symptoms**: Hotkey sometimes works, sometimes doesn't.

**Possible Causes**:
- Focus issues with certain applications
- System resource constraints
- Background process interference

**Solutions**:

1. **Test with Different Applications**:
   - Note which applications cause the issue
   - Report these as bugs

2. **Check System Resources**:
   - Open Task Manager
   - Check CPU and memory usage
   - Close unnecessary applications

3. **Update Windows**:
   - Ensure Windows is up to date
   - Install all pending updates

## Search Issues

### No Search Results

**Symptoms**: Typing in the search bar returns no results.

**Possible Causes**:
- Search providers disabled
- Everything SDK not running (for file search)
- Database corruption

**Solutions**:

1. **Check Enabled Providers**:
   - Open Settings
   - Ensure desired search providers are enabled

2. **Install Everything** (for file search):
   - Download from https://www.voidtools.com/
   - Install and run Everything
   - Restart Global Search Launcher

3. **Clear Cache**:
   - Close the application
   - Navigate to `%APPDATA%\better-finder\`
   - Delete the `cache` folder
   - Restart the application

4. **Reset Settings**:
   - Navigate to `%APPDATA%\better-finder\`
   - Rename `settings.json` to `settings.json.backup`
   - Restart the application (will use default settings)

### File Search is Slow

**Symptoms**: File search takes several seconds to return results.

**Possible Causes**:
- Everything SDK not installed
- Using Windows Search fallback
- Large number of files

**Solutions**:

1. **Install Everything**:
   - Download from https://www.voidtools.com/
   - Install and run Everything
   - Restart Global Search Launcher
   - File search should now be instant

2. **Configure Everything**:
   - Open Everything
   - Go to Tools > Options > Indexes
   - Ensure all drives are indexed
   - Wait for indexing to complete

3. **Reduce Search Scope**:
   - In Settings, reduce "Max Results"
   - This limits the number of results processed

### Incorrect or Irrelevant Results

**Symptoms**: Search returns results that don't match the query.

**Possible Causes**:
- Fuzzy matching too aggressive
- Provider priority issues
- Outdated cache

**Solutions**:

1. **Use More Specific Queries**:
   - Type more characters
   - Use exact file extensions (e.g., `.pdf`)

2. **Clear Cache**:
   - Close the application
   - Navigate to `%APPDATA%\better-finder\`
   - Delete the `cache` folder
   - Restart the application

3. **Report Issue**:
   - Note the query and unexpected results
   - Report as a bug with details

## Performance Issues

### High CPU Usage

**Symptoms**: Application uses excessive CPU resources.

**Possible Causes**:
- Background indexing
- Memory leak
- Inefficient search provider

**Solutions**:

1. **Wait for Indexing**:
   - Initial indexing may take time
   - CPU usage should decrease after completion

2. **Restart Application**:
   - Right-click tray icon
   - Select "Exit"
   - Launch again

3. **Disable Providers**:
   - Open Settings
   - Disable providers you don't use
   - Especially clipboard history and bookmarks

4. **Check Logs**:
   - Navigate to `%APPDATA%\better-finder\logs\`
   - Look for repeated errors or warnings

### High Memory Usage

**Symptoms**: Application uses more than 100MB of RAM.

**Possible Causes**:
- Large cache
- Memory leak
- Too many cached icons

**Solutions**:

1. **Clear Cache**:
   - Close the application
   - Navigate to `%APPDATA%\better-finder\`
   - Delete the `cache` folder
   - Restart the application

2. **Reduce Max Results**:
   - Open Settings
   - Reduce "Max Results" to 5 or fewer

3. **Disable Unused Providers**:
   - Open Settings
   - Disable providers you don't use

4. **Restart Application**:
   - Right-click tray icon
   - Select "Exit"
   - Launch again

### Slow Startup

**Symptoms**: Application takes more than 5 seconds to start.

**Possible Causes**:
- Too many providers enabled
- Large database
- System resource constraints

**Solutions**:

1. **Disable Unused Providers**:
   - Open Settings
   - Disable providers you don't use

2. **Clear Old Data**:
   - Navigate to `%APPDATA%\better-finder\`
   - Delete old log files
   - Delete the `cache` folder

3. **Check System Resources**:
   - Ensure sufficient RAM available
   - Close unnecessary startup applications

## Update Issues

### Updates Won't Download

**Symptoms**: Update notification appears but download fails.

**Possible Causes**:
- Network connectivity issues
- Firewall blocking
- Update server unavailable

**Solutions**:

1. **Check Internet Connection**:
   - Verify you can access other websites
   - Try again later

2. **Check Firewall**:
   - Add exception for better-finder.exe
   - Allow outbound connections

3. **Manual Update**:
   - Visit the Releases page
   - Download the latest installer
   - Run the installer to update

### Update Installed But Not Applied

**Symptoms**: Update notification says "installed" but version doesn't change.

**Solutions**:

1. **Restart Application**:
   - Right-click tray icon
   - Select "Exit"
   - Launch the application again

2. **Restart Computer**:
   - Some updates require a system restart

3. **Verify Version**:
   - Open Settings
   - Check the version number at the bottom
   - Compare with the latest release

## System Integration Issues

### Auto-Start Not Working

**Symptoms**: Application doesn't start with Windows despite being enabled.

**Possible Causes**:
- Registry entry missing or incorrect
- Windows startup settings
- Antivirus blocking

**Solutions**:

1. **Re-enable Auto-Start**:
   - Open Settings
   - Disable "Start with Windows"
   - Click "Save"
   - Enable "Start with Windows" again
   - Click "Save"

2. **Check Task Manager**:
   - Open Task Manager
   - Go to "Startup" tab
   - Find "Global Search Launcher"
   - Ensure it's "Enabled"

3. **Manual Startup Entry**:
   - Press Win+R
   - Type `shell:startup`
   - Create a shortcut to the application

### System Tray Icon Missing

**Symptoms**: Application is running but tray icon doesn't appear.

**Possible Causes**:
- Tray icon hidden by Windows
- Display driver issues
- Application crash

**Solutions**:

1. **Show Hidden Icons**:
   - Click the up arrow in the system tray
   - Look for the application icon
   - Drag it to the main tray area

2. **Restart Explorer**:
   - Open Task Manager
   - Find "Windows Explorer"
   - Right-click and select "Restart"

3. **Restart Application**:
   - Open Task Manager
   - End "better-finder.exe" process
   - Launch the application again

## Logging and Diagnostics

### Accessing Logs

Logs are stored at: `%APPDATA%\better-finder\logs\app.log`

To access:
1. Press Win+R
2. Type `%APPDATA%\better-finder\logs\`
3. Press Enter
4. Open the latest `app.log` file

### Understanding Log Levels

- **ERROR**: Critical issues that prevent functionality
- **WARN**: Non-critical issues or degraded functionality
- **INFO**: Normal operational messages
- **DEBUG**: Detailed diagnostic information

### Enabling Debug Logging

1. Navigate to `%APPDATA%\better-finder\`
2. Create or edit `config.toml`
3. Add:
   ```toml
   [logging]
   level = "debug"
   ```
4. Restart the application

### Collecting Diagnostic Information

When reporting issues, include:

1. **System Information**:
   - Windows version
   - RAM amount
   - Disk space available

2. **Application Information**:
   - Application version (from Settings)
   - Enabled providers
   - Custom hotkey (if any)

3. **Log Files**:
   - Latest `app.log` file
   - Any error messages

4. **Steps to Reproduce**:
   - What you were doing
   - What you expected
   - What actually happened

### Resetting to Defaults

To completely reset the application:

1. **Uninstall**:
   - Use Windows Settings > Apps
   - Uninstall "Global Search Launcher"

2. **Delete Data**:
   - Navigate to `%APPDATA%\better-finder\`
   - Delete the entire folder

3. **Reinstall**:
   - Download the latest installer
   - Install the application

## Getting Help

If you can't resolve your issue:

1. **Check GitHub Issues**:
   - Search existing issues
   - Your problem may already be reported

2. **Create a New Issue**:
   - Include diagnostic information
   - Attach relevant log files
   - Describe steps to reproduce

3. **Community Support**:
   - Join GitHub Discussions
   - Ask the community for help

4. **Contact Support**:
   - Email: support@example.com
   - Include all diagnostic information

---

Last updated: January 2025
