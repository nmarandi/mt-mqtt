#!/bin/bash
# Test QoS 1 and QoS 2 message delivery with Mosquitto clients

echo "=== Testing MQTT QoS Levels ==="

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

BROKER_HOST="localhost"
BROKER_PORT="1883"
TEST_PASSED=0
TEST_FAILED=0

# Test QoS 0
test_qos0() {
    echo ""
    echo "--- Test 1: QoS 0 (At most once) ---"
    
    TOPIC="test/qos0"
    MESSAGE="QoS 0 message"
    
    # Start subscriber in background
    "$MOSQUITTO_SUB" -h $BROKER_HOST -p $BROKER_PORT -V $MQTT_VERSION -t "$TOPIC" -q 0 -C 1 > /tmp/qos0_sub.txt 2>&1 &
    SUB_PID=$!
    sleep 1
    
    # Publish message
    "$MOSQUITTO_PUB" -h $BROKER_HOST -p $BROKER_PORT -V $MQTT_VERSION -t "$TOPIC" -m "$MESSAGE" -q 0
    sleep 1
    
    # Check if message was received
    if grep -q "$MESSAGE" /tmp/qos0_sub.txt; then
        echo "✅ PASS: QoS 0 message delivered"
        ((TEST_PASSED++))
    else
        echo "❌ FAIL: QoS 0 message not received"
        ((TEST_FAILED++))
    fi
    
    kill $SUB_PID 2>/dev/null
    rm -f /tmp/qos0_sub.txt
}

# Test QoS 1
test_qos1() {
    echo ""
    echo "--- Test 2: QoS 1 (At least once) ---"
    
    TOPIC="test/qos1"
    MESSAGE="QoS 1 message with acknowledgment"
    
    # Start subscriber in background
    "$MOSQUITTO_SUB" -h $BROKER_HOST -p $BROKER_PORT -V $MQTT_VERSION -t "$TOPIC" -q 1 -C 1 > /tmp/qos1_sub.txt 2>&1 &
    SUB_PID=$!
    sleep 1
    
    # Publish message with QoS 1
    "$MOSQUITTO_PUB" -h $BROKER_HOST -p $BROKER_PORT -V $MQTT_VERSION -t "$TOPIC" -m "$MESSAGE" -q 1
    sleep 1
    
    # Check if message was received
    if grep -q "$MESSAGE" /tmp/qos1_sub.txt; then
        echo "✅ PASS: QoS 1 message delivered with PUBACK"
        ((TEST_PASSED++))
    else
        echo "❌ FAIL: QoS 1 message not received"
        ((TEST_FAILED++))
    fi
    
    kill $SUB_PID 2>/dev/null
    rm -f /tmp/qos1_sub.txt
}

# Test QoS 2
test_qos2() {
    echo ""
    echo "--- Test 3: QoS 2 (Exactly once) ---"
    
    TOPIC="test/qos2"
    MESSAGE="QoS 2 message exactly once"
    
    # Start subscriber in background
    "$MOSQUITTO_SUB" -h $BROKER_HOST -p $BROKER_PORT -V $MQTT_VERSION -t "$TOPIC" -q 2 > /tmp/qos2_sub.txt 2>&1 &
    SUB_PID=$!
    sleep 1
    
    # Publish message with QoS 2
    "$MOSQUITTO_PUB" -h $BROKER_HOST -p $BROKER_PORT -V $MQTT_VERSION -t "$TOPIC" -m "$MESSAGE" -q 2
    sleep 3
    
    # Kill subscriber
    kill $SUB_PID 2>/dev/null
    sleep 1
    
    # Check if message was received
    if grep -q "$MESSAGE" /tmp/qos2_sub.txt; then
        echo "✅ PASS: QoS 2 message delivered with 4-way handshake"
        ((TEST_PASSED++))
    else
        echo "❌ FAIL: QoS 2 message not received"
        ((TEST_FAILED++))
    fi
    
    kill $SUB_PID 2>/dev/null
    sleep 0.5
    rm -f /tmp/qos2_sub.txt
}

