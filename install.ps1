# PowerShell installer for FlowSight (Windows)
#
# Usage:
#   .\install.ps1                    # Install latest version
#   .\install.ps1 -Version "0.1.0"   # Install specific version
#   .\install.ps1 -NoDesktop         # CLI only, no desktop app
#
# Or with curl:
#  irm https://flowsight.ai/install.ps1 | iex

param(
    [string]$Version = "",
    [switch]$NoDesktop,
    [switch]$NoModifyPath,
    [switch]$Help
)

if ($Help) {
    Write-Host @"
FlowSight Installer (Windows PowerShell)

Usage: install.ps1 [options]

Options:
    -Version <version>    Install a specific version (e.g., 0.1.0)
    -NoDesktop            Skip Tauri desktop app, CLI only
    -NoModifyPath         Don't modify PATH

Examples:
    # Install latest
    .\install.ps1

    # Specific version
    .\install.ps1 -Version "0.1.0"

    # CLI only
    .\install.ps1 -NoDesktop

    # One-liner (run as Administrator):
    iex "& { $(irm https://flowsight.ai/install.ps1) } -NoModifyPath"

"@
    exit 0
}

$ErrorActionPreference = "Stop"

# Colors for PS Core (PowerShell 7+)
$GREEN = if ($PSVersionTable.PSVersion.Major -ge 7) { "`e[0;32m" } else { "" }
$RED = if ($PSVersionTable.PSVersion.Major -ge 7) { "`e[0;31m" } else { "" }
$ORANGE = if ($PSVersionTable.PSVersion.Major -ge 7) { "`e[38;5;214m" } else { "" }
$NC = if ($PSVersionTable.PSVersion.Major -ge 7) { "`e[0m" } else { "" }

function Write-Info {
    param([string]$Message)
    Write-Host $Message
}

function Write-Success {
    param([string]$Message)
    Write-Host "${GREEN}$Message${NC}"
}

function Write-Warning {
    param([string]$Message)
    Write-Host "${ORANGE}$Message${NC}"
}

function Write-Error {
    param([string]$Message)
    Write-Host "${RED}$Message${NC}"
}

# Detect architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "x86" }
Write-Info "Platform: windows-$arch"

# Check prerequisites
function Check-Prerequisites {
    $missing = @()

    # Check Rust
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        $missing += "Rust (cargo)"
    }

    # Check Node.js
    if (-not $NoDesktop) {
        if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
            $missing += "Node.js"
        }
        if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
            $missing += "pnpm (npm install -g pnpm)"
        }
    }

    # Check Git
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        $missing += "Git"
    }

    if ($missing.Count -gt 0) {
        Write-Error "Missing prerequisites:"
        foreach ($item in $missing) {
            Write-Host "  - $item"
        }
        Write-Host ""
        Write-Host "Please install missing dependencies and try again."
        Write-Host "Visit https://flowsight.ai/docs/install for instructions."
        exit 1
    }
}

# Build from source
function Build-FromSource {
    param([string]$Version)

    Write-Info "Building FlowSight from source..."

    $buildDir = Join-Path $env:TEMP "flowsight_build_$([guid]::NewGuid().ToString('N')[0..7])"
    New-Item -ItemType Directory -Force -Path $buildDir | Out-Null
    Push-Location $buildDir

    try {
        # Clone repository
        if ($Version) {
            Write-Info "Cloning repository at version $Version..."
            git clone --depth 1 --branch "v$Version" "https://github.com/TbusOS/flowsight.git" .
        } else {
            Write-Info "Cloning repository..."
            git clone --depth 1 "https://github.com/TbusOS/flowsight.git" .
        }

        # Build CLI
        Write-Info "Building CLI..."
        cargo build --release -p flowsight-cli

        # Copy binary
        $installDir = Join-Path $env:HOME ".flowsight\bin"
        New-Item -ItemType Directory -Force -Path $installDir | Out-Null
        Copy-Item "target/release/flowsight.exe" $installDir

        # Build desktop app (optional)
        if (-not $NoDesktop) {
            Write-Info "Building desktop app..."
            Set-Location (Join-Path $buildDir "app")
            pnpm install
            pnpm tauri build
            Copy-Item "src-tauri/target/release/bundle/msi/flowsight_*.msi" $installDir -ErrorAction SilentlyContinue
        }

        Write-Success "Build complete!"
        Pop-Location
        return $installDir
    }
    catch {
        Pop-Location
        throw
    }
    finally {
        # Cleanup
        Remove-Item -Recurse -Force $buildDir -ErrorAction SilentlyContinue
    }
}

# Add to PATH
function Add-ToPath {
    param([string]$InstallDir)

    if ($NoModifyPath) {
        Write-Info "Skipping PATH modification (--NoModifyPath)"
        return
    }

    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($currentPath -split ";" -contains $InstallDir) {
        Write-Info "FlowSight already in PATH"
        return
    }

    try {
        $newPath = if ($currentPath) { "$currentPath;$InstallDir" } else { $InstallDir }
        [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
        Write-Success "Added FlowSight to PATH"
    }
    catch {
        Write-Warning "Could not modify PATH automatically"
        Write-Info "Please add this to your PATH: $InstallDir"
    }
}

# Main
Check-Prerequisites
$installDir = Build-FromSource -Version $Version
Add-ToPath -InstallDir $installDir

Write-Host ""
Write-Success "Installation complete!"
Write-Host ""
Write-Host "To get started:"
Write-Host "  flowsight --help       # Show CLI options"
Write-Host "  flowsight analyze <file>  # Analyze a C file"
Write-Host ""
Write-Host "For more information: https://flowsight.ai/docs"
