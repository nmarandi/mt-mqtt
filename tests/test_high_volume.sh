#!/bin/bash

# Test High Volume Messages
echo "=== Test 5: High Volume Messages ==="

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

# Start subscriber
"$MOSQUITTO_SUB" -h localhost -p 1883 -V $MQTT_VERSION -t "volume/test" -v > /tmp/volume_output.txt &
SUB_PID=$!

sleep 1

# Publish 100 messages
echo "Publishing 100 messages..."
for i in {1..100}; do
    "$MOSQUITTO_PUB" -h localhost -p 1883 -V $MQTT_VERSION -t "volume/test" -m "Message $i"
    if [ $((i % 10)) -eq 0 ]; then
        echo -ne "Published $i messages\r"
    fi
done
echo ""

sleep 2

# Count received messages
RECEIVED=$(grep -c "volume/test Message" /tmp/volume_output.txt)

echo "Received $RECEIVED out of 100 messages"

if [ $RECEIVED -eq 100 ]; then
    echo "✅ PASS: All 100 messages received"
    RESULT=0
elif [ $RECEIVED -ge 95 ]; then
    echo "⚠️  WARN: $RECEIVED/100 messages received (acceptable)"
    RESULT=0
else
    echo "❌ FAIL: Only $RECEIVED/100 messages received"
    RESULT=1
fi

kill $SUB_PID 2>/dev/null
rm -f /tmp/volume_output.txt

exit $RESULT
