#!/bin/bash

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================"
echo "  MT-MQTT Development Test Runner"
echo "========================================"
echo ""

# Check if broker is running
echo "Checking if broker is running..."
if nc -z localhost 1883 2>/dev/null; then
    echo -e "${YELLOW}⚠️  Broker already running on port 1883${NC}"
    echo "Using existing broker instance"
    BROKER_RUNNING=1
else
    echo "Starting broker..."
    cargo build --bin mt-mqtt --release
    
    # Start broker in background
    cargo run --bin mt-mqtt --release > /tmp/broker_output.log 2>&1 &
    BROKER_PID=$!
    
    # Wait for broker to start
    echo -n "Waiting for broker to start"
    for i in {1..10}; do
        sleep 1
        echo -n "."
        if nc -z localhost 1883 2>/dev/null; then
            echo ""
            echo -e "${GREEN}✅ Broker started${NC}"
            break
        fi
    done
    
    if ! nc -z localhost 1883 2>/dev/null; then
        echo -e "${RED}❌ Failed to start broker${NC}"
        cat /tmp/broker_output.log
        exit 1
    fi
    
    BROKER_RUNNING=0
fi

echo ""
echo "========================================"
echo "  Running Unit Tests"
echo "========================================"
cargo test --lib

UNIT_RESULT=$?

echo ""
echo "========================================"
echo "  Running Integration Tests"
echo "========================================"
cargo test --test integration_tests

INTEGRATION_RESULT=$?

echo ""
echo "========================================"
echo "  Running Shell-based E2E Tests"
echo "========================================"
bash tests/run_all_tests.sh

E2E_RESULT=$?

# Cleanup
if [ $BROKER_RUNNING -eq 0 ]; then
    echo ""
    echo "Stopping broker (PID: $BROKER_PID)..."
    kill $BROKER_PID 2>/dev/null
    sleep 1
fi

# Summary
echo ""
echo "========================================"
echo "  Test Summary"
echo "========================================"

if [ $UNIT_RESULT -eq 0 ]; then
    echo -e "${GREEN}✅ Unit Tests: PASSED${NC}"
else
    echo -e "${RED}❌ Unit Tests: FAILED${NC}"
fi

if [ $INTEGRATION_RESULT -eq 0 ]; then
    echo -e "${GREEN}✅ Integration Tests: PASSED${NC}"
else
    echo -e "${RED}❌ Integration Tests: FAILED${NC}"
fi

if [ $E2E_RESULT -eq 0 ]; then
    echo -e "${GREEN}✅ E2E Tests: PASSED${NC}"
else
    echo -e "${RED}❌ E2E Tests: FAILED${NC}"
fi

echo ""

# Exit with failure if any test failed
if [ $UNIT_RESULT -ne 0 ] || [ $INTEGRATION_RESULT -ne 0 ] || [ $E2E_RESULT -ne 0 ]; then
    exit 1
fi

exit 0
