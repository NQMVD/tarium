#!/bin/bash

# Build Tarium with Embedded GitHub App Credentials
# This script builds Tarium with GitHub App credentials embedded in the binary

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_colored() {
    echo -e "${1}${2}${NC}"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 <app_id> <installation_id> <private_key_path> [target] [profile] [extra_flags]"
    echo ""
    echo "Arguments:"
    echo "  app_id           Your GitHub App ID (required)"
    echo "  installation_id  Your GitHub App Installation ID (required)"
    echo "  private_key_path Path to your private key .pem file (required)"
    echo "  target          Build target (optional, e.g., x86_64-unknown-linux-gnu)"
    echo "  profile         Build profile (optional, debug or release, default: release)"
    echo "  extra_flags     Additional cargo build flags (optional)"
    echo ""
    echo "Examples:"
    echo "  $0 1910665 84660496 ./tarium-private-key.pem"
    echo "  $0 1910665 84660496 ./key.pem x86_64-unknown-linux-gnu release"
    echo "  $0 1910665 84660496 ./key.pem \"\" release \"--features extra\""
    echo ""
    echo "For more information, see GITHUB_APP_SETUP.md"
}

# Check if minimum arguments provided
if [ $# -lt 3 ]; then
    print_colored $RED "Error: Missing required arguments"
    echo ""
    show_usage
    exit 1
fi

APP_ID="$1"
INSTALLATION_ID="$2"
PRIVATE_KEY_PATH="$3"
TARGET="${4:-}"
PROFILE="${5:-release}"
EXTRA_FLAGS="${6:-}"

print_colored $BLUE "🚀 Building Tarium with embedded GitHub App credentials..."

# Validate parameters
if [ -z "$APP_ID" ]; then
    print_colored $RED "Error: App ID cannot be empty"
    exit 1
fi

if [ -z "$INSTALLATION_ID" ]; then
    print_colored $RED "Error: Installation ID cannot be empty"
    exit 1
fi

if [ -z "$PRIVATE_KEY_PATH" ]; then
    print_colored $RED "Error: Private key path cannot be empty"
    exit 1
fi

# Check if private key file exists
if [ ! -f "$PRIVATE_KEY_PATH" ]; then
    print_colored $RED "Error: Private key file not found at: $PRIVATE_KEY_PATH"
    exit 1
fi

# Resolve full path
FULL_PRIVATE_KEY_PATH=$(realpath "$PRIVATE_KEY_PATH")

# Validate App ID is numeric
if ! [[ "$APP_ID" =~ ^[0-9]+$ ]]; then
    print_colored $RED "Error: App ID must be numeric: $APP_ID"
    exit 1
fi

# Validate Installation ID is numeric
if ! [[ "$INSTALLATION_ID" =~ ^[0-9]+$ ]]; then
    print_colored $RED "Error: Installation ID must be numeric: $INSTALLATION_ID"
    exit 1
fi

# Validate profile
if [ "$PROFILE" != "debug" ] && [ "$PROFILE" != "release" ]; then
    print_colored $RED "Error: Profile must be 'debug' or 'release': $PROFILE"
    exit 1
fi

print_colored $GREEN "✅ App ID: $APP_ID"
print_colored $GREEN "✅ Installation ID: $INSTALLATION_ID"
print_colored $GREEN "✅ Private Key: $FULL_PRIVATE_KEY_PATH"
print_colored $GREEN "✅ Profile: $PROFILE"

if [ -n "$TARGET" ]; then
    print_colored $GREEN "✅ Target: $TARGET"
fi

if [ -n "$EXTRA_FLAGS" ]; then
    print_colored $GREEN "✅ Extra flags: $EXTRA_FLAGS"
fi

# Set environment variables for the build
export TARIUM_EMBED_CREDENTIALS="1"
export TARIUM_EMBED_GITHUB_APP_ID="$APP_ID"
export TARIUM_EMBED_GITHUB_INSTALLATION_ID="$INSTALLATION_ID"
export TARIUM_EMBED_GITHUB_PRIVATE_KEY_PATH="$FULL_PRIVATE_KEY_PATH"

# Build the cargo command
CARGO_ARGS="build"

if [ "$PROFILE" = "release" ]; then
    CARGO_ARGS="$CARGO_ARGS --release"
fi

if [ -n "$TARGET" ]; then
    CARGO_ARGS="$CARGO_ARGS --target $TARGET"
fi

if [ -n "$EXTRA_FLAGS" ]; then
    CARGO_ARGS="$CARGO_ARGS $EXTRA_FLAGS"
fi

echo ""
print_colored $YELLOW "🔨 Running cargo build..."
print_colored $CYAN "Command: cargo $CARGO_ARGS"

# Run the build
if cargo $CARGO_ARGS; then
    echo ""
    print_colored $GREEN "🎉 Build completed successfully!"

    # Determine binary path
    BINARY_NAME="tarium"
    if [ -n "$TARGET" ]; then
        BINARY_PATH="target/$TARGET/$PROFILE/$BINARY_NAME"
    else
        BINARY_PATH="target/$PROFILE/$BINARY_NAME"
    fi

    if [ -f "$BINARY_PATH" ]; then
        FILE_SIZE=$(stat -c%s "$BINARY_PATH" 2>/dev/null || stat -f%z "$BINARY_PATH" 2>/dev/null || echo "unknown")
        if [ "$FILE_SIZE" != "unknown" ]; then
            FILE_SIZE_MB=$(echo "scale=2; $FILE_SIZE / 1024 / 1024" | bc 2>/dev/null || echo "unknown")
        else
            FILE_SIZE_MB="unknown"
        fi

        print_colored $GREEN "📦 Binary created: $BINARY_PATH"
        print_colored $GREEN "📏 Size: ${FILE_SIZE_MB} MB"
        echo ""
        print_colored $GREEN "✅ The binary now contains embedded GitHub App credentials"
        print_colored $GREEN "✅ Users can run it without any authentication setup"
        echo ""
        print_colored $BLUE "🧪 Test the binary with:"
        print_colored $CYAN "  ./$BINARY_PATH auth status"
    else
        print_colored $YELLOW "Warning: Binary not found at expected path: $BINARY_PATH"
    fi
else
    BUILD_EXIT_CODE=$?
    print_colored $RED "Error: Build failed with exit code: $BUILD_EXIT_CODE"
    exit $BUILD_EXIT_CODE
fi

# Clean up environment variables
unset TARIUM_EMBED_CREDENTIALS
unset TARIUM_EMBED_GITHUB_APP_ID
unset TARIUM_EMBED_GITHUB_INSTALLATION_ID
unset TARIUM_EMBED_GITHUB_PRIVATE_KEY_PATH

echo ""
print_colored $YELLOW "🔒 Security note:"
print_colored $NC "  The private key is now embedded in the binary"
print_colored $NC "  Treat this binary as sensitive - don't share in public repositories"
print_colored $NC "  For open source distribution, use a separate GitHub App"
