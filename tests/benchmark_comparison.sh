#!/bin/bash
# Benchmark comparison between mt-mqtt and mosquitto broker
# Supports: Linux, macOS, Windows (Git Bash/MSYS2)
# Usage: ./benchmark_comparison.sh [quick|full|mt-mqtt|mosquitto]

set -e

# Detect OS and set paths accordingly
detect_os() {
    case "$(uname -s)" in
        Linux*)
            OS="linux"
            MOSQUITTO_PUB="mosquitto_pub"
            MOSQUITTO_SUB="mosquitto_sub"
            MOSQUITTO_BROKER="mosquitto"
            MT_MQTT_BROKER="./target/release/mt-mqtt"
            KILL_CMD="pkill"
            ;;
        Darwin*)
            OS="macos"
            # Homebrew paths
            if [ -x "/opt/homebrew/bin/mosquitto_pub" ]; then
                MOSQUITTO_PUB="/opt/homebrew/bin/mosquitto_pub"
                MOSQUITTO_SUB="/opt/homebrew/bin/mosquitto_sub"
                MOSQUITTO_BROKER="/opt/homebrew/sbin/mosquitto"
            else
                MOSQUITTO_PUB="/usr/local/bin/mosquitto_pub"
                MOSQUITTO_SUB="/usr/local/bin/mosquitto_sub"
                MOSQUITTO_BROKER="/usr/local/sbin/mosquitto"
            fi
            MT_MQTT_BROKER="./target/release/mt-mqtt"
            KILL_CMD="pkill"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            OS="windows"
            MOSQUITTO_PUB="/c/Program Files/mosquitto/mosquitto_pub.exe"
            MOSQUITTO_SUB="/c/Program Files/mosquitto/mosquitto_sub.exe"
            MOSQUITTO_BROKER="/c/Program Files/mosquitto/mosquitto.exe"
            MT_MQTT_BROKER="./target/release/mt-mqtt.exe"
            KILL_CMD="taskkill"
            ;;
        *)
            echo "Unsupported OS: $(uname -s)"
            exit 1
            ;;
    esac
    
    # Get script directory and project root
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
}

detect_os

# Test parameters
MESSAGE_COUNTS=(10 50 100)
PAYLOAD_SIZES=(10 100 1000)
QOS_LEVELS=(0 1)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "\n${BLUE}============================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}============================================${NC}"
}

print_result() {
    echo -e "${GREEN}$1${NC}"
}

print_warning() {
    echo -e "${YELLOW}$1${NC}"
}

# Kill any existing broker processes
cleanup() {
    if [ "$OS" = "windows" ]; then
        taskkill //F //IM mt-mqtt.exe 2>/dev/null || true
        taskkill //F //IM mosquitto.exe 2>/dev/null || true
    else
        pkill -f "mt-mqtt" 2>/dev/null || true
        pkill -f "mosquitto" 2>/dev/null || true
    fi
    sleep 1
}

# Wait for broker to be ready
wait_for_broker() {
    local max_attempts=10
    local attempt=0
    local timeout_cmd="timeout"
    
    # macOS uses gtimeout from coreutils
    if [ "$OS" = "macos" ] && command -v gtimeout &>/dev/null; then
        timeout_cmd="gtimeout"
    fi
    
    while ! $timeout_cmd 1 "$MOSQUITTO_PUB" -h localhost -p 1883 -t "test/ping" -m "ping" 2>/dev/null; do
        attempt=$((attempt + 1))
        if [ $attempt -ge $max_attempts ]; then
            echo "Broker failed to start!"
            return 1
        fi
        sleep 0.5
    done
    return 0
}

# Benchmark: Message throughput (publish only, no subscriber)
benchmark_publish_throughput() {
    local broker_name=$1
    local count=$2
    local payload_size=$3
    local qos=$4
    
    # Generate payload
    local payload=$(head -c $payload_size /dev/zero | tr '\0' 'X')
    
    local start_ms=$(date +%s%3N)
    
    for i in $(seq 1 $count); do
        "$MOSQUITTO_PUB" -h localhost -p 1883 -t "benchmark/throughput" -m "$payload" -q $qos
    done
    
    local end_ms=$(date +%s%3N)
    local duration_ms=$((end_ms - start_ms))
    local rate=0
    if [ "$duration_ms" -gt 0 ]; then
        rate=$((count * 1000 / duration_ms))
    fi
    
    echo "$duration_ms $rate"
}

