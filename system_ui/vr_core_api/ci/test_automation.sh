#!/bin/bash
# Test Automation for VR Headset Project
# Designed for Orange Pi CM5 (16GB variant) platform
# This script handles automated testing for all components of the VR headset system

set -e  # Exit immediately if a command exits with a non-zero status

# Configuration
PROJECT_ROOT="/home/ubuntu/orb_slam3_project"
BUILD_DIR="${PROJECT_ROOT}/build"
TEST_RESULTS_DIR="${BUILD_DIR}/test_results"
LOG_DIR="${BUILD_DIR}/logs"
TARGET_PLATFORM="aarch64-unknown-linux-gnu"  # Orange Pi CM5 target
TEST_TYPE=${1:-"all"}  # Default to all tests, can be: unit, integration, system, performance, security
HARDWARE_MODE=${2:-"auto"}  # auto, hardware, simulation
VERBOSE=${3:-"false"}  # Default to non-verbose output
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
TEST_LOG="${LOG_DIR}/test_${TIMESTAMP}.log"
TEST_REPORT="${TEST_RESULTS_DIR}/test_report_${TIMESTAMP}.json"
TEST_SUMMARY="${TEST_RESULTS_DIR}/test_summary_${TIMESTAMP}.txt"

# Create necessary directories
mkdir -p "${BUILD_DIR}"
mkdir -p "${TEST_RESULTS_DIR}"
mkdir -p "${LOG_DIR}"

# Log function
log() {
    local message="$1"
    local level=${2:-"INFO"}
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[${timestamp}] [${level}] ${message}" | tee -a "${TEST_LOG}"
}

# Error handler
error_handler() {
    log "Test automation failed at line $1" "ERROR"
    exit 1
}

trap 'error_handler $LINENO' ERR

# Print test information
log "Starting test automation for VR Headset Project"
log "Target platform: ${TARGET_PLATFORM}"
log "Test type: ${TEST_TYPE}"
log "Hardware mode: ${HARDWARE_MODE}"
log "Project root: ${PROJECT_ROOT}"

# Detect hardware
detect_hardware() {
    log "Detecting hardware environment"
    
    if [ "${HARDWARE_MODE}" = "hardware" ]; then
        log "Hardware mode explicitly set to 'hardware'"
        return 0
    elif [ "${HARDWARE_MODE}" = "simulation" ]; then
        log "Hardware mode explicitly set to 'simulation'"
        return 1
    fi
    
    # Auto-detect hardware
    if [ "$(uname -m)" = "aarch64" ] && grep -q "Orange Pi" /proc/device-tree/model 2>/dev/null; then
        log "Detected Orange Pi hardware"
        return 0
    else
        log "Orange Pi hardware not detected, using simulation mode"
        return 1
    fi
}

# Setup test environment
setup_test_environment() {
    log "Setting up test environment"
    
    # Set environment variables for testing
    export VR_TEST_MODE="${TEST_TYPE}"
    export VR_TEST_LOG="${TEST_LOG}"
    export VR_TEST_REPORT="${TEST_REPORT}"
    
    if detect_hardware; then
        export VR_HARDWARE_MODE="hardware"
        log "Testing in hardware mode"
    else
        export VR_HARDWARE_MODE="simulation"
        log "Testing in simulation mode"
    fi
    
    # Create initial test report structure
    cat > "${TEST_REPORT}" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "test_type": "${TEST_TYPE}",
  "hardware_mode": "${VR_HARDWARE_MODE}",
  "results": {
    "total": 0,
    "passed": 0,
    "failed": 0,
    "skipped": 0
  },
  "components": []
}
EOF
    
    log "Test environment setup complete"
}

# Run unit tests
run_unit_tests() {
    local component_dir="${PROJECT_ROOT}/system_ui/vr_core_api"
    local test_flags=""
    
    log "Running unit tests"
    
    if [ "${VERBOSE}" = "true" ]; then
        test_flags="--verbose"
    fi
    
    cd "${component_dir}"
    
    # Run tests using the testing framework
    log "Executing unit tests with flags: ${test_flags}"
    
    # Use our custom testing framework
    RUST_BACKTRACE=1 cargo test --lib "testing::unit_tests" -- --nocapture ${test_flags} | tee -a "${TEST_LOG}"
    
    # Parse test results and update report
    local total=$(grep -c "test result:" "${TEST_LOG}" || echo "0")
    local passed=$(grep "test result:" "${TEST_LOG}" | grep -o "[0-9]\\+ passed" | awk '{sum += $1} END {print sum}' || echo "0")
    local failed=$(grep "test result:" "${TEST_LOG}" | grep -o "[0-9]\\+ failed" | awk '{sum += $1} END {print sum}' || echo "0")
    
    # Update test report
    jq ".results.total += ${total} | .results.passed += ${passed} | .results.failed += ${failed} | .components += [{\"name\": \"core_api\", \"test_type\": \"unit\", \"total\": ${total}, \"passed\": ${passed}, \"failed\": ${failed}}]" "${TEST_REPORT}" > "${TEST_REPORT}.tmp" && mv "${TEST_REPORT}.tmp" "${TEST_REPORT}"
    
    if [ "${failed}" -gt 0 ]; then
        log "Unit tests completed with ${failed} failures" "WARNING"
    else
        log "Unit tests completed successfully"
    fi
}

