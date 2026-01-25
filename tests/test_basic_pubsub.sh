#!/bin/bash

# Test Basic Pub/Sub
echo "=== Test 1: Basic Publish/Subscribe ==="

# MQTT Protocol version (default: 311 for MQTT 3.1.1, use 5 for MQTT 5.0)
MQTT_VERSION="${1:-311}"
echo "Using MQTT protocol version: $MQTT_VERSION"

# Mosquitto installation directory (can be overridden)
MOSQUITTO_DIR="${MOSQUITTO_DIR:-C:\Program Files\mosquitto}"
MOSQUITTO_PUB="$MOSQUITTO_DIR/mosquitto_pub.exe"
MOSQUITTO_SUB="$MOSQUITTO_DIR/mosquitto_sub.exe"

# Fall back to command if exe files don't exist (Linux/macOS)
if [ ! -f "$MOSQUITTO_PUB" ]; then
    MOSQUITTO_PUB="mosquitto_pub"
    MOSQUITTO_SUB="mosquitto_sub"
fi

# Start subscriber in background
"$MOSQUITTO_SUB" -h localhost -p 1883 -V $MQTT_VERSION -t "test/basic" -v > /tmp/sub_output.txt &
SUB_PID=$!

# Wait for subscriber to connect
sleep 1

# Publish message
"$MOSQUITTO_PUB" -h localhost -p 1883 -V $MQTT_VERSION -t "test/basic" -m "Hello MQTT"

# Wait for message delivery
sleep 1

# Check output
if grep -q "test/basic Hello MQTT" /tmp/sub_output.txt; then
    echo "✅ PASS: Basic pub/sub works"
    RESULT=0
else
    echo "❌ FAIL: Message not received"
    RESULT=1
fi

# Cleanup
kill $SUB_PID 2>/dev/null
rm -f /tmp/sub_output.txt

exit $RESULT
