# Release Checklist

Complete checklist for releasing a new version of Global Search Launcher.

## Pre-Release (1-2 weeks before)

### Code Freeze
- [ ] All planned features merged to develop branch
- [ ] No new features accepted
- [ ] Only bug fixes allowed
- [ ] Create release branch: `release/vX.X.X`

### Version Updates
- [ ] Update version in `package.json`
- [ ] Update version in `src-tauri/Cargo.toml`
- [ ] Update version in `src-tauri/tauri.conf.json`
- [ ] Update CHANGELOG.md with release notes
- [ ] Update README.md if needed
- [ ] Commit version bump: `git commit -m "chore: bump version to X.X.X"`

### Testing
- [ ] Run full test suite: `npm test`
- [ ] Run Rust tests: `cd src-tauri && cargo test`
- [ ] Manual testing on Windows 10
- [ ] Manual testing on Windows 11
- [ ] Test with Everything SDK installed
- [ ] Test without Everything SDK
- [ ] Test all search providers
- [ ] Test settings changes
- [ ] Test auto-start functionality
- [ ] Test update mechanism (if applicable)
- [ ] Performance testing (see TESTING.md)
- [ ] Memory leak testing
- [ ] Complete TEST_CHECKLIST.md

### Documentation
- [ ] README.md is up to date
- [ ] INSTALLATION.md is accurate
- [ ] TROUBLESHOOTING.md covers known issues
- [ ] CHANGELOG.md is complete
- [ ] API documentation updated (if applicable)
- [ ] Screenshots updated (if UI changed)

### Security
- [ ] Security audit completed
- [ ] No known vulnerabilities
- [ ] Dependencies updated
- [ ] Run `npm audit`
- [ ] Run `cargo audit` (install with `cargo install cargo-audit`)
- [ ] No hardcoded secrets
- [ ] Clipboard encryption working

## Build Phase

### Build Preparation
- [ ] Clean build environment
- [ ] Update dependencies: `npm install`
- [ ] Update Rust dependencies: `cargo update`
- [ ] Run linters: `npm run lint` and `cargo clippy`
- [ ] Format code: `npm run format` and `cargo fmt`

### Build Installers
- [ ] Run build script: `.\scripts\build-installer.ps1`
- [ ] Verify NSIS installer created
- [ ] Verify MSI installer created
- [ ] Check installer file sizes (<50MB)
- [ ] Test installers on clean VM

### Code Signing (if applicable)
- [ ] Sign NSIS installer
- [ ] Sign MSI installer
- [ ] Verify signatures
- [ ] Test signed installers

### Virus Scanning
- [ ] Upload to VirusTotal
- [ ] Verify no false positives
- [ ] If flagged, investigate and resolve
- [ ] Document any known false positives

## Release Phase

### GitHub Release
- [ ] Create Git tag: `git tag -a vX.X.X -m "Release vX.X.X"`
- [ ] Push tag: `git push origin vX.X.X`
- [ ] Create GitHub release
- [ ] Upload NSIS installer
- [ ] Upload MSI installer
- [ ] Copy release notes from CHANGELOG.md
- [ ] Mark as pre-release if beta
- [ ] Publish release

### Update Server
- [ ] Upload installers to update server
- [ ] Create update manifest JSON
- [ ] Sign update manifest (if using Tauri updater)
- [ ] Test update URL accessibility
- [ ] Verify update manifest format

### Update Documentation
- [ ] Update website (if applicable)
- [ ] Update download links
- [ ] Publish blog post (if applicable)
- [ ] Update social media
- [ ] Notify users via email (if applicable)

## Post-Release

### Verification
- [ ] Download installer from release page
- [ ] Verify installer works
- [ ] Test auto-update from previous version
- [ ] Check update server logs
- [ ] Monitor error reports

### Communication
- [ ] Announce on GitHub Discussions
- [ ] Post on social media
- [ ] Update documentation site
- [ ] Notify beta testers
- [ ] Send newsletter (if applicable)

### Monitoring
- [ ] Monitor GitHub Issues for new bugs
- [ ] Check error logs
- [ ] Monitor download statistics
- [ ] Collect user feedback
- [ ] Track crash reports

### Merge Back
- [ ] Merge release branch to main: `git checkout main && git merge release/vX.X.X`
- [ ] Merge release branch to develop: `git checkout develop && git merge release/vX.X.X`
- [ ] Delete release branch: `git branch -d release/vX.X.X`
- [ ] Push changes: `git push origin main develop`

## Hotfix Process (if needed)

### Create Hotfix
- [ ] Create hotfix branch from main: `git checkout -b hotfix/vX.X.X+1 main`
- [ ] Fix the critical bug
- [ ] Update version number (patch increment)
- [ ] Update CHANGELOG.md
- [ ] Test thoroughly
- [ ] Build installers

### Release Hotfix
- [ ] Create Git tag
- [ ] Create GitHub release
- [ ] Upload installers
- [ ] Update update server
- [ ] Announce hotfix

### Merge Hotfix
- [ ] Merge to main: `git checkout main && git merge hotfix/vX.X.X+1`
- [ ] Merge to develop: `git checkout develop && git merge hotfix/vX.X.X+1`
- [ ] Delete hotfix branch
- [ ] Push changes

## Release Types

### Major Release (X.0.0)
- Breaking changes
- Major new features
- Significant UI changes
- Requires migration guide
- Extended testing period (2-4 weeks)

### Minor Release (0.X.0)
- New features
- Non-breaking changes
- Enhancements
- Standard testing period (1-2 weeks)

### Patch Release (0.0.X)
- Bug fixes only
- Security patches
- Performance improvements
- Quick testing period (2-3 days)

## Version Numbering

Follow Semantic Versioning (semver.org):

- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible new features
- **PATCH**: Backwards-compatible bug fixes

Examples:
- `1.0.0` - First stable release
- `1.1.0` - Added new search provider
- `1.1.1` - Fixed crash bug
- `2.0.0` - Redesigned UI (breaking change)

## Release Schedule

### Regular Releases
- **Minor releases**: Every 4-6 weeks
- **Patch releases**: As needed for critical bugs
- **Major releases**: Once or twice per year

### Beta Releases
- 1-2 weeks before stable release
- Marked as pre-release on GitHub
- Announced to beta testers only

## Rollback Plan

If critical issues found after release:

1. **Immediate**:
   - [ ] Remove download links
   - [ ] Update update server to stop serving new version
   - [ ] Post warning on GitHub and website

2. **Short-term**:
   - [ ] Investigate issue
   - [ ] Prepare hotfix
   - [ ] Test hotfix thoroughly

3. **Communication**:
   - [ ] Notify affected users
   - [ ] Provide workaround if available
   - [ ] Announce hotfix timeline

## Sign-Off

### Release Manager

**Name**: ___________________________

**Date**: ___________________________

**Signature**: ___________________________

### QA Lead

**Name**: ___________________________

**Date**: ___________________________

**Signature**: ___________________________

### Technical Lead

**Name**: ___________________________

**Date**: ___________________________

**Signature**: ___________________________

---

## Quick Commands

```bash
# Version bump
npm version patch  # or minor, or major
cd src-tauri && cargo bump patch

# Build
.\scripts\build-installer.ps1

# Tag and push
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0

# Create release (using GitHub CLI)
gh release create v0.1.0 \
  src-tauri/target/release/bundle/nsis/*.exe \
  src-tauri/target/release/bundle/msi/*.msi \
  --title "v0.1.0" \
  --notes-file CHANGELOG.md
```

---

Last updated: January 2025
