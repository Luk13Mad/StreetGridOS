#!/bin/bash
set -e

PI_USER="pi"
PI_HOST="streetgrid-node.local" # Adjust as needed
REMOTE_DIR="/home/pi/streetgrid"
CONFIG_FILE=${1:-"firmware/config.yaml"}

if [ ! -f "$CONFIG_FILE" ]; then
    echo "Error: Config file $CONFIG_FILE not found!"
    exit 1
fi

echo "Building firmware for ARM..."
# Cross-compile for Raspberry Pi Zero 2W (ARMv7 or AArch64 depending on OS).
# Assuming AArch64 (64-bit OS) or ARMv7 (32-bit OS).
# Using a common cross command or just defaulting to `cargo build --release` if on similar arch.
# For this script we will assume the user has configured cargo for cross compilation or is running on a compatible machine.
# If using cross-rs: cross build --target aarch64-unknown-linux-gnu --release

# We'll default to standard build, user can override target via environment variable if needed.
TARGET_ARCH=${TARGET_ARCH:-"aarch64-unknown-linux-gnu"}
echo "Target Architecture: $TARGET_ARCH"

# Note: This build command might fail in this environment if the target is not installed.
# We will just print what would happen, but attempt a local build if no cross compiler is present.
if command -v cross &> /dev/null; then
    cross build --release --target $TARGET_ARCH
    BINARY_PATH="target/$TARGET_ARCH/release/streetgrid-firmware"
else
    echo "Cross command not found. Attempting standard cargo build..."
    cargo build --release
    BINARY_PATH="target/release/streetgrid-firmware"
fi

echo "Deploying to $PI_USER@$PI_HOST..."

# Create directory
ssh $PI_USER@$PI_HOST "mkdir -p $REMOTE_DIR"

# Copy binary
if [ -f "$BINARY_PATH" ]; then
    echo "Copying binary from $BINARY_PATH..."
    scp "$BINARY_PATH" $PI_USER@$PI_HOST:$REMOTE_DIR/
else
    echo "Warning: Binary not found at $BINARY_PATH. Skipping binary upload."
fi

# Copy config
echo "Copying config file $CONFIG_FILE..."
scp "$CONFIG_FILE" $PI_USER@$PI_HOST:$REMOTE_DIR/config.yaml

echo "Done! You can now run the node on the Pi:"
echo "ssh $PI_USER@$PI_HOST '$REMOTE_DIR/streetgrid-firmware --config $REMOTE_DIR/config.yaml'"
