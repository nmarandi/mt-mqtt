#!/bin/bash

# Test Wildcard Subscriptions
echo "=== Test 2: Wildcard Subscriptions ==="

# Test multi-level wildcard (#)
echo "Testing multi-level wildcard (#)..."
"C:\Program Files\mosquitto\mosquitto_sub.exe" -h localhost -p 1883 -t "sensor/#" -v > /tmp/wildcard_output.txt &
SUB_PID=$!
sleep 1

"C:\Program Files\mosquitto\mosquitto_pub.exe" -h localhost -p 1883 -t "sensor/temp" -m "20"
"C:\Program Files\mosquitto\mosquitto_pub.exe" -h localhost -p 1883 -t "sensor/humidity/room1" -m "65"
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
"C:\Program Files\mosquitto\mosquitto_sub.exe" -h localhost -p 1883 -t "device/+/status" -v > /tmp/wildcard_output.txt &
SUB_PID=$!
sleep 1

"C:\Program Files\mosquitto\mosquitto_pub.exe" -h localhost -p 1883 -t "device/123/status" -m "online"
"C:\Program Files\mosquitto\mosquitto_pub.exe" -h localhost -p 1883 -t "device/456/status" -m "offline"
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
