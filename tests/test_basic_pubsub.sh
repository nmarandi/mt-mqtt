#!/bin/bash

# Test Basic Pub/Sub
echo "=== Test 1: Basic Publish/Subscribe ==="

# Start subscriber in background
"C:\Program Files\mosquitto\mosquitto_sub.exe" -h localhost -p 1883 -t "test/basic" -v > /tmp/sub_output.txt &
SUB_PID=$!

# Wait for subscriber to connect
sleep 1

# Publish message
"C:\Program Files\mosquitto\mosquitto_pub.exe" -h localhost -p 1883 -t "test/basic" -m "Hello MQTT"

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
