#!/bin/bash

# Test Multiple Subscribers
echo "=== Test 3: Multiple Subscribers ==="

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

# Start multiple subscribers
"$MOSQUITTO_SUB" -h localhost -p 1883 -V $MQTT_VERSION -t "broadcast" -v > /tmp/sub1_output.txt &
SUB1_PID=$!

"$MOSQUITTO_SUB" -h localhost -p 1883 -V $MQTT_VERSION -t "broadcast" -v > /tmp/sub2_output.txt &
SUB2_PID=$!

"$MOSQUITTO_SUB" -h localhost -p 1883 -V $MQTT_VERSION -t "broadcast" -v > /tmp/sub3_output.txt &
SUB3_PID=$!

sleep 1

# Publish one message
"$MOSQUITTO_PUB" -h localhost -p 1883 -V $MQTT_VERSION -t "broadcast" -m "Message to all"

sleep 1

# Check all subscribers received it
RESULT=0
for i in 1 2 3; do
    if grep -q "broadcast Message to all" /tmp/sub${i}_output.txt; then
        echo "✅ Subscriber $i received message"
    else
        echo "❌ Subscriber $i did NOT receive message"
        RESULT=1
    fi
done

# Cleanup
kill $SUB1_PID $SUB2_PID $SUB3_PID 2>/dev/null
rm -f /tmp/sub{1,2,3}_output.txt

[ $RESULT -eq 0 ] && echo "✅ PASS: All subscribers received message" || echo "❌ FAIL"
exit $RESULT
