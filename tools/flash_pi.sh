#!/bin/bash
echo "Flashing Pi..."
# This script simulates deploying the firmware and configuration to a remote Pi.

# Usage: ./flash_pi.sh <user>@<host> <config_file>

TARGET=$1
CONFIG=$2

if [ -z "$TARGET" ]; then
    echo "Usage: ./flash_pi.sh <user>@<host> [config_file]"
    exit 1
fi

echo "Building release binary..."
# Assuming we are in the root, cd to firmware
cd firmware && cargo build --release
cd ..

BINARY="firmware/target/release/streetgrid-firmware"

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    exit 1
fi

echo "Deploying binary to $TARGET..."
# rsync -avz --progress "$BINARY" "$TARGET:~/streetgrid-firmware"

if [ -n "$CONFIG" ] && [ -f "$CONFIG" ]; then
    echo "Deploying config $CONFIG to $TARGET..."
    # rsync -avz --progress "$CONFIG" "$TARGET:~/config.yaml"
else
    echo "No config file specified or found. Skipping config deployment."
fi

echo "Done. To run on Pi:"
echo "  ssh $TARGET './streetgrid-firmware config.yaml'"
