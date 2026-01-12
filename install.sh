#!/bin/bash

# Install/uninstall script for envcrypt
# Usage: ./install.sh [--uninstall]

set -euo pipefail

ENVCRYPT_HOME="$HOME/.envcrypt"
BIN_DIR="$ENVCRYPT_HOME/bin"
INSTALL_PATH="$BIN_DIR/envcrypt"

# GitHub repository (can be overridden with ENVCRYPT_REPO environment variable)
# Format: owner/repo (e.g., "username/envcrypt")
# Default repository for this project
ENVCRYPT_REPO="${ENVCRYPT_REPO:-SanderCokart/envcrypt}"

# Try to detect GitHub repo from git remote if default wasn't overridden
if [ "$ENVCRYPT_REPO" = "SanderCokart/envcrypt" ] && command -v git >/dev/null 2>&1 && git rev-parse --git-dir >/dev/null 2>&1; then
    GIT_REMOTE=$(git remote get-url origin 2>/dev/null || echo "")
    if [[ "$GIT_REMOTE" =~ github\.com[:/]([^/]+/[^/]+)\.git?$ ]]; then
        ENVCRYPT_REPO="${BASH_REMATCH[1]}"
        # Remove .git suffix if present
        ENVCRYPT_REPO="${ENVCRYPT_REPO%.git}"
    fi
fi