# Run integration tests
run_integration_tests() {
    local component_dir="${PROJECT_ROOT}/system_ui/vr_core_api"
    local test_flags=""
    
    log "Running integration tests"
    
    if [ "${VERBOSE}" = "true" ]; then
        test_flags="--verbose"
    fi
    
    cd "${component_dir}"
    
    # Run tests using the testing framework
    log "Executing integration tests with flags: ${test_flags}"
    
    # Use our custom testing framework
    RUST_BACKTRACE=1 cargo test --lib "testing::integration_tests" -- --nocapture ${test_flags} | tee -a "${TEST_LOG}"
    
    # Parse test results and update report
    local total=$(grep -c "test result:" "${TEST_LOG}" || echo "0")
    local passed=$(grep "test result:" "${TEST_LOG}" | grep -o "[0-9]\\+ passed" | awk '{sum += $1} END {print sum}' || echo "0")
    local failed=$(grep "test result:" "${TEST_LOG}" | grep -o "[0-9]\\+ failed" | awk '{sum += $1} END {print sum}' || echo "0")
    
    # Update test report
    jq ".results.total += ${total} | .results.passed += ${passed} | .results.failed += ${failed} | .components += [{\"name\": \"core_api\", \"test_type\": \"integration\", \"total\": ${total}, \"passed\": ${passed}, \"failed\": ${failed}}]" "${TEST_REPORT}" > "${TEST_REPORT}.tmp" && mv "${TEST_REPORT}.tmp" "${TEST_REPORT}"
    
    if [ "${failed}" -gt 0 ]; then
        log "Integration tests completed with ${failed} failures" "WARNING"
    else
        log "Integration tests completed successfully"
    fi
}

# Run system tests
run_system_tests() {
    local component_dir="${PROJECT_ROOT}/system_ui/vr_core_api"
    local test_flags=""
    
    log "Running system tests"
    
    if [ "${VERBOSE}" = "true" ]; then
        test_flags="--verbose"
    fi
    
    cd "${component_dir}"
    
    # Run tests using the testing framework
    log "Executing system tests with flags: ${test_flags}"
    
    # Use our custom testing framework
    RUST_BACKTRACE=1 cargo test --lib "testing::system_tests" -- --nocapture ${test_flags} | tee -a "${TEST_LOG}"
    
    # Parse test results and update report
    local total=$(grep -c "test result:" "${TEST_LOG}" || echo "0")
    local passed=$(grep "test result:" "${TEST_LOG}" | grep -o "[0-9]\\+ passed" | awk '{sum += $1} END {print sum}' || echo "0")
    local failed=$(grep "test result:" "${TEST_LOG}" | grep -o "[0-9]\\+ failed" | awk '{sum += $1} END {print sum}' || echo "0")
    
    # Update test report
    jq ".results.total += ${total} | .results.passed += ${passed} | .results.failed += ${failed} | .components += [{\"name\": \"core_api\", \"test_type\": \"system\", \"total\": ${total}, \"passed\": ${passed}, \"failed\": ${failed}}]" "${TEST_REPORT}" > "${TEST_REPORT}.tmp" && mv "${TEST_REPORT}.tmp" "${TEST_REPORT}"
    
    if [ "${failed}" -gt 0 ]; then
        log "System tests completed with ${failed} failures" "WARNING"
    else
        log "System tests completed successfully"
    fi
}

# Run performance tests
run_performance_tests() {
    local component_dir="${PROJECT_ROOT}/system_ui/vr_core_api"
    local test_flags=""
    
    log "Running performance tests"
    
    if [ "${VERBOSE}" = "true" ]; then
        test_flags="--verbose"
    fi
    
    cd "${component_dir}"
    
    # Run tests using the testing framework
    log "Executing performance tests with flags: ${test_flags}"
    
    # Use our custom testing framework
    RUST_BACKTRACE=1 cargo test --lib "testing::performance_tests" -- --nocapture ${test_flags} | tee -a "${TEST_LOG}"
    
    # Parse test results and update report
    local total=$(grep -c "test result:" "${TEST_LOG}" || echo "0")
    local passed=$(grep "test result:" "${TEST_LOG}" | grep -o "[0-9]\\+ passed" | awk '{sum += $1} END {print sum}' || echo "0")
    local failed=$(grep "test result:" "${TEST_LOG}" | grep -o "[0-9]\\+ failed" | awk '{sum += $1} END {print sum}' || echo "0")
    
    # Update test report
    jq ".results.total += ${total} | .results.passed += ${passed} | .results.failed += ${failed} | .components += [{\"name\": \"core_api\", \"test_type\": \"performance\", \"total\": ${total}, \"passed\": ${passed}, \"failed\": ${failed}}]" "${TEST_REPORT}" > "${TEST_REPORT}.tmp" && mv "${TEST_REPORT}.tmp" "${TEST_REPORT}"
    
    if [ "${failed}" -gt 0 ]; then
        log "Performance tests completed with ${failed} failures" "WARNING"
    else
        log "Performance tests completed successfully"
    fi
}

