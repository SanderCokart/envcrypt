# Install/uninstall script for envcrypt (Windows PowerShell)
# Usage: .\install.ps1 [--uninstall]

param(
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

$ENVCRYPT_HOME = "$env:USERPROFILE\.envcrypt"
$BIN_DIR = "$ENVCRYPT_HOME\bin"
$INSTALL_PATH = "$BIN_DIR\envcrypt.exe"

# GitHub repository (can be overridden with ENVCRYPT_REPO environment variable)
# Format: owner/repo (e.g., "username/envcrypt")
# Default repository for this project
$ENVCRYPT_REPO = if ($env:ENVCRYPT_REPO) { $env:ENVCRYPT_REPO } else { "SanderCokart/envcrypt" }

# Try to detect GitHub repo from git remote if default wasn't overridden
if ($ENVCRYPT_REPO -eq "SanderCokart/envcrypt") {
    try {
        $gitRemote = git remote get-url origin 2>$null
        if ($gitRemote -match 'github\.com[:/]([^/]+/[^/]+)\.git?$') {
            $ENVCRYPT_REPO = $matches[1] -replace '\.git$', ''
        }
    } catch {
        # Git not available or not in a git repo, use default
    }
}

# Function to detect platform (OS and architecture)
function Detect-Platform {
    $os = "windows"
    $arch = "unknown"
    
    # Detect architecture
    switch ($env:PROCESSOR_ARCHITECTURE) {
        "AMD64" { $arch = "x86_64" }
        "ARM64" { $arch = "aarch64" }
        default {
            # Fallback to checking processor
            if ([Environment]::Is64BitOperatingSystem) {
                $arch = "x86_64"
            } else {
                $arch = "x86"
            }
        }
    }
    
    return "${os}-${arch}"
}

# Function to get latest release version from GitHub
function Get-LatestVersion {
    if (-not $ENVCRYPT_REPO) {
        return $null
    }
    
    $apiUrl = "https://api.github.com/repos/$ENVCRYPT_REPO/releases/latest"
    
    try {
        $response = Invoke-RestMethod -Uri $apiUrl -Method Get -ErrorAction Stop
        $version = $response.tag_name -replace '^v', ''
        return $version
    } catch {
        return $null
    }
}

# Function to download binary from GitHub Releases
function Download-Binary {
    param(
        [string]$Version,
        [string]$Platform,
        [string]$OutputPath
    )
    
    if (-not $ENVCRYPT_REPO) {
        return $false
    }
    
    $binaryName = "envcrypt-${Version}-${Platform}.exe"
    $downloadUrl = "https://github.com/$ENVCRYPT_REPO/releases/download/v${Version}/$binaryName"
    $tempFile = [System.IO.Path]::GetTempFileName()
    
    try {
        Write-Host "Downloading from $downloadUrl..."
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -ErrorAction Stop
        
        # Verify file was downloaded and is not empty
        if (-not (Test-Path $tempFile) -or (Get-Item $tempFile).Length -eq 0) {
            Remove-Item $tempFile -ErrorAction SilentlyContinue
            return $false
        }
        
        # Move to final location
        $outputDir = Split-Path -Parent $OutputPath
        if (-not (Test-Path $outputDir)) {
            New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
        }
        
        Move-Item -Path $tempFile -Destination $OutputPath -Force
        return $true
    } catch {
        Remove-Item $tempFile -ErrorAction SilentlyContinue
        return $false
    }
}

# Function to verify binary works
function Test-Binary {
    param([string]$BinaryPath)
    
    if (-not (Test-Path $BinaryPath)) {
        return $false
    }
    
    try {
        $result = & $BinaryPath --version 2>&1
        return $LASTEXITCODE -eq 0
    } catch {
        return $false
    }
}

# Function to add directory to PATH
function Add-ToPath {
    param([string]$PathToAdd)
    
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $pathParts = $userPath -split ';' | Where-Object { $_ -ne $PathToAdd -and $_ -ne '' }
    $newPath = ($pathParts + $PathToAdd) -join ';'
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    
    # Also add to current session
    $env:Path = "$PathToAdd;$env:Path"
}

# Function to remove directory from PATH
function Remove-FromPath {
    param([string]$PathToRemove)
    
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $pathParts = $userPath -split ';' | Where-Object { $_ -ne $PathToRemove -and $_ -ne '' }
    $newPath = $pathParts -join ';'
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
}

# Uninstall function
function Uninstall-Envcrypt {
    Write-Host "Uninstalling envcrypt..."
    
    # Remove binary
    if (Test-Path $INSTALL_PATH) {
        Remove-Item $INSTALL_PATH -Force
        Write-Host "✓ Removed $INSTALL_PATH"
        
        # Remove bin directory if empty
        if (Test-Path $BIN_DIR) {
            $items = Get-ChildItem $BIN_DIR -ErrorAction SilentlyContinue
            if (-not $items) {
                Remove-Item $BIN_DIR -ErrorAction SilentlyContinue
            }
        }
        
        # Remove .envcrypt directory if empty
        if (Test-Path $ENVCRYPT_HOME) {
            $items = Get-ChildItem $ENVCRYPT_HOME -ErrorAction SilentlyContinue
            if (-not $items) {
                Remove-Item $ENVCRYPT_HOME -ErrorAction SilentlyContinue
            }
        }
    } else {
        Write-Host "⚠️  Binary not found at $INSTALL_PATH"
    }
    
    # Remove from PATH
    Remove-FromPath $BIN_DIR
    Write-Host "✓ Removed from PATH"
    
    Write-Host ""
    Write-Host "✓ Uninstallation complete!"
    Write-Host "Note: You may need to restart your terminal for PATH changes to take effect."
}

# Install function
function Install-Envcrypt {
    $platform = Detect-Platform
    $buildFromSource = $false
    
    # Create .envcrypt\bin directory if it doesn't exist
    if (-not (Test-Path $BIN_DIR)) {
        New-Item -ItemType Directory -Path $BIN_DIR -Force | Out-Null
    }
    
    # Try to download pre-built binary first
    if ($ENVCRYPT_REPO -and $platform -ne "unknown-unknown") {
        Write-Host "Checking for pre-built binary for $platform..."
        
        $version = Get-LatestVersion
        if ($version) {
            Write-Host "Found release version: $version"
            Write-Host "Downloading binary..."
            
            if (Download-Binary $version $platform $INSTALL_PATH) {
                if (Test-Binary $INSTALL_PATH) {
                    Write-Host "✓ envcrypt was installed successfully to $INSTALL_PATH"
                    $buildFromSource = $false
                } else {
                    Write-Host "⚠️  Downloaded binary failed verification, will build from source"
                    Remove-Item $INSTALL_PATH -ErrorAction SilentlyContinue
                    $buildFromSource = $true
                }
            } else {
                Write-Host "⚠️  Could not download pre-built binary, will build from source"
                $buildFromSource = $true
            }
        } else {
            Write-Host "⚠️  Could not determine latest release version, will build from source"
            $buildFromSource = $true
        }
    } else {
        if (-not $ENVCRYPT_REPO) {
            Write-Host "⚠️  GitHub repository not configured, will build from source"
        } else {
            Write-Host "⚠️  Platform $platform not supported for pre-built binaries, will build from source"
        }
        $buildFromSource = $true
    }
    
    # Fall back to building from source if needed
    if ($buildFromSource) {
        if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
            Write-Host ""
            Write-Host "Error: Rust and Cargo are required to build from source."
            Write-Host "Please install Rust from https://rustup.rs/ or ensure a pre-built binary is available."
            if ($ENVCRYPT_REPO) {
                Write-Host ""
                Write-Host "Alternatively, you can set ENVCRYPT_REPO environment variable:"
                Write-Host "  `$env:ENVCRYPT_REPO = 'owner/repo'"
                Write-Host "  .\install.ps1"
            }
            exit 1
        }
        
        Write-Host "Building envcrypt from source..."
        
        # Build the release version
        cargo build --release
        
        # Copy the binary to .envcrypt\bin\envcrypt.exe
        $binaryPath = "target\release\envcrypt.exe"
        
        if (-not (Test-Path $binaryPath)) {
            Write-Host "Error: Binary not found at $binaryPath"
            Write-Host "Build may have failed. Please check the output above."
            exit 1
        }
        
        Copy-Item $binaryPath $INSTALL_PATH -Force
        Write-Host "✓ envcrypt was installed successfully to $INSTALL_PATH"
    }
    
    # Add to PATH in current session immediately
    $env:ENVCRYPT_HOME = $ENVCRYPT_HOME
    $env:Path = "$BIN_DIR;$env:Path"
    
    # Check if envcrypt is already available in PATH
    if (Get-Command envcrypt -ErrorAction SilentlyContinue) {
        Write-Host ""
        Write-Host "Run 'envcrypt --help' to get started"
        exit 0
    }
    
    # Add to user PATH
    Add-ToPath $BIN_DIR
    Write-Host "Added $BIN_DIR to PATH"
    
    Write-Host ""
    Write-Host "To get started, run:"
    Write-Host "  envcrypt --help"
    Write-Host ""
    Write-Host "Note: You may need to restart your terminal for PATH changes to take effect."
}

# Main script logic
if ($Uninstall) {
    Uninstall-Envcrypt
} else {
    Install-Envcrypt
}
