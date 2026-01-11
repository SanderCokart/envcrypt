#!/bin/bash

# Install/uninstall script for envcrypt
# Usage: ./install.sh [--uninstall]

set -euo pipefail

BIN_DIR="$HOME/.bin"
INSTALL_PATH="$BIN_DIR/envcrypt"

# Helper function to show paths with ~
tildify() {
    if [[ $1 = $HOME/* ]]; then
        local replacement=\~/
        echo "${1/$HOME\//$replacement}"
    else
        echo "$1"
    fi
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
        sed -i '/# envcrypt/,/set --export PATH.*\.bin/d' "$config_file"
    else
        # For bash/zsh, remove the envcrypt section
        sed -i '/# envcrypt/,/export PATH.*\.bin/d' "$config_file"
    fi
}

# Uninstall function
uninstall() {
    set +e  # Don't exit on errors during uninstall
    
    echo "Uninstalling envcrypt..."
    
    # Remove binary
    if [ -f "$INSTALL_PATH" ]; then
        rm "$INSTALL_PATH"
        tilde_exe=$(tildify "$INSTALL_PATH")
        echo "✓ Removed $tilde_exe"
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
    echo "Building envcrypt..."
    
    # Build the release version
    cargo build --release
    
    # Create ~/.bin directory if it doesn't exist
    mkdir -p "$BIN_DIR"
    
    # Copy the binary to ~/.bin/envcrypt
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
    
    # Check if envcrypt is already available in PATH
    if command -v envcrypt >/dev/null 2>&1; then
        echo ""
        echo "Run 'envcrypt --help' to get started"
        exit 0
    fi
    
    # Automatically add ~/.bin to PATH
    CONFIG_FILE=$(detect_shell_config)
    SHELL_NAME=$(basename "$SHELL" 2>/dev/null || echo "bash")
    refresh_command=''
    tilde_config=$(tildify "$CONFIG_FILE")
    tilde_bin_dir=$(tildify "$BIN_DIR")
    
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
                    echo 'set --export PATH "$HOME/.bin" $PATH'
                } >> "$CONFIG_FILE"
            else
                {
                    echo ""
                    echo "# envcrypt"
                    echo 'export PATH="$HOME/.bin:$PATH"'
                } >> "$CONFIG_FILE"
            fi
            echo "Added \"$tilde_bin_dir\" to \$PATH in \"$tilde_config\""
        else
            echo "Manually add the directory to $tilde_config (or similar):"
            if [ "$SHELL_NAME" = "fish" ]; then
                echo "  set --export PATH \"\$HOME/.bin\" \$PATH"
            else
                echo "  export PATH=\"\$HOME/.bin:\$PATH\""
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