# Benchmark: End-to-end latency with subscriber
benchmark_e2e_latency() {
    local broker_name=$1
    local count=$2
    local qos=$3
    
    local received_file=$(mktemp)
    local timeout_cmd="timeout"
    if [ "$OS" = "macos" ] && command -v gtimeout &>/dev/null; then
        timeout_cmd="gtimeout"
    fi
    
    # Start subscriber in background
    $timeout_cmd 60 "$MOSQUITTO_SUB" -h localhost -p 1883 -t "benchmark/latency" -q $qos -C $count > "$received_file" 2>/dev/null &
    local sub_pid=$!
    sleep 0.5
    
    local start_ms=$(date +%s%3N)
    
    # Publish messages
    for i in $(seq 1 $count); do
        "$MOSQUITTO_PUB" -h localhost -p 1883 -t "benchmark/latency" -m "msg$i" -q $qos
    done
    
    # Wait for subscriber to finish
    wait $sub_pid 2>/dev/null || true
    
    local end_ms=$(date +%s%3N)
    local duration_ms=$((end_ms - start_ms))
    local received=$(wc -l < "$received_file" | tr -d ' ')
    
    rm -f "$received_file"
    
    echo "$duration_ms $received"
}

# Benchmark: Connection rate
benchmark_connection_rate() {
    local broker_name=$1
    local count=$2
    
    local start_ms=$(date +%s%3N)
    
    for i in $(seq 1 $count); do
        "$MOSQUITTO_PUB" -h localhost -p 1883 -t "benchmark/conn" -m "x" -q 0
    done
    
    local end_ms=$(date +%s%3N)
    local duration_ms=$((end_ms - start_ms))
    local rate=0
    if [ "$duration_ms" -gt 0 ]; then
        rate=$((count * 1000 / duration_ms))
    fi
    
    echo "$duration_ms $rate"
}

# Store benchmark results for comparison table
declare -A RESULTS

# Run benchmarks and store results
run_benchmarks_with_storage() {
    local broker_name=$1
    
    print_header "Benchmarking $broker_name"
    
    # Connection rate benchmark
    echo -e "\n${YELLOW}Connection Rate (connect + publish + disconnect per iteration):${NC}"
    printf "%-15s %-15s %-15s\n" "Connections" "Time (ms)" "Rate (conn/s)"
    printf "%-15s %-15s %-15s\n" "-----------" "---------" "-------------"
    
    for count in 10 50 100; do
        result=$(benchmark_connection_rate "$broker_name" $count)
        duration=$(echo $result | cut -d' ' -f1)
        rate=$(echo $result | cut -d' ' -f2)
        printf "%-15s %-15s %-15s\n" "$count" "$duration" "$rate"
        RESULTS["${broker_name}_conn_${count}"]="$duration"
        RESULTS["${broker_name}_conn_${count}_rate"]="$rate"
    done
    
    # Publish throughput benchmark
    echo -e "\n${YELLOW}Publish Throughput (QoS 0):${NC}"
    printf "%-10s %-15s %-15s %-15s\n" "Messages" "Payload" "Time (ms)" "Rate (msg/s)"
    printf "%-10s %-15s %-15s %-15s\n" "--------" "-------" "---------" "------------"
    
    for count in "${MESSAGE_COUNTS[@]}"; do
        for size in "${PAYLOAD_SIZES[@]}"; do
            result=$(benchmark_publish_throughput "$broker_name" $count $size 0)
            duration=$(echo $result | cut -d' ' -f1)
            rate=$(echo $result | cut -d' ' -f2)
            printf "%-10s %-15s %-15s %-15s\n" "$count" "${size}B" "$duration" "$rate"
            RESULTS["${broker_name}_pub_${count}_${size}"]="$duration"
            RESULTS["${broker_name}_pub_${count}_${size}_rate"]="$rate"
        done
    done
    
    # End-to-end latency benchmark
    echo -e "\n${YELLOW}End-to-End Latency (publish to subscriber receive):${NC}"
    printf "%-10s %-10s %-15s %-15s\n" "Messages" "QoS" "Time (ms)" "Received"
    printf "%-10s %-10s %-15s %-15s\n" "--------" "---" "---------" "--------"
    
    for count in 10 50; do
        for qos in "${QOS_LEVELS[@]}"; do
            result=$(benchmark_e2e_latency "$broker_name" $count $qos)
            duration=$(echo $result | cut -d' ' -f1)
            received=$(echo $result | cut -d' ' -f2)
            printf "%-10s %-10s %-15s %-15s\n" "$count" "$qos" "$duration" "$received"
            RESULTS["${broker_name}_e2e_${count}_qos${qos}"]="$duration"
        done
    done
}

