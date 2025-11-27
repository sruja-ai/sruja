#!/bin/bash
# Fallback dev script that uses a simple file watcher if air is not available

# Function to find air binary
find_air() {
    # Check if air is in PATH
    if command -v air &> /dev/null; then
        echo "air"
        return 0
    fi
    
    # Check in GOPATH/bin
    local gopath=$(go env GOPATH 2>/dev/null)
    if [ -n "$gopath" ] && [ -f "$gopath/bin/air" ]; then
        echo "$gopath/bin/air"
        return 0
    fi
    
    # Check in default location
    if [ -f "$HOME/go/bin/air" ]; then
        echo "$HOME/go/bin/air"
        return 0
    fi
    
    return 1
}

AIR_CMD=$(find_air)

if [ -n "$AIR_CMD" ]; then
    echo "Using Air for hot reload..."
    "$AIR_CMD"
else
    echo "Air not found. Installing..."
    go install github.com/air-verse/air@latest
    
    # Try to find it again after installation
    AIR_CMD=$(find_air)
    
    if [ -n "$AIR_CMD" ]; then
        echo "Air installed successfully. Starting with hot reload..."
        "$AIR_CMD"
    else
        echo "Air installation completed but not found in PATH."
        echo "Falling back to simple go run (no hot reload)..."
        echo ""
        echo "To enable hot reload, add Go bin to your PATH:"
        echo "  export PATH=\$PATH:\$(go env GOPATH)/bin"
        echo "  # Add this to ~/.zshrc or ~/.bashrc"
        echo ""
        go run ./main.go
    fi
fi

