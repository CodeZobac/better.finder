# Testing Guide

Comprehensive testing procedures for the Global Search Launcher installer and application.

## Table of Contents

- [Test Environment Setup](#test-environment-setup)
- [Installer Testing](#installer-testing)
- [Functionality Testing](#functionality-testing)
- [Performance Testing](#performance-testing)
- [Update Testing](#update-testing)
- [Regression Testing](#regression-testing)
- [Test Reporting](#test-reporting)

## Test Environment Setup

### Required Test Environments

1. **Windows 10 (Clean Install)**
   - Version 1809 or later
   - No previous installation
   - Standard user account
   - Administrator account

2. **Windows 11 (Clean Install)**
   - Latest version
   - No previous installation
   - Standard user account
   - Administrator account

3. **Windows 10 (Upgrade Scenario)**
   - Previous version installed
   - Test upgrade path

4. **Windows 11 (Upgrade Scenario)**
   - Previous version installed
   - Test upgrade path

### Test Machine Specifications

**Minimum Spec Machine**:
- CPU: 1 GHz single-core
- RAM: 4 GB
- Disk: 100 GB HDD
- Display: 1024x768

**Recommended Spec Machine**:
- CPU: 2 GHz dual-core or better
- RAM: 8 GB or more
- Disk: 256 GB SSD
- Display: 1920x1080 or higher

### Prerequisites

- [ ] Clean Windows installation or VM snapshot
- [ ] Internet connection for update testing
- [ ] Administrator access
- [ ] Standard user account for permission testing
- [ ] Test data (sample files, applications)
- [ ] Everything SDK installer (for optional component testing)

## Installer Testing

### Test Case 1: NSIS Installer - Clean Install (Standard User)

**Objective**: Verify NSIS installer works for standard users without admin rights.

**Steps**:
1. Log in as standard user (non-administrator)
2. Download `better-finder_x.x.x_x64-setup.exe`
3. Double-click the installer
4. If SmartScreen appears, click "More info" → "Run anyway"
5. Select "Install for current user only"
6. Accept default installation location
7. Select all optional components (desktop shortcut, auto-start)
8. Click "Install"
9. Wait for installation to complete
10. Click "Finish"

**Expected Results**:
- [ ] Installer runs without requesting admin privileges
- [ ] Installation completes successfully
- [ ] Application starts automatically after installation
- [ ] System tray icon appears
- [ ] Desktop shortcut created
- [ ] Start menu entry created
- [ ] Pressing Ctrl+K shows search bar
- [ ] Installation directory: `%LOCALAPPDATA%\Programs\better-finder\`

**Test Data**: N/A

**Pass/Fail**: ___________

**Notes**: ___________________________________________

---

### Test Case 2: NSIS Installer - Clean Install (Administrator)

**Objective**: Verify NSIS installer works for all users when run as administrator.

**Steps**:
1. Log in as administrator
2. Download `better-finder_x.x.x_x64-setup.exe`
3. Right-click installer → "Run as administrator"
4. Select "Install for all users"
5. Accept default installation location
6. Select all optional components
7. Click "Install"
8. Wait for installation to complete
9. Click "Finish"
10. Log out and log in as different user
11. Test application functionality

**Expected Results**:
- [ ] UAC prompt appears
- [ ] Installation completes successfully
- [ ] Application available for all users
- [ ] System tray icon appears for all users
- [ ] Installation directory: `C:\Program Files\Global Search Launcher\`

**Pass/Fail**: ___________

---

### Test Case 3: MSI Installer - Clean Install

**Objective**: Verify MSI installer works correctly.

**Steps**:
1. Log in as administrator
2. Download `better-finder_x.x.x_x64_en-US.msi`
3. Double-click the MSI file
4. Click "Next" on welcome screen
5. Accept license agreement
6. Accept default installation folder
7. Click "Install"
8. Provide admin credentials if prompted
9. Wait for installation to complete
10. Click "Finish"
11. Launch application from Start Menu

**Expected Results**:
- [ ] MSI installer wizard appears
- [ ] Installation completes successfully
- [ ] Application launches successfully
- [ ] System tray icon appears
- [ ] Installation directory: `C:\Program Files\Global Search Launcher\`

**Pass/Fail**: ___________

---

### Test Case 4: Silent Installation (NSIS)

**Objective**: Verify silent installation works for automated deployment.

**Steps**:
1. Open Command Prompt as administrator
2. Navigate to installer directory
3. Run: `better-finder_x.x.x_x64-setup.exe /S`
4. Wait 30 seconds
5. Check Task Manager for running process
6. Check installation directory
7. Test application functionality

**Expected Results**:
- [ ] No UI appears during installation
- [ ] Installation completes silently
- [ ] Application installed correctly
- [ ] Application can be launched
- [ ] Exit code 0 (success)

**Pass/Fail**: ___________

---

### Test Case 5: Silent Installation (MSI)

**Objective**: Verify MSI silent installation works.

**Steps**:
1. Open Command Prompt as administrator
2. Navigate to installer directory
3. Run: `msiexec /i better-finder_x.x.x_x64_en-US.msi /quiet /qn /l*v install.log`
4. Wait for completion
5. Check install.log for errors
6. Verify installation
7. Test application functionality

**Expected Results**:
- [ ] No UI appears during installation
- [ ] Installation completes silently
- [ ] Log file created with no errors
- [ ] Application installed correctly
- [ ] Exit code 0 (success)

**Pass/Fail**: ___________

---

### Test Case 6: Installation with Insufficient Disk Space

**Objective**: Verify installer handles insufficient disk space gracefully.

**Steps**:
1. Fill disk to leave <50 MB free space
2. Run installer
3. Attempt installation

**Expected Results**:
- [ ] Installer detects insufficient space
- [ ] Clear error message displayed
- [ ] Installation does not proceed
- [ ] No partial installation left behind

**Pass/Fail**: ___________

---

### Test Case 7: Installation Path with Special Characters

**Objective**: Verify installer handles custom paths with special characters.

**Steps**:
1. Run NSIS installer
2. Choose custom installation path: `C:\Test Folder (2024)\App's Dir\`
3. Complete installation
4. Test application functionality

**Expected Results**:
- [ ] Installer accepts custom path
- [ ] Installation completes successfully
- [ ] Application runs correctly from custom path
- [ ] All features work as expected

**Pass/Fail**: ___________

---

### Test Case 8: Uninstallation (NSIS)

**Objective**: Verify complete uninstallation.

**Steps**:
1. Install application using NSIS installer
2. Use application (create some data)
3. Open Windows Settings → Apps
4. Find "Global Search Launcher"
5. Click "Uninstall"
6. Confirm uninstallation
7. Check installation directory
8. Check `%APPDATA%\better-finder\`
9. Check registry for leftover entries
10. Check Start Menu
11. Check Desktop for shortcuts

**Expected Results**:
- [ ] Uninstaller runs successfully
- [ ] Installation directory removed
- [ ] User data preserved in AppData (optional removal)
- [ ] Registry entries cleaned up
- [ ] Start Menu entries removed
- [ ] Desktop shortcuts removed
- [ ] System tray icon removed

**Pass/Fail**: ___________

---

### Test Case 9: Uninstallation (MSI)

**Objective**: Verify MSI uninstallation.

**Steps**:
1. Install application using MSI installer
2. Use application (create some data)
3. Open Control Panel → Programs and Features
4. Find "Global Search Launcher"
5. Right-click → Uninstall
6. Complete uninstallation wizard
7. Verify removal

**Expected Results**:
- [ ] Uninstaller runs successfully
- [ ] All files removed
- [ ] Registry cleaned up
- [ ] No leftover entries in Programs and Features

**Pass/Fail**: ___________

---

### Test Case 10: Upgrade Installation

**Objective**: Verify upgrade from previous version preserves settings.

**Steps**:
1. Install version 0.1.0
2. Configure custom settings:
   - Change hotkey to Ctrl+Space
   - Change theme to Dark
   - Disable some providers
   - Enable auto-start
3. Create some usage data (recent files, clipboard history)
4. Run installer for version 0.2.0
5. Complete installation
6. Launch application
7. Verify settings and data

**Expected Results**:
- [ ] Installer detects previous version
- [ ] Upgrade completes successfully
- [ ] Custom settings preserved
- [ ] User data preserved (recent files, clipboard history)
- [ ] Application version updated
- [ ] All features work correctly

**Pass/Fail**: ___________

---

## Functionality Testing

### Test Case 11: Auto-Start Functionality

**Objective**: Verify auto-start with Windows works correctly.

**Steps**:
1. Install application
2. Open Settings
3. Enable "Start with Windows"
4. Click "Save"
5. Restart computer
6. Log in
7. Wait 10 seconds
8. Check system tray for icon
9. Press Ctrl+K to test

**Expected Results**:
- [ ] Application starts automatically after login
- [ ] System tray icon appears within 5 seconds
- [ ] Hotkey works immediately
- [ ] No error messages or crashes
- [ ] Registry entry created in: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`

**Pass/Fail**: ___________

---

### Test Case 12: Auto-Start Disable

**Objective**: Verify disabling auto-start works.

**Steps**:
1. Enable auto-start (from Test Case 11)
2. Open Settings
3. Disable "Start with Windows"
4. Click "Save"
5. Restart computer
6. Log in
7. Wait 30 seconds
8. Check system tray

**Expected Results**:
- [ ] Application does not start automatically
- [ ] No system tray icon
- [ ] Registry entry removed
- [ ] Can manually launch application

**Pass/Fail**: ___________

---

### Test Case 13: Hotkey Registration

**Objective**: Verify global hotkey works from any application.

**Steps**:
1. Launch application
2. Open various applications:
   - Notepad
   - File Explorer
   - Web browser
   - Command Prompt
3. From each application, press Ctrl+K
4. Verify search bar appears
5. Press Esc to close
6. Repeat for each application

**Expected Results**:
- [ ] Hotkey works from all applications
- [ ] Search bar appears centered on screen
- [ ] Search bar has focus
- [ ] Input field is ready for typing
- [ ] Esc closes the search bar

**Pass/Fail**: ___________

---

### Test Case 14: Hotkey Customization

**Objective**: Verify custom hotkey configuration works.

**Steps**:
1. Open Settings
2. Change hotkey to "Ctrl+Space"
3. Click "Save"
4. Test old hotkey (Ctrl+K) - should not work
5. Test new hotkey (Ctrl+Space) - should work
6. Change to "Alt+Space"
7. Test again
8. Change back to "Ctrl+K"

**Expected Results**:
- [ ] Old hotkey stops working after change
- [ ] New hotkey works immediately
- [ ] No application restart required
- [ ] Settings persist after restart
- [ ] Invalid hotkeys rejected with error message

**Pass/Fail**: ___________

---

### Test Case 15: File Search (Everything SDK)

**Objective**: Verify file search works with Everything SDK.

**Prerequisites**: Everything SDK installed and running

**Steps**:
1. Ensure Everything is running
2. Press Ctrl+K
3. Type: "document"
4. Observe results
5. Measure response time
6. Select a file result
7. Press Enter

**Expected Results**:
- [ ] Results appear within 50ms
- [ ] File results displayed with icons
- [ ] File paths shown
- [ ] Fuzzy matching works (e.g., "dcmnt" finds "document")
- [ ] Pressing Enter opens the file
- [ ] File opens in default application

**Pass/Fail**: ___________

**Response Time**: _____ ms

---

### Test Case 16: File Search (Windows Search Fallback)

**Objective**: Verify file search works without Everything SDK.

**Prerequisites**: Everything SDK NOT installed

**Steps**:
1. Ensure Everything is not running
2. Press Ctrl+K
3. Type: "document"
4. Observe results
5. Wait for results
6. Select a file result
7. Press Enter

**Expected Results**:
- [ ] Warning notification about Everything not available
- [ ] Results appear (may be slower)
- [ ] File results displayed
- [ ] Pressing Enter opens the file
- [ ] Fallback to Windows Search works

**Pass/Fail**: ___________

---

### Test Case 17: Application Search

**Objective**: Verify application search and launch works.

**Steps**:
1. Press Ctrl+K
2. Type: "notepad"
3. Observe results
4. Select Notepad
5. Press Enter
6. Verify Notepad launches
7. Close Notepad
8. Test with other apps: "chrome", "calc", "paint"

**Expected Results**:
- [ ] Application results appear
- [ ] Application icons displayed
- [ ] Fuzzy matching works
- [ ] Pressing Enter launches application
- [ ] Application launches successfully
- [ ] Search bar closes after launch

**Pass/Fail**: ___________

---

### Test Case 18: Calculator

**Objective**: Verify calculator functionality.

**Steps**:
1. Press Ctrl+K
2. Type: "2+2"
3. Observe result
4. Press Enter
5. Check clipboard
6. Test other expressions:
   - "15*8"
   - "100/4"
   - "(10+5)*2"
   - "2^8"

**Expected Results**:
- [ ] Calculator result appears immediately
- [ ] Result is correct
- [ ] Pressing Enter copies result to clipboard
- [ ] Toast notification confirms copy
- [ ] All mathematical operations work
- [ ] Parentheses handled correctly

**Pass/Fail**: ___________

---

### Test Case 19: Quick Actions

**Objective**: Verify system quick actions work.

**Steps**:
1. Press Ctrl+K
2. Type: "lock"
3. Observe result
4. Note: Don't press Enter (will lock system)
5. Test search for other actions:
   - "shutdown"
   - "restart"
   - "sleep"
   - "hibernate"

**Expected Results**:
- [ ] Quick action results appear
- [ ] Action icons displayed
- [ ] Fuzzy matching works
- [ ] All system actions available
- [ ] Descriptions clear

**Pass/Fail**: ___________

**Note**: Actual execution testing should be done carefully to avoid system shutdown.

---

### Test Case 20: Web Search Fallback

**Objective**: Verify web search fallback works.

**Steps**:
1. Press Ctrl+K
2. Type: "how to use keyboard shortcuts"
3. Press Enter
4. Observe behavior
5. Test with other queries:
   - "weather today"
   - "what is the capital of France"

**Expected Results**:
- [ ] No local results found
- [ ] Web search result appears
- [ ] Pressing Enter opens browser
- [ ] Google search page opens with query
- [ ] Query properly URL-encoded
- [ ] Default browser detected correctly

**Pass/Fail**: ___________

---

## Performance Testing

### Test Case 21: Search Response Time

**Objective**: Verify search meets performance targets.

**Steps**:
1. Press Ctrl+K
2. Type a query
3. Measure time until results appear
4. Repeat 10 times with different queries
5. Calculate average

**Expected Results**:
- [ ] Average response time < 50ms
- [ ] 95th percentile < 100ms
- [ ] No queries exceed 200ms

**Measurements**:
1. _____ ms
2. _____ ms
3. _____ ms
4. _____ ms
5. _____ ms
6. _____ ms
7. _____ ms
8. _____ ms
9. _____ ms
10. _____ ms

**Average**: _____ ms

**Pass/Fail**: ___________

---

### Test Case 22: UI Render Time

**Objective**: Verify UI renders quickly.

**Steps**:
1. Press Ctrl+K
2. Measure time until window fully visible
3. Repeat 10 times
4. Calculate average

**Expected Results**:
- [ ] Average render time < 100ms
- [ ] Window appears smoothly
- [ ] No flickering or artifacts

**Average**: _____ ms

**Pass/Fail**: ___________

---

### Test Case 23: Memory Usage

**Objective**: Verify memory usage is within limits.

**Steps**:
1. Launch application
2. Wait 1 minute for initialization
3. Open Task Manager
4. Note memory usage (idle)
5. Perform 20 searches
6. Note memory usage (active)
7. Wait 5 minutes
8. Note memory usage (idle again)

**Expected Results**:
- [ ] Idle memory < 100 MB
- [ ] Active memory < 150 MB
- [ ] No memory leaks (returns to baseline)

**Measurements**:
- Idle (initial): _____ MB
- Active: _____ MB
- Idle (after use): _____ MB

**Pass/Fail**: ___________

---

### Test Case 24: Startup Time

**Objective**: Verify application starts quickly.

**Steps**:
1. Close application
2. Launch application
3. Measure time until tray icon appears
4. Measure time until hotkey works
5. Repeat 5 times
6. Calculate average

**Expected Results**:
- [ ] Tray icon appears < 2 seconds
- [ ] Hotkey works < 3 seconds
- [ ] No errors during startup

**Average Startup Time**: _____ seconds

**Pass/Fail**: ___________

---

## Update Testing

### Test Case 25: Update Check

**Objective**: Verify update check works.

**Steps**:
1. Launch application
2. Wait 10 seconds (auto-check delay)
3. Check logs for update check
4. Manually trigger update check from Settings
5. Observe behavior

**Expected Results**:
- [ ] Auto-check runs after 5 seconds
- [ ] Manual check works from Settings
- [ ] No errors in logs
- [ ] Appropriate message if no updates available

**Pass/Fail**: ___________

---

### Test Case 26: Update Download and Install

**Objective**: Verify update process works end-to-end.

**Prerequisites**: Update available on server

**Steps**:
1. Launch application (old version)
2. Wait for update notification
3. Observe download progress
4. Wait for "Update installed" notification
5. Restart application
6. Verify new version

**Expected Results**:
- [ ] Update notification appears
- [ ] Download completes successfully
- [ ] Install notification appears
- [ ] After restart, new version active
- [ ] Settings preserved
- [ ] User data preserved

**Pass/Fail**: ___________

---

### Test Case 27: Update Failure Handling

**Objective**: Verify graceful handling of update failures.

**Prerequisites**: Simulate network failure or invalid update

**Steps**:
1. Disconnect network during update download
2. Observe behavior
3. Reconnect network
4. Retry update

**Expected Results**:
- [ ] Error notification appears
- [ ] Application continues to work
- [ ] Can retry update
- [ ] No corruption or crashes

**Pass/Fail**: ___________

---

## Regression Testing

### Test Case 28: Settings Persistence

**Objective**: Verify settings persist across restarts.

**Steps**:
1. Open Settings
2. Change all settings:
   - Hotkey
   - Theme
   - Max results
   - Enabled providers
   - Auto-start
3. Click "Save"
4. Close application
5. Restart application
6. Open Settings
7. Verify all settings preserved

**Expected Results**:
- [ ] All settings saved correctly
- [ ] Settings persist after restart
- [ ] Settings file created in AppData

**Pass/Fail**: ___________

---

### Test Case 29: Clipboard History Persistence

**Objective**: Verify clipboard history persists.

**Steps**:
1. Copy 5 different text items
2. Search clipboard history
3. Verify all items present
4. Close application
5. Restart application
6. Search clipboard history again

**Expected Results**:
- [ ] Clipboard items tracked
- [ ] History persists after restart
- [ ] Can restore old clipboard items

**Pass/Fail**: ___________

---

### Test Case 30: Recent Files Tracking

**Objective**: Verify recent files tracking works.

**Steps**:
1. Open search bar with empty query
2. Note recent files (if any)
3. Search for and open 3 files
4. Open search bar with empty query
5. Verify recent files updated
6. Restart application
7. Check recent files again

**Expected Results**:
- [ ] Recent files displayed when query empty
- [ ] Files opened through launcher tracked
- [ ] Recent files persist after restart
- [ ] Most recent files shown first

**Pass/Fail**: ___________

---

## Test Reporting

### Test Summary Template

```
Test Date: _______________
Tester: _______________
Environment: _______________
Application Version: _______________

Total Tests: _____
Passed: _____
Failed: _____
Blocked: _____
Not Tested: _____

Pass Rate: _____%

Critical Issues: _____
Major Issues: _____
Minor Issues: _____
```

### Issue Report Template

```
Issue ID: _____
Test Case: _____
Severity: [Critical/Major/Minor]
Priority: [High/Medium/Low]

Description:
_____________________________________

Steps to Reproduce:
1. _____
2. _____
3. _____

Expected Result:
_____________________________________

Actual Result:
_____________________________________

Screenshots/Logs:
_____________________________________

Environment:
- OS: _____
- Version: _____
- Hardware: _____
```

### Sign-Off

```
Testing completed by: _______________
Date: _______________
Signature: _______________

Approved by: _______________
Date: _______________
Signature: _______________
```

---

## Automated Testing

For automated testing, see:
- Frontend tests: `npm test`
- Backend tests: `cd src-tauri && cargo test`
- Integration tests: `npm run test:integration`

---

Last updated: January 2025
