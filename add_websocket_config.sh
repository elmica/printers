#!/bin/bash
# Script to add WebSocket support to Mosquitto config

CONFIG_FILE="/opt/homebrew/etc/mosquitto/mosquitto.conf"

if [ ! -f "$CONFIG_FILE" ]; then
    echo "Mosquitto config file not found at $CONFIG_FILE"
    exit 1
fi

# Check if WebSocket listener already exists
if grep -q "listener 9001" "$CONFIG_FILE" 2>/dev/null; then
    echo "WebSocket listener already configured!"
    exit 0
fi

# Backup original config
cp "$CONFIG_FILE" "${CONFIG_FILE}.backup.$(date +%Y%m%d_%H%M%S)"
echo "Backed up config to ${CONFIG_FILE}.backup.*"

# Add WebSocket configuration at the end
cat >> "$CONFIG_FILE" << 'EOF'

# WebSocket listener for browser/Tauri apps
listener 9001
protocol websockets
allow_anonymous true
EOF

echo "✅ Added WebSocket listener configuration to $CONFIG_FILE"
echo ""
echo "To apply changes, restart Mosquitto:"
echo "  pkill mosquitto"
echo "  mosquitto -c $CONFIG_FILE -v"