# Run security tests
run_security_tests() {
    local component_dir="${PROJECT_ROOT}/system_ui/vr_core_api"
    local test_flags=""
    
    log "Running security tests"
    
    if [ "${VERBOSE}" = "true" ]; then
        test_flags="--verbose"
    fi
    
    cd "${component_dir}"
    
    # Run tests using the testing framework
    log "Executing security tests with flags: ${test_flags}"
    
    # Use our custom testing framework
    RUST_BACKTRACE=1 cargo test --lib "testing::security_tests" -- --nocapture ${test_flags} | tee -a "${TEST_LOG}"
    
    # Parse test results and update report
    local total=$(grep -c "test result:" "${TEST_LOG}" || echo "0")
    local passed=$(grep "test result:" "${TEST_LOG}" | grep -o "[0-9]\\+ passed" | awk '{sum += $1} END {print sum}' || echo "0")
    local failed=$(grep "test result:" "${TEST_LOG}" | grep -o "[0-9]\\+ failed" | awk '{sum += $1} END {print sum}' || echo "0")
    
    # Update test report
    jq ".results.total += ${total} | .results.passed += ${passed} | .results.failed += ${failed} | .components += [{\"name\": \"core_api\", \"test_type\": \"security\", \"total\": ${total}, \"passed\": ${passed}, \"failed\": ${failed}}]" "${TEST_REPORT}" > "${TEST_REPORT}.tmp" && mv "${TEST_REPORT}.tmp" "${TEST_REPORT}"
    
    if [ "${failed}" -gt 0 ]; then
        log "Security tests completed with ${failed} failures" "WARNING"
    else
        log "Security tests completed successfully"
    fi
}

# Generate test summary
generate_test_summary() {
    log "Generating test summary"
    
    # Extract summary information from the test report
    local total=$(jq '.results.total' "${TEST_REPORT}")
    local passed=$(jq '.results.passed' "${TEST_REPORT}")
    local failed=$(jq '.results.failed' "${TEST_REPORT}")
    local skipped=$(jq '.results.skipped' "${TEST_REPORT}")
    local pass_rate=0
    
    if [ "${total}" -gt 0 ]; then
        pass_rate=$(echo "scale=2; ${passed} * 100 / ${total}" | bc)
    fi
    
    # Create summary file
    cat > "${TEST_SUMMARY}" << EOF
VR Headset Project - Test Summary
=================================
Timestamp: $(date -Iseconds)
Test Type: ${TEST_TYPE}
Hardware Mode: ${VR_HARDWARE_MODE}

Summary:
--------
Total Tests: ${total}
Passed: ${passed}
Failed: ${failed}
Skipped: ${skipped}
Pass Rate: ${pass_rate}%

Component Details:
-----------------
EOF
    
    # Add component details to summary
    jq -r '.components[] | "Component: \(.name)\nTest Type: \(.test_type)\nTotal: \(.total)\nPassed: \(.passed)\nFailed: \(.failed)\n"' "${TEST_REPORT}" >> "${TEST_SUMMARY}"
    
    # Add final status
    if [ "${failed}" -gt 0 ]; then
        echo -e "\nTest Status: FAILED" >> "${TEST_SUMMARY}"
        log "Tests completed with failures" "WARNING"
    else
        echo -e "\nTest Status: PASSED" >> "${TEST_SUMMARY}"
        log "All tests passed successfully"
    fi
    
    log "Test summary generated: ${TEST_SUMMARY}"
}

# Main test process
main() {
    log "Starting main test process"
    
    # Setup test environment
    setup_test_environment
    
    # Run tests based on test type
    if [ "${TEST_TYPE}" = "all" ] || [ "${TEST_TYPE}" = "unit" ]; then
        run_unit_tests
    fi
    
    if [ "${TEST_TYPE}" = "all" ] || [ "${TEST_TYPE}" = "integration" ]; then
        run_integration_tests
    fi
    
    if [ "${TEST_TYPE}" = "all" ] || [ "${TEST_TYPE}" = "system" ]; then
        run_system_tests
    fi
    
    if [ "${TEST_TYPE}" = "all" ] || [ "${TEST_TYPE}" = "performance" ]; then
        run_performance_tests
    fi
    
    if [ "${TEST_TYPE}" = "all" ] || [ "${TEST_TYPE}" = "security" ]; then
        run_security_tests
    fi
    
    # Generate test summary
    generate_test_summary
    
    log "Test automation completed"
    
    # Return non-zero exit code if any tests failed
    local failed=$(jq '.results.failed' "${TEST_REPORT}")
    if [ "${failed}" -gt 0 ]; then
        log "Exiting with status code 1 due to test failures" "WARNING"
        exit 1
    fi
}

# Execute main function
main