# Print comparison table
print_comparison_table() {
    print_header "Comparison Summary"
    
    echo -e "\n${YELLOW}=== Connection Rate (ms / rate) ===${NC}"
    printf "%-20s %-20s %-20s %-15s\n" "Test" "mt-mqtt" "mosquitto" "Winner"
    printf "%-20s %-20s %-20s %-15s\n" "----" "-------" "---------" "------"
    
    for count in 10 50 100; do
        local mt="${RESULTS[mt-mqtt_conn_${count}]:-N/A}"
        local mo="${RESULTS[mosquitto_conn_${count}]:-N/A}"
        local mt_rate="${RESULTS[mt-mqtt_conn_${count}_rate]:-0}"
        local mo_rate="${RESULTS[mosquitto_conn_${count}_rate]:-0}"
        local winner=""
        if [ "$mt" != "N/A" ] && [ "$mo" != "N/A" ]; then
            if [ "$mt" -lt "$mo" ]; then
                winner="${GREEN}mt-mqtt${NC}"
            elif [ "$mo" -lt "$mt" ]; then
                winner="${YELLOW}mosquitto${NC}"
            else
                winner="tie"
            fi
        fi
        printf "%-20s %-20s %-20s " "${count} connections" "${mt}ms (${mt_rate}/s)" "${mo}ms (${mo_rate}/s)"
        echo -e "$winner"
    done
    
    echo -e "\n${YELLOW}=== Publish Throughput QoS 0 (ms / rate) ===${NC}"
    printf "%-20s %-20s %-20s %-15s\n" "Test" "mt-mqtt" "mosquitto" "Winner"
    printf "%-20s %-20s %-20s %-15s\n" "----" "-------" "---------" "------"
    
    for count in 100; do
        for size in 10 1000; do
            local mt="${RESULTS[mt-mqtt_pub_${count}_${size}]:-N/A}"
            local mo="${RESULTS[mosquitto_pub_${count}_${size}]:-N/A}"
            local mt_rate="${RESULTS[mt-mqtt_pub_${count}_${size}_rate]:-0}"
            local mo_rate="${RESULTS[mosquitto_pub_${count}_${size}_rate]:-0}"
            local winner=""
            if [ "$mt" != "N/A" ] && [ "$mo" != "N/A" ]; then
                if [ "$mt" -lt "$mo" ]; then
                    winner="${GREEN}mt-mqtt${NC}"
                elif [ "$mo" -lt "$mt" ]; then
                    winner="${YELLOW}mosquitto${NC}"
                else
                    winner="tie"
                fi
            fi
            printf "%-20s %-20s %-20s " "${count}msg ${size}B" "${mt}ms (${mt_rate}/s)" "${mo}ms (${mo_rate}/s)"
            echo -e "$winner"
        done
    done
    
    echo -e "\n${YELLOW}=== End-to-End Latency (ms) ===${NC}"
    printf "%-20s %-20s %-20s %-15s\n" "Test" "mt-mqtt" "mosquitto" "Winner"
    printf "%-20s %-20s %-20s %-15s\n" "----" "-------" "---------" "------"
    
    for count in 50; do
        for qos in 0 1; do
            local mt="${RESULTS[mt-mqtt_e2e_${count}_qos${qos}]:-N/A}"
            local mo="${RESULTS[mosquitto_e2e_${count}_qos${qos}]:-N/A}"
            local winner=""
            if [ "$mt" != "N/A" ] && [ "$mo" != "N/A" ]; then
                if [ "$mt" -lt "$mo" ]; then
                    winner="${GREEN}mt-mqtt${NC}"
                elif [ "$mo" -lt "$mt" ]; then
                    winner="${YELLOW}mosquitto${NC}"
                else
                    winner="tie"
                fi
            fi
            printf "%-20s %-20s %-20s " "E2E ${count}msg QoS${qos}" "${mt}ms" "${mo}ms"
            echo -e "$winner"
        done
    done
    
    echo ""
}

