#!/bin/bash

# Test Multiple Subscribers
echo "=== Test 3: Multiple Subscribers ==="

# Start multiple subscribers
"C:\Program Files\mosquitto\mosquitto_sub.exe" -h localhost -p 1883 -t "broadcast" -v > /tmp/sub1_output.txt &
SUB1_PID=$!

"C:\Program Files\mosquitto\mosquitto_sub.exe" -h localhost -p 1883 -t "broadcast" -v > /tmp/sub2_output.txt &
SUB2_PID=$!

"C:\Program Files\mosquitto\mosquitto_sub.exe" -h localhost -p 1883 -t "broadcast" -v > /tmp/sub3_output.txt &
SUB3_PID=$!

sleep 1

# Publish one message
"C:\Program Files\mosquitto\mosquitto_pub.exe" -h localhost -p 1883 -t "broadcast" -m "Message to all"

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
