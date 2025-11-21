# Mosquitto WebSocket Setup

## Problem
The Tauri app runs in a browser context, which cannot use raw TCP connections (`mqtt://`). It requires WebSocket connections (`ws://`).

## Solution: Enable WebSocket in Mosquitto

1. **Find your Mosquitto config file:**
   ```bash
   # Common locations:
   /opt/homebrew/etc/mosquitto/mosquitto.conf  # Homebrew on Apple Silicon
   /usr/local/etc/mosquitto/mosquitto.conf     # Homebrew on Intel
   ~/.mosquitto/mosquitto.conf                 # User config
   ```

2. **Add WebSocket listener to the config file:**
   ```conf
   # Existing TCP listener (port 1883)
   listener 1883
   protocol mqtt

   # Add WebSocket listener (port 9001)
   listener 9001
   protocol websockets
   allow_anonymous true
   ```

3. **Restart Mosquitto:**
   ```bash
   # Stop current Mosquitto
   pkill mosquitto
   
   # Start with config file
   mosquitto -c /path/to/mosquitto.conf -v
   ```

4. **In the Tauri app, use one of these URLs:**
   - `ws://localhost:9001` (WebSocket)
   - `mqtt://localhost:1883` (will auto-convert to ws://localhost:9001)

## Alternative: Use WebSocket URL directly

You can also configure the app to use WebSocket URL directly:
- MQTT Server: `ws://localhost:9001`

## Testing WebSocket Connection

Test if WebSocket is working:
```bash
# Check if port 9001 is listening
lsof -i :9001
```

If you see Mosquitto listening on port 9001, WebSocket is enabled!
