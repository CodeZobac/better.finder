# Build Installer Script for Global Search Launcher
# This script builds both NSIS and MSI installers

param(
    [switch]$Debug,
    [switch]$NsisOnly,
    [switch]$MsiOnly,
    [switch]$SkipTests
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Global Search Launcher - Build Installer" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check prerequisites
Write-Host "Checking prerequisites..." -ForegroundColor Yellow

# Check Node.js
try {
    $nodeVersion = node --version
    Write-Host "✓ Node.js: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Node.js not found. Please install Node.js 18+" -ForegroundColor Red
    exit 1
}

# Check npm
try {
    $npmVersion = npm --version
    Write-Host "✓ npm: $npmVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ npm not found" -ForegroundColor Red
    exit 1
}

# Check Rust
try {
    $rustVersion = rustc --version
    Write-Host "✓ Rust: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Rust not found. Please install Rust from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Check Cargo
try {
    $cargoVersion = cargo --version
    Write-Host "✓ Cargo: $cargoVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Cargo not found" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Install dependencies
Write-Host "Installing dependencies..." -ForegroundColor Yellow
npm install
if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ Failed to install dependencies" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Dependencies installed" -ForegroundColor Green
Write-Host ""

# Run tests (unless skipped)
if (-not $SkipTests) {
    Write-Host "Running tests..." -ForegroundColor Yellow
    
    # Frontend tests
    Write-Host "  Running frontend tests..." -ForegroundColor Cyan
    npm test
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ Frontend tests failed" -ForegroundColor Red
        exit 1
    }
    Write-Host "  ✓ Frontend tests passed" -ForegroundColor Green
    
    # Backend tests
    Write-Host "  Running backend tests..." -ForegroundColor Cyan
    Set-Location src-tauri
    cargo test --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ Backend tests failed" -ForegroundColor Red
        Set-Location ..
        exit 1
    }
    Set-Location ..
    Write-Host "  ✓ Backend tests passed" -ForegroundColor Green
    Write-Host ""
} else {
    Write-Host "⚠ Skipping tests" -ForegroundColor Yellow
    Write-Host ""
}

# Build frontend
Write-Host "Building frontend..." -ForegroundColor Yellow
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ Frontend build failed" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Frontend built successfully" -ForegroundColor Green
Write-Host ""

# Determine build mode
$buildMode = if ($Debug) { "--debug" } else { "" }
$buildModeText = if ($Debug) { "Debug" } else { "Release" }

Write-Host "Building installers ($buildModeText mode)..." -ForegroundColor Yellow
Write-Host ""

# Build NSIS installer
if (-not $MsiOnly) {
    Write-Host "Building NSIS installer..." -ForegroundColor Cyan
    if ($Debug) {
        npm run tauri build -- --debug --bundles nsis
    } else {
        npm run bundle:nsis
    }
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ NSIS installer build failed" -ForegroundColor Red
        exit 1
    }
    Write-Host "✓ NSIS installer built successfully" -ForegroundColor Green
    Write-Host ""
}

# Build MSI installer
if (-not $NsisOnly) {
    Write-Host "Building MSI installer..." -ForegroundColor Cyan
    if ($Debug) {
        npm run tauri build -- --debug --bundles msi
    } else {
        npm run bundle:msi
    }
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ MSI installer build failed" -ForegroundColor Red
        exit 1
    }
    Write-Host "✓ MSI installer built successfully" -ForegroundColor Green
    Write-Host ""
}

# Show output location
$outputDir = if ($Debug) { "src-tauri\target\debug\bundle" } else { "src-tauri\target\release\bundle" }

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Build Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Installers location:" -ForegroundColor Yellow
Write-Host "  $outputDir" -ForegroundColor White
Write-Host ""

# List built installers
if (Test-Path $outputDir) {
    Write-Host "Built installers:" -ForegroundColor Yellow
    
    if (-not $MsiOnly) {
        $nsisPath = Join-Path $outputDir "nsis"
        if (Test-Path $nsisPath) {
            Get-ChildItem $nsisPath -Filter "*.exe" | ForEach-Object {
                $size = [math]::Round($_.Length / 1MB, 2)
                Write-Host "  ✓ $($_.Name) ($size MB)" -ForegroundColor Green
            }
        }
    }
    
    if (-not $NsisOnly) {
        $msiPath = Join-Path $outputDir "msi"
        if (Test-Path $msiPath) {
            Get-ChildItem $msiPath -Filter "*.msi" | ForEach-Object {
                $size = [math]::Round($_.Length / 1MB, 2)
                Write-Host "  ✓ $($_.Name) ($size MB)" -ForegroundColor Green
            }
        }
    }
} else {
    Write-Host "⚠ Output directory not found: $outputDir" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Test the installers on clean Windows 10 and 11 systems" -ForegroundColor White
Write-Host "  2. Verify all functionality works correctly" -ForegroundColor White
Write-Host "  3. Run the test suite from TESTING.md" -ForegroundColor White
Write-Host "  4. Sign the installers (if applicable)" -ForegroundColor White
Write-Host "  5. Upload to release server" -ForegroundColor White
Write-Host ""
