#!/bin/bash

# Test Retained Messages
echo "=== Test 4: Retained Messages ==="

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

# Publish retained message
"$MOSQUITTO_PUB" -h localhost -p 1883 -V $MQTT_VERSION -t "status/server" -m "online" -r

sleep 1

# Subscribe AFTER publishing (should receive retained message)
"$MOSQUITTO_SUB" -h localhost -p 1883 -V $MQTT_VERSION -t "status/server" -v -C 1 > /tmp/retained_output.txt 2>&1

if grep -q "status/server online" /tmp/retained_output.txt; then
    echo "✅ PASS: Retained message delivered to late subscriber"
    RESULT=0
else
    echo "❌ FAIL: Retained message not delivered"
    RESULT=1
fi

rm -f /tmp/retained_output.txt
exit $RESULT
