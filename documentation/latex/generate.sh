#!/usr/bin/env bash

# LaTeX documentation build script
# Handles platform differences and gracefully skips if LaTeX is not available

set -e

# Try to set up paths on macOS (homebrew)
if [[ "$OSTYPE" == "darwin"* ]]; then
    # Source path_helper to get homebrew paths (includes MacTeX)
    if [[ -x /usr/libexec/path_helper ]]; then
        eval "$(/usr/libexec/path_helper -s)" 2>/dev/null || true
    fi
    # Also try common homebrew locations directly
    if [[ -d /opt/homebrew/bin ]]; then
        export PATH="/opt/homebrew/bin:$PATH"
    fi
    if [[ -d /usr/local/bin ]]; then
        export PATH="/usr/local/bin:$PATH"
    fi
    # MacTeX paths
    if [[ -d /Library/TeX/texbin ]]; then
        export PATH="/Library/TeX/texbin:$PATH"
    fi
fi

# Check if latexmk is available
if ! command -v latexmk &> /dev/null; then
    echo "latexmk not found - skipping documentation build"
    echo "To build documentation, install LaTeX:"
    echo "  macOS: brew install --cask mactex"
    echo "  Debian/Ubuntu: apt-get install texlive-full latexmk"
    exit 0
fi

# Create output directory if it doesn't exist
mkdir -p output

# Build the PDF
latexmk -pdf -output-directory=output main.tex