# Main comparison function
run_comparison() {
    local test_target=${1:-both}
    
    print_header "MQTT Broker Benchmark Comparison"
    echo "Date: $(date)"
    echo "Host: $(hostname)"
    echo "OS: $OS"
    
    if [ "$test_target" = "mt-mqtt" ] || [ "$test_target" = "both" ]; then
        cleanup
        print_warning "Starting mt-mqtt broker..."
        cd "$PROJECT_ROOT"
        RUST_LOG=error "$MT_MQTT_BROKER" &
        sleep 2
        
        if wait_for_broker; then
            run_benchmarks_with_storage "mt-mqtt"
        else
            echo "Failed to start mt-mqtt broker"
        fi
        
        cleanup
    fi
    
    if [ "$test_target" = "mosquitto" ] || [ "$test_target" = "both" ]; then
        cleanup
        print_warning "Starting mosquitto broker..."
        "$MOSQUITTO_BROKER" &
        sleep 2
        
        if wait_for_broker; then
            run_benchmarks_with_storage "mosquitto"
        else
            echo "Failed to start mosquitto broker"
        fi
        
        cleanup
    fi
    
    # Print comparison table if both brokers were tested
    if [ "$test_target" = "both" ]; then
        print_comparison_table
    fi
    
    print_header "Benchmark Complete"
}

# Quick comparison (fewer iterations for fast feedback)
run_quick_comparison() {
    print_header "Quick MQTT Broker Comparison"
    echo "OS: $OS"
    echo "Running 50 messages test..."
    
    # Test mt-mqtt
    cleanup
    print_warning "Testing mt-mqtt..."
    cd "$PROJECT_ROOT"
    RUST_LOG=error "$MT_MQTT_BROKER" &
    sleep 2
    
    local mt_mqtt_ms=0
    local mt_mqtt_rate=0
    if wait_for_broker; then
        mt_mqtt_ms=$(benchmark_connection_rate "mt-mqtt" 50 | cut -d' ' -f1)
        if [ "$mt_mqtt_ms" -gt 0 ]; then
            mt_mqtt_rate=$((50 * 1000 / mt_mqtt_ms))
        fi
        echo -e "mt-mqtt:    ${GREEN}${mt_mqtt_ms}ms${NC} (${mt_mqtt_rate} msg/s)"
    fi
    
    cleanup
    
    # Test mosquitto
    print_warning "Testing mosquitto..."
    "$MOSQUITTO_BROKER" &
    sleep 2
    
    local mosquitto_ms=0
    local mosquitto_rate=0
    if wait_for_broker; then
        mosquitto_ms=$(benchmark_connection_rate "mosquitto" 50 | cut -d' ' -f1)
        if [ "$mosquitto_ms" -gt 0 ]; then
            mosquitto_rate=$((50 * 1000 / mosquitto_ms))
        fi
        echo -e "mosquitto:  ${GREEN}${mosquitto_ms}ms${NC} (${mosquitto_rate} msg/s)"
    fi
    
    cleanup
    
    # Summary
    print_header "Summary"
    printf "%-15s %-15s %-15s\n" "Broker" "Time (ms)" "Rate (msg/s)"
    printf "%-15s %-15s %-15s\n" "------" "---------" "-----------"
    printf "%-15s %-15s %-15s\n" "mt-mqtt" "$mt_mqtt_ms" "$mt_mqtt_rate"
    printf "%-15s %-15s %-15s\n" "mosquitto" "$mosquitto_ms" "$mosquitto_rate"
    
    if [ "$mt_mqtt_ms" -gt 0 ] && [ "$mosquitto_ms" -gt 0 ]; then
        if [ "$mt_mqtt_ms" -lt "$mosquitto_ms" ]; then
            local diff=$((mosquitto_ms - mt_mqtt_ms))
            local pct=$((diff * 100 / mosquitto_ms))
            echo -e "\n${GREEN}mt-mqtt is ${pct}% faster!${NC}"
        elif [ "$mosquitto_ms" -lt "$mt_mqtt_ms" ]; then
            local diff=$((mt_mqtt_ms - mosquitto_ms))
            local pct=$((diff * 100 / mt_mqtt_ms))
            echo -e "\n${YELLOW}mosquitto is ${pct}% faster${NC}"
        else
            echo -e "\n${GREEN}Both brokers perform equally!${NC}"
        fi
    fi
}

# Parse command line arguments
case "${1:-quick}" in
    quick)
        run_quick_comparison
        ;;
    full)
        run_comparison both
        ;;
    mt-mqtt)
        run_comparison mt-mqtt
        ;;
    mosquitto)
        run_comparison mosquitto
        ;;
    *)
        echo "Usage: $0 [quick|full|mt-mqtt|mosquitto]"
        echo "  quick     - Fast comparison (50 messages)"
        echo "  full      - Full benchmark suite"
        echo "  mt-mqtt   - Benchmark mt-mqtt only"
        echo "  mosquitto - Benchmark mosquitto only"
        exit 1
        ;;
esac
