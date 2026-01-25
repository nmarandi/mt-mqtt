#!/bin/bash

echo "========================================="
echo "MT-MQTT Broker Integration Test Suite"
echo "========================================="
echo ""

# MQTT Protocol version (default: 311 for MQTT 3.1.1, use 5 for MQTT 5.0)
MQTT_VERSION="${1:-311}"
echo "MQTT Protocol Version: $MQTT_VERSION"
echo ""

TOTAL=0
PASSED=0
FAILED=0

# Run all test scripts
for test_script in test_*.sh; do
    if [ -f "$test_script" ]; then
        TOTAL=$((TOTAL + 1))
        
        bash "$test_script" "$MQTT_VERSION"
        
        if [ $? -eq 0 ]; then
            PASSED=$((PASSED + 1))
        else
            FAILED=$((FAILED + 1))
        fi
        
        echo ""
    fi
done

echo "========================================="
echo "Test Summary"
echo "========================================="
echo "Total:  $TOTAL"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "✅ All tests passed!"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi
