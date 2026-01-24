@echo off
REM MT-MQTT Test Runner for Windows

echo ========================================
echo   MT-MQTT Development Test Runner
echo ========================================
echo.

REM Check if broker is running
netstat -an | find "1883" | find "LISTENING" > nul
if %errorlevel% == 0 (
    echo WARNING: Broker already running on port 1883
    echo Using existing broker instance
    set BROKER_RUNNING=1
) else (
    echo Starting broker...
    cargo build --bin mt-mqtt --release
    
    REM Start broker in background
    start /B cargo run --bin mt-mqtt --release > broker_output.log 2>&1
    
    REM Wait for broker to start
    echo Waiting for broker to start...
    timeout /t 3 /nobreak > nul
    
    netstat -an | find "1883" | find "LISTENING" > nul
    if %errorlevel% == 0 (
        echo Broker started successfully
        set BROKER_RUNNING=0
    ) else (
        echo ERROR: Failed to start broker
        type broker_output.log
        exit /b 1
    )
)

echo.
echo ========================================
echo   Running Unit Tests
echo ========================================
cargo test --lib

set UNIT_RESULT=%errorlevel%

echo.
echo ========================================
echo   Running Integration Tests
echo ========================================
cargo test --test integration_tests

set INTEGRATION_RESULT=%errorlevel%

echo.
echo ========================================
echo   Test Summary
echo ========================================

if %UNIT_RESULT% == 0 (
    echo [PASS] Unit Tests
) else (
    echo [FAIL] Unit Tests
)

if %INTEGRATION_RESULT% == 0 (
    echo [PASS] Integration Tests
) else (
    echo [FAIL] Integration Tests
)

echo.

REM Cleanup
if %BROKER_RUNNING% == 0 (
    echo Stopping broker...
    taskkill /F /IM mt-mqtt.exe > nul 2>&1
)

REM Exit with failure if any test failed
if %UNIT_RESULT% neq 0 exit /b 1
if %INTEGRATION_RESULT% neq 0 exit /b 1

exit /b 0
