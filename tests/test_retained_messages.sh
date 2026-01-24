#!/bin/bash

# Test Retained Messages
echo "=== Test 4: Retained Messages ==="

# Publish retained message
"C:\Program Files\mosquitto\mosquitto_pub.exe" -h localhost -p 1883 -t "status/server" -m "online" -r

sleep 1

# Subscribe AFTER publishing (should receive retained message)
timeout 2 "C:\Program Files\mosquitto\mosquitto_sub.exe" -h localhost -p 1883 -t "status/server" -v -C 1 > /tmp/retained_output.txt

if grep -q "status/server online" /tmp/retained_output.txt; then
    echo "✅ PASS: Retained message delivered to late subscriber"
    RESULT=0
else
    echo "❌ FAIL: Retained message not delivered"
    RESULT=1
fi

rm -f /tmp/retained_output.txt
exit $RESULT
