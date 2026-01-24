#!/bin/bash

echo "========================================="
echo "MT-MQTT Broker Integration Test Suite"
echo "========================================="
echo ""

# Check if broker is running (Windows-compatible)
# Try using timeout command which is available in Git Bash on Windows
if timeout 1 bash -c "</dev/tcp/localhost/1883" 2>/dev/null; then
    echo "✅ Broker is running on port 1883"
elif command -v nc >/dev/null 2>&1 && nc -z localhost 1883 2>/dev/null; then
    echo "✅ Broker is running on port 1883"
else
    echo "⚠️  Warning: Could not verify broker status"
    echo "   Assuming broker is running and continuing tests..."
    echo "   If tests fail, make sure broker is running: cargo run --bin mt-mqtt"
fi

echo ""

TOTAL=0
PASSED=0
FAILED=0

# Run all test scripts
for test_script in test_*.sh; do
    if [ -f "$test_script" ]; then
        TOTAL=$((TOTAL + 1))
        
        bash "$test_script"
        
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
