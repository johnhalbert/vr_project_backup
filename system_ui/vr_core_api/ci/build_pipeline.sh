#!/bin/bash
# Build Pipeline for VR Headset Project
# Designed for Orange Pi CM5 (16GB variant) platform
# This script handles the build process for all components of the VR headset system

set -e  # Exit immediately if a command exits with a non-zero status

# Configuration
PROJECT_ROOT="/home/ubuntu/orb_slam3_project"
BUILD_DIR="${PROJECT_ROOT}/build"
ARTIFACTS_DIR="${BUILD_DIR}/artifacts"
LOG_DIR="${BUILD_DIR}/logs"
TARGET_PLATFORM="aarch64-unknown-linux-gnu"  # Orange Pi CM5 target
BUILD_TYPE=${1:-"debug"}  # Default to debug build, can be overridden with "release"
VERBOSE=${2:-"false"}     # Default to non-verbose output
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BUILD_LOG="${LOG_DIR}/build_${TIMESTAMP}.log"

# Create necessary directories
mkdir -p "${BUILD_DIR}"
mkdir -p "${ARTIFACTS_DIR}"
mkdir -p "${LOG_DIR}"

# Log function
log() {
    local message="$1"
    local level=${2:-"INFO"}
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[${timestamp}] [${level}] ${message}" | tee -a "${BUILD_LOG}"
}

# Error handler
error_handler() {
    log "Build failed at line $1" "ERROR"
    exit 1
}

trap 'error_handler $LINENO' ERR

# Print build information
log "Starting build pipeline for VR Headset Project"
log "Target platform: ${TARGET_PLATFORM}"
log "Build type: ${BUILD_TYPE}"
log "Project root: ${PROJECT_ROOT}"

# Check for required tools
log "Checking for required tools..."
for tool in cargo rustc gcc g++ cmake make zip unzip; do
    if ! command -v $tool &> /dev/null; then
        log "Required tool not found: $tool" "ERROR"
        exit 1
    fi
done
log "All required tools are available"

# Setup cross-compilation environment if needed
setup_cross_compilation() {
    log "Setting up cross-compilation environment for ${TARGET_PLATFORM}"
    
    # Check if target is already installed
    if ! rustup target list | grep -q "${TARGET_PLATFORM} (installed)"; then
        log "Installing Rust target: ${TARGET_PLATFORM}"
        rustup target add "${TARGET_PLATFORM}"
    else
        log "Rust target already installed: ${TARGET_PLATFORM}"
    fi
    
    # Check for cross-compilation toolchain
    if ! command -v aarch64-linux-gnu-gcc &> /dev/null; then
        log "Installing cross-compilation toolchain"
        apt-get update && apt-get install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
    fi
    
    # Create or update .cargo/config for cross-compilation
    mkdir -p "${PROJECT_ROOT}/.cargo"
    cat > "${PROJECT_ROOT}/.cargo/config" << EOF
[target.${TARGET_PLATFORM}]
linker = "aarch64-linux-gnu-gcc"
EOF
    
    log "Cross-compilation environment setup complete"
}

# Build Core API
build_core_api() {
    local component_dir="${PROJECT_ROOT}/system_ui/vr_core_api"
    local build_flags=""
    
    log "Building Core API component"
    
    if [ "${BUILD_TYPE}" = "release" ]; then
        build_flags="--release"
    fi
    
    if [ "${VERBOSE}" = "true" ]; then
        build_flags="${build_flags} --verbose"
    fi
    
    cd "${component_dir}"
    
    log "Running cargo build for Core API with flags: ${build_flags}"
    cargo build --target "${TARGET_PLATFORM}" ${build_flags}
    
    # Copy artifacts
    local target_dir="target/${TARGET_PLATFORM}/${BUILD_TYPE}"
    mkdir -p "${ARTIFACTS_DIR}/core_api"
    cp -r "${target_dir}/libvr_core_api.rlib" "${ARTIFACTS_DIR}/core_api/"
    
    log "Core API build completed successfully"
}

# Build CLI Interface
build_cli_interface() {
    local component_dir="${PROJECT_ROOT}/system_ui/vr_cli"
    local build_flags=""
    
    log "Building CLI Interface component"
    
    if [ "${BUILD_TYPE}" = "release" ]; then
        build_flags="--release"
    fi
    
    if [ "${VERBOSE}" = "true" ]; then
        build_flags="${build_flags} --verbose"
    fi
    
    cd "${component_dir}"
    
    log "Running cargo build for CLI Interface with flags: ${build_flags}"
    cargo build --target "${TARGET_PLATFORM}" ${build_flags}
    
    # Copy artifacts
    local target_dir="target/${TARGET_PLATFORM}/${BUILD_TYPE}"
    mkdir -p "${ARTIFACTS_DIR}/cli"
    cp -r "${target_dir}/vr_cli" "${ARTIFACTS_DIR}/cli/"
    
    log "CLI Interface build completed successfully"
}

# Build Web Interface
build_web_interface() {
    local component_dir="${PROJECT_ROOT}/system_ui/vr_web"
    local build_flags=""
    
    log "Building Web Interface component"
    
    if [ "${BUILD_TYPE}" = "release" ]; then
        build_flags="--release"
    fi
    
    if [ "${VERBOSE}" = "true" ]; then
        build_flags="${build_flags} --verbose"
    fi
    
    cd "${component_dir}"
    
    # Build backend
    log "Running cargo build for Web Interface backend with flags: ${build_flags}"
    cargo build --target "${TARGET_PLATFORM}" ${build_flags}
    
    # Build frontend
    log "Building Web Interface frontend"
    cd "${component_dir}/frontend/vr_frontend"
    
    if [ -f "package.json" ]; then
        log "Installing frontend dependencies"
        npm install
        
        log "Building frontend"
        if [ "${BUILD_TYPE}" = "release" ]; then
            npm run build
        else
            npm run build:dev
        fi
        
        # Copy artifacts
        mkdir -p "${ARTIFACTS_DIR}/web/frontend"
        cp -r "dist" "${ARTIFACTS_DIR}/web/frontend/"
    else
        log "Frontend package.json not found, skipping frontend build" "WARNING"
    fi
    
    # Copy backend artifacts
    local target_dir="${component_dir}/target/${TARGET_PLATFORM}/${BUILD_TYPE}"
    mkdir -p "${ARTIFACTS_DIR}/web/backend"
    cp -r "${target_dir}/vr_web" "${ARTIFACTS_DIR}/web/backend/"
    
    log "Web Interface build completed successfully"
}

# Package artifacts
package_artifacts() {
    log "Packaging build artifacts"
    
    cd "${BUILD_DIR}"
    
    # Create version file
    echo "Build timestamp: ${TIMESTAMP}" > "${ARTIFACTS_DIR}/version.txt"
    echo "Build type: ${BUILD_TYPE}" >> "${ARTIFACTS_DIR}/version.txt"
    echo "Target platform: ${TARGET_PLATFORM}" >> "${ARTIFACTS_DIR}/version.txt"
    
    # Create archive
    local archive_name="vr_headset_build_${TIMESTAMP}_${BUILD_TYPE}.zip"
    zip -r "${archive_name}" "artifacts"
    
    log "Build artifacts packaged: ${archive_name}"
}

# Main build process
main() {
    log "Starting main build process"
    
    # Setup cross-compilation if needed
    if [ "$(uname -m)" != "aarch64" ]; then
        setup_cross_compilation
    fi
    
    # Build components
    build_core_api
    build_cli_interface
    build_web_interface
    
    # Package artifacts
    package_artifacts
    
    log "Build pipeline completed successfully"
}

# Execute main function
main
