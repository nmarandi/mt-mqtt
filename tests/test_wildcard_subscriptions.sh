#!/bin/bash

# Test Wildcard Subscriptions
echo "=== Test 2: Wildcard Subscriptions ==="

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

# Test multi-level wildcard (#)
echo "Testing multi-level wildcard (#)..."
"$MOSQUITTO_SUB" -h localhost -p 1883 -V $MQTT_VERSION -t "sensor/#" -v > /tmp/wildcard_output.txt &
SUB_PID=$!
sleep 1

"$MOSQUITTO_PUB" -h localhost -p 1883 -V $MQTT_VERSION -t "sensor/temp" -m "20"
"$MOSQUITTO_PUB" -h localhost -p 1883 -V $MQTT_VERSION -t "sensor/humidity/room1" -m "65"
sleep 1

if grep -q "sensor/temp 20" /tmp/wildcard_output.txt && \
   grep -q "sensor/humidity/room1 65" /tmp/wildcard_output.txt; then
    echo "✅ PASS: Multi-level wildcard (#) works"
    RESULT1=0
else
    echo "❌ FAIL: Multi-level wildcard failed"
    RESULT1=1
fi

kill $SUB_PID 2>/dev/null
rm -f /tmp/wildcard_output.txt

# Test single-level wildcard (+)
echo "Testing single-level wildcard (+)..."
"$MOSQUITTO_SUB" -h localhost -p 1883 -V $MQTT_VERSION -t "device/+/status" -v > /tmp/wildcard_output.txt &
SUB_PID=$!
sleep 1

"$MOSQUITTO_PUB" -h localhost -p 1883 -V $MQTT_VERSION -t "device/123/status" -m "online"
"$MOSQUITTO_PUB" -h localhost -p 1883 -V $MQTT_VERSION -t "device/456/status" -m "offline"
sleep 1

if grep -q "device/123/status online" /tmp/wildcard_output.txt && \
   grep -q "device/456/status offline" /tmp/wildcard_output.txt; then
    echo "✅ PASS: Single-level wildcard (+) works"
    RESULT2=0
else
    echo "❌ FAIL: Single-level wildcard failed"
    RESULT2=1
fi

kill $SUB_PID 2>/dev/null
rm -f /tmp/wildcard_output.txt

[ $RESULT1 -eq 0 ] && [ $RESULT2 -eq 0 ]