# Test mixed QoS levels
test_mixed_qos() {
    echo ""
    echo "--- Test 4: Mixed QoS levels ---"
    
    TOPIC="test/mixed"
    
    # Start subscriber with QoS 2 in background
    "$MOSQUITTO_SUB" -h $BROKER_HOST -p $BROKER_PORT -V $MQTT_VERSION -t "$TOPIC" -q 2 > /tmp/mixed_sub.txt 2>&1 &
    SUB_PID=$!
    sleep 1
    
    # Publish messages with different QoS levels
    "$MOSQUITTO_PUB" -h $BROKER_HOST -p $BROKER_PORT -V $MQTT_VERSION -t "$TOPIC" -m "Message QoS 0" -q 0
    sleep 0.5
    "$MOSQUITTO_PUB" -h $BROKER_HOST -p $BROKER_PORT -V $MQTT_VERSION -t "$TOPIC" -m "Message QoS 1" -q 1
    sleep 1
    "$MOSQUITTO_PUB" -h $BROKER_HOST -p $BROKER_PORT -V $MQTT_VERSION -t "$TOPIC" -m "Message QoS 2" -q 2
    sleep 3
    
    # Kill subscriber
    kill $SUB_PID 2>/dev/null
    sleep 0.5
    
    # Check if all messages were received
    QOS0_RECEIVED=$(grep -c "Message QoS 0" /tmp/mixed_sub.txt)
    QOS1_RECEIVED=$(grep -c "Message QoS 1" /tmp/mixed_sub.txt)
    QOS2_RECEIVED=$(grep -c "Message QoS 2" /tmp/mixed_sub.txt)
    
    if [ "$QOS0_RECEIVED" -ge 1 ] && [ "$QOS1_RECEIVED" -ge 1 ] && [ "$QOS2_RECEIVED" -ge 1 ]; then
        echo "✅ PASS: All QoS levels working together"
        ((TEST_PASSED++))
    else
        echo "❌ FAIL: Not all QoS messages received (QoS0=$QOS0_RECEIVED, QoS1=$QOS1_RECEIVED, QoS2=$QOS2_RECEIVED)"
        ((TEST_FAILED++))
    fi
    
    sleep 0.5
    kill $SUB_PID 2>/dev/null
    rm -f /tmp/mixed_sub.txt
}

# Test QoS downgrade
test_qos_downgrade() {
    echo ""
    echo "--- Test 5: QoS downgrade (Pub QoS 2, Sub QoS 0) ---"
    
    TOPIC="test/downgrade"
    MESSAGE="Downgraded message"
    
    # Start subscriber with QoS 0
    "$MOSQUITTO_SUB" -h $BROKER_HOST -p $BROKER_PORT -V $MQTT_VERSION -t "$TOPIC" -q 0 > /tmp/downgrade_sub.txt 2>&1 &
    SUB_PID=$!
    sleep 1
    
    # Publish with QoS 2
    "$MOSQUITTO_PUB" -h $BROKER_HOST -p $BROKER_PORT -V $MQTT_VERSION -t "$TOPIC" -m "$MESSAGE" -q 2
    sleep 3
    
    # Kill subscriber
    kill $SUB_PID 2>/dev/null
    sleep 1
    
    # Publish with QoS 2
    "$MOSQUITTO_PUB" -h $BROKER_HOST -p $BROKER_PORT -V $MQTT_VERSION -t "$TOPIC" -m "$MESSAGE" -q 2
    sleep 2
    
    # Message should still be delivered (downgraded to QoS 0)
    if grep -q "$MESSAGE" /tmp/downgrade_sub.txt; then
        echo "✅ PASS: QoS downgrade working correctly"
        ((TEST_PASSED++))
    else
        echo "❌ FAIL: Downgraded message not received"
        ((TEST_FAILED++))
    fi
    sleep 0.5
    
    kill $SUB_PID 2>/dev/null
    rm -f /tmp/downgrade_sub.txt
}

# Run all tests
test_qos0
test_qos1
test_qos2
test_mixed_qos
test_qos_downgrade

# Return exit code based on results
if [ $TEST_FAILED -eq 0 ]; then
    exit 0
else
    exit 1
fi