# Helper function to show paths with ~
tildify() {
    if [[ $1 = $HOME/* ]]; then
        local replacement=\~/
        echo "${1/$HOME\//$replacement}"
    else
        echo "$1"
    fi
}

# Function to detect platform (OS and architecture)
detect_platform() {
    local os
    local arch
    
    # Detect OS
    case "$(uname -s)" in
        Linux*)
            os="linux"
            ;;
        Darwin*)
            os="darwin"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            os="windows"
            ;;
        *)
            os="unknown"
            ;;
    esac
    
    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        armv7l|armv6l)
            arch="arm"
            ;;
        *)
            arch="unknown"
            ;;
    esac
    
    echo "${os}-${arch}"
}

# Function to get latest release version from GitHub
get_latest_version() {
    if [ -z "$ENVCRYPT_REPO" ]; then
        return 1
    fi
    
    local api_url="https://api.github.com/repos/${ENVCRYPT_REPO}/releases/latest"
    local version
    
    # Try to fetch latest release
    if command -v curl >/dev/null 2>&1; then
        version=$(curl -sL "$api_url" | grep -o '"tag_name": "[^"]*' | cut -d'"' -f4 | sed 's/^v//' || echo "")
    elif command -v wget >/dev/null 2>&1; then
        version=$(wget -qO- "$api_url" | grep -o '"tag_name": "[^"]*' | cut -d'"' -f4 | sed 's/^v//' || echo "")
    else
        return 1
    fi
    
    if [ -z "$version" ]; then
        return 1
    fi
    
    echo "$version"
}

# Function to download binary from GitHub Releases
download_binary() {
    local version="$1"
    local platform="$2"
    local output_path="$3"
    
    if [ -z "$ENVCRYPT_REPO" ]; then
        return 1
    fi
    
    local binary_name="envcrypt-${version}-${platform}"
    local download_url="https://github.com/${ENVCRYPT_REPO}/releases/download/v${version}/${binary_name}"
    local temp_file
    
    temp_file=$(mktemp)
    
    # Download binary
    if command -v curl >/dev/null 2>&1; then
        if ! curl -fsSL -o "$temp_file" "$download_url"; then
            rm -f "$temp_file"
            return 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -qO "$temp_file" "$download_url"; then
            rm -f "$temp_file"
            return 1
        fi
    else
        rm -f "$temp_file"
        return 1
    fi
    
    # Verify file was downloaded and is not empty
    if [ ! -s "$temp_file" ]; then
        rm -f "$temp_file"
        return 1
    fi
    
    # Move to final location
    mv "$temp_file" "$output_path"
    chmod +x "$output_path"
    
    return 0
}

# Function to verify binary works
verify_binary() {
    local binary_path="$1"
    
    if [ ! -f "$binary_path" ]; then
        return 1
    fi
    
    if [ ! -x "$binary_path" ]; then
        return 1
    fi
    
    # Try to run --version to verify it works
    if "$binary_path" --version >/dev/null 2>&1; then
        return 0
    fi
    
    return 1
}

# Function to detect shell and get config file
detect_shell_config() {
    SHELL_NAME=$(basename "$SHELL" 2>/dev/null || echo "bash")
    CONFIG_FILE=""
    
    case "$SHELL_NAME" in
        zsh)
            if [ -f "$HOME/.zshrc" ]; then
                CONFIG_FILE="$HOME/.zshrc"
            elif [ -f "$HOME/.zprofile" ]; then
                CONFIG_FILE="$HOME/.zprofile"
            else
                CONFIG_FILE="$HOME/.zshrc"
            fi
            ;;
        fish)
            CONFIG_FILE="$HOME/.config/fish/config.fish"
            mkdir -p "$HOME/.config/fish"
            ;;
        *)
            # Default to bash
            if [ -f "$HOME/.bashrc" ]; then
                CONFIG_FILE="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                CONFIG_FILE="$HOME/.bash_profile"
            elif [ -f "$HOME/.profile" ]; then
                CONFIG_FILE="$HOME/.profile"
            else
                CONFIG_FILE="$HOME/.bashrc"
            fi
            ;;
    esac
    
    echo "$CONFIG_FILE"
}

# Function to remove PATH configuration from shell config
remove_path_config() {
    local config_file="$1"
    
    if [ ! -f "$config_file" ]; then
        return
    fi
    
    local shell_name=$(basename "$SHELL" 2>/dev/null || echo "bash")
    
    # Remove the comment line and the PATH export line
    if [ "$shell_name" = "fish" ]; then
        # For fish shell, remove the envcrypt section
        sed -i '/# envcrypt/,/set --export PATH.*ENVCRYPT_HOME/d' "$config_file"
        sed -i '/export ENVCRYPT_HOME/d' "$config_file"
    else
        # For bash/zsh, remove the envcrypt section
        sed -i '/# envcrypt/,/export PATH.*ENVCRYPT_HOME/d' "$config_file"
        sed -i '/export ENVCRYPT_HOME/d' "$config_file"
    fi
}

# Uninstall function
uninstall() {
    set +e  # Don't exit on errors during uninstall
    
    echo "Uninstalling envcrypt..."
    
    # Remove binary and directory if empty
    if [ -f "$INSTALL_PATH" ]; then
        rm "$INSTALL_PATH"
        tilde_exe=$(tildify "$INSTALL_PATH")
        echo "✓ Removed $tilde_exe"
        
        # Remove bin directory if empty
        if [ -d "$BIN_DIR" ] && [ -z "$(ls -A "$BIN_DIR" 2>/dev/null)" ]; then
            rmdir "$BIN_DIR" 2>/dev/null || true
        fi
        
        # Remove .envcrypt directory if empty
        if [ -d "$ENVCRYPT_HOME" ] && [ -z "$(ls -A "$ENVCRYPT_HOME" 2>/dev/null)" ]; then
            rmdir "$ENVCRYPT_HOME" 2>/dev/null || true
        fi
    else
        tilde_exe=$(tildify "$INSTALL_PATH")
        echo "⚠️  Binary not found at $tilde_exe"
    fi
    
    # Remove PATH configuration
    CONFIG_FILE=$(detect_shell_config)
    tilde_config=$(tildify "$CONFIG_FILE")
    
    if [ -f "$CONFIG_FILE" ] && grep -q "# envcrypt" "$CONFIG_FILE" 2>/dev/null; then
        remove_path_config "$CONFIG_FILE"
        echo "✓ Removed PATH configuration from $tilde_config"
    else
        echo "⚠️  No PATH configuration found in $tilde_config"
    fi
    
    echo ""
    echo "✓ Uninstallation complete!"
    echo "Note: You may need to restart your terminal for PATH changes to take effect."
    
    set -e  # Re-enable exit on error
}

# Install function
install() {
    local platform
    local version
    local build_from_source=false
    
    # Create ~/.envcrypt/bin directory if it doesn't exist
    mkdir -p "$BIN_DIR"
    
    # Detect platform
    platform=$(detect_platform)
    
    # Try to download pre-built binary first
    if [ -n "$ENVCRYPT_REPO" ] && [ "$platform" != "unknown-unknown" ]; then
        echo "Checking for pre-built binary for $platform..."
        
        version=$(get_latest_version)
        if [ -n "$version" ]; then
            echo "Found release version: $version"
            echo "Downloading binary..."
            
            if download_binary "$version" "$platform" "$INSTALL_PATH"; then
                if verify_binary "$INSTALL_PATH"; then
                    tilde_exe=$(tildify "$INSTALL_PATH")
                    echo "✓ envcrypt was installed successfully to $tilde_exe"
                    build_from_source=false
                else
                    echo "⚠️  Downloaded binary failed verification, will build from source"
                    rm -f "$INSTALL_PATH"
                    build_from_source=true
                fi
            else
                echo "⚠️  Could not download pre-built binary, will build from source"
                build_from_source=true
            fi
        else
            echo "⚠️  Could not determine latest release version, will build from source"
            build_from_source=true
        fi
    else
        if [ -z "$ENVCRYPT_REPO" ]; then
            echo "⚠️  GitHub repository not configured, will build from source"
        else
            echo "⚠️  Platform $platform not supported for pre-built binaries, will build from source"
        fi
        build_from_source=true
    fi
    
    # Fall back to building from source if needed
    if [ "$build_from_source" = true ]; then
        if ! command -v cargo >/dev/null 2>&1; then
            echo ""
            echo "Error: Rust and Cargo are required to build from source."
            echo "Please install Rust from https://rustup.rs/ or ensure a pre-built binary is available."
            if [ -n "$ENVCRYPT_REPO" ]; then
                echo ""
                echo "Alternatively, you can set ENVCRYPT_REPO environment variable:"
                echo "  export ENVCRYPT_REPO=\"owner/repo\""
                echo "  ./install.sh"
            fi
            exit 1
        fi
        
        echo "Building envcrypt from source..."
        
        # Build the release version
        cargo build --release
        
        # Copy the binary to ~/.envcrypt/bin/envcrypt
        BINARY_PATH="target/release/envcrypt"
        
        if [ ! -f "$BINARY_PATH" ]; then
            echo "Error: Binary not found at $BINARY_PATH"
            echo "Build may have failed. Please check the output above."
            exit 1
        fi
        
        cp "$BINARY_PATH" "$INSTALL_PATH"
        chmod +x "$INSTALL_PATH"
        
        tilde_exe=$(tildify "$INSTALL_PATH")
        echo "✓ envcrypt was installed successfully to $tilde_exe"
    fi
    
    # Add to PATH in current session immediately
    export ENVCRYPT_HOME
    export PATH="$BIN_DIR:$PATH"
    
    # Check if envcrypt is already available in PATH
    if command -v envcrypt >/dev/null 2>&1; then
        echo ""
        echo "Run 'envcrypt --help' to get started"
        exit 0
    fi
    
    # Automatically add ~/.envcrypt/bin to PATH
    CONFIG_FILE=$(detect_shell_config)
    SHELL_NAME=$(basename "$SHELL" 2>/dev/null || echo "bash")
    refresh_command=''
    tilde_config=$(tildify "$CONFIG_FILE")
    tilde_home=$(tildify "$ENVCRYPT_HOME")
    quoted_home="${ENVCRYPT_HOME//\"/\\\"}"
    
    # Check if PATH export already exists in config file
    if [ -f "$CONFIG_FILE" ] && grep -q "# envcrypt" "$CONFIG_FILE" 2>/dev/null; then
        # Already configured
        :
    else
        # Add PATH export to config file if writable
        if [ -w "$CONFIG_FILE" ] || [ ! -f "$CONFIG_FILE" ]; then
            if [ "$SHELL_NAME" = "fish" ]; then
                {
                    echo ""
                    echo "# envcrypt"
                    echo "set --export ENVCRYPT_HOME \"$quoted_home\""
                    echo "set --export PATH \"\$ENVCRYPT_HOME/bin\" \$PATH"
                } >> "$CONFIG_FILE"
            else
                {
                    echo ""
                    echo "# envcrypt"
                    echo "export ENVCRYPT_HOME=\"$quoted_home\""
                    echo "export PATH=\"\$ENVCRYPT_HOME/bin:\$PATH\""
                } >> "$CONFIG_FILE"
            fi
            echo "Added \"$tilde_home/bin\" to \$PATH in \"$tilde_config\""
        else
            echo "Manually add the directory to $tilde_config (or similar):"
            if [ "$SHELL_NAME" = "fish" ]; then
                echo "  set --export ENVCRYPT_HOME \"\$HOME/.envcrypt\""
                echo "  set --export PATH \"\$ENVCRYPT_HOME/bin\" \$PATH"
            else
                echo "  export ENVCRYPT_HOME=\"\$HOME/.envcrypt\""
                echo "  export PATH=\"\$ENVCRYPT_HOME/bin:\$PATH\""
            fi
        fi
    fi
    
    # Determine refresh command based on shell
    case "$SHELL_NAME" in
        fish)
            refresh_command="source $tilde_config"
            ;;
        zsh)
            refresh_command="exec $SHELL"
            ;;
        bash)
            refresh_command="source $CONFIG_FILE"
            ;;
        *)
            refresh_command="source $CONFIG_FILE"
            ;;
    esac
    
    echo ""
    echo "To get started, run:"
    echo ""
    if [ -n "$refresh_command" ]; then
        echo "  $refresh_command"
    fi
    echo "  envcrypt --help"
}

# Main script logic
if [ "${1:-}" = "--uninstall" ]; then
    uninstall
else
    install
fi
