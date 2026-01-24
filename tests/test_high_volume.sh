#!/bin/bash

# Test High Volume Messages
echo "=== Test 5: High Volume Messages ==="

# Start subscriber
"C:\Program Files\mosquitto\mosquitto_sub.exe" -h localhost -p 1883 -t "volume/test" -v > /tmp/volume_output.txt &
SUB_PID=$!

sleep 1

# Publish 100 messages
echo "Publishing 100 messages..."
for i in {1..100}; do
    "C:\Program Files\mosquitto\mosquitto_pub.exe" -h localhost -p 1883 -t "volume/test" -m "Message $i"
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
