# Installer Test Checklist

Quick reference checklist for installer testing before release.

## Pre-Release Testing Checklist

### Build Verification

- [ ] Frontend builds without errors
- [ ] Backend builds without errors
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] No compiler warnings
- [ ] Version number updated in all files
- [ ] CHANGELOG.md updated

### Installer Build

- [ ] NSIS installer builds successfully
- [ ] MSI installer builds successfully
- [ ] Installer file sizes reasonable (<50MB)
- [ ] Installers not flagged by VirusTotal (test on virustotal.com)

### Windows 10 Testing

#### Clean Install (NSIS)
- [ ] Installs without admin rights (current user)
- [ ] Installs with admin rights (all users)
- [ ] Desktop shortcut created
- [ ] Start menu entry created
- [ ] System tray icon appears
- [ ] Hotkey (Ctrl+K) works
- [ ] All search providers work
- [ ] Settings can be changed
- [ ] Auto-start works

#### Clean Install (MSI)
- [ ] Installs successfully
- [ ] All features work
- [ ] Can be deployed via Group Policy

#### Upgrade Install
- [ ] Detects previous version
- [ ] Settings preserved
- [ ] User data preserved
- [ ] Upgrade completes successfully

#### Uninstall
- [ ] Uninstalls cleanly
- [ ] No leftover files (except user data)
- [ ] Registry cleaned up
- [ ] Shortcuts removed

### Windows 11 Testing

#### Clean Install (NSIS)
- [ ] Installs without admin rights
- [ ] Installs with admin rights
- [ ] All features work
- [ ] UI renders correctly
- [ ] Animations smooth
- [ ] Theme detection works

#### Clean Install (MSI)
- [ ] Installs successfully
- [ ] All features work

#### Upgrade Install
- [ ] Settings preserved
- [ ] Upgrade successful

#### Uninstall
- [ ] Uninstalls cleanly

### Functionality Testing

#### Core Features
- [ ] Global hotkey works from all apps
- [ ] Search bar appears centered
- [ ] File search works (with Everything)
- [ ] File search works (without Everything)
- [ ] Application search works
- [ ] Calculator works
- [ ] Quick actions work
- [ ] Web search fallback works
- [ ] Keyboard navigation works
- [ ] Recent files tracking works
- [ ] Clipboard history works (if enabled)

#### Settings
- [ ] Can change hotkey
- [ ] Can change theme
- [ ] Can enable/disable providers
- [ ] Can change max results
- [ ] Can enable/disable auto-start
- [ ] Settings persist after restart

#### System Integration
- [ ] Auto-start works
- [ ] System tray menu works
- [ ] Can open settings from tray
- [ ] Can exit from tray
- [ ] Window management works
- [ ] Click outside closes window

### Performance Testing

- [ ] Search response < 50ms
- [ ] UI render < 100ms
- [ ] Memory usage < 100MB idle
- [ ] Startup time < 2 seconds
- [ ] No memory leaks after extended use

### Update Testing

- [ ] Update check works
- [ ] Update notification appears
- [ ] Update downloads successfully
- [ ] Update installs successfully
- [ ] Settings preserved after update
- [ ] Can restart to apply update

### Edge Cases

- [ ] Works with non-English Windows
- [ ] Works with high DPI displays
- [ ] Works with multiple monitors
- [ ] Works with special characters in paths
- [ ] Handles insufficient disk space
- [ ] Handles network disconnection
- [ ] Handles Everything not installed
- [ ] Handles hotkey conflicts

### Documentation

- [ ] README.md accurate and complete
- [ ] INSTALLATION.md accurate
- [ ] TROUBLESHOOTING.md covers common issues
- [ ] CHANGELOG.md updated
- [ ] Version numbers consistent
- [ ] Screenshots up to date (if any)

### Security

- [ ] No hardcoded credentials
- [ ] No sensitive data in logs
- [ ] Clipboard history encrypted
- [ ] Settings file permissions correct
- [ ] No unnecessary network requests
- [ ] Update mechanism secure

### Compatibility

- [ ] Works on Windows 10 (1809+)
- [ ] Works on Windows 11
- [ ] Works with 4GB RAM
- [ ] Works with HDD (not just SSD)
- [ ] Works with standard user account
- [ ] Works with admin account

## Sign-Off

### Tested By

**Name**: ___________________________

**Date**: ___________________________

**Signature**: ___________________________

### Test Results

**Total Tests**: _____

**Passed**: _____

**Failed**: _____

**Pass Rate**: _____%

### Critical Issues

List any critical issues found:

1. ___________________________
2. ___________________________
3. ___________________________

### Approval

**Approved for Release**: [ ] Yes [ ] No

**Approved By**: ___________________________

**Date**: ___________________________

**Signature**: ___________________________

---

## Quick Test Script

For rapid testing, run these commands:

```powershell
# Build installers
.\scripts\build-installer.ps1

# Test NSIS installer
$installer = Get-ChildItem "src-tauri\target\release\bundle\nsis\*.exe" | Select-Object -First 1
Start-Process $installer.FullName

# After installation, test basic functionality
# 1. Press Ctrl+K
# 2. Type "notepad"
# 3. Press Enter
# 4. Verify Notepad opens

# Test uninstall
# 1. Open Windows Settings > Apps
# 2. Find "Global Search Launcher"
# 3. Uninstall
# 4. Verify clean removal
```

---

## Automated Test Commands

```bash
# Run all tests
npm test
cd src-tauri && cargo test

# Build and test
npm run bundle
```

---

Last updated: January 2025
