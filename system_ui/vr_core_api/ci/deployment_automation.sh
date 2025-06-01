#!/bin/bash
# Deployment Automation for VR Headset Project
# Designed for Orange Pi CM5 (16GB variant) platform
# This script handles automated deployment for all components of the VR headset system

set -e  # Exit immediately if a command exits with a non-zero status

# Configuration
PROJECT_ROOT="/home/ubuntu/orb_slam3_project"
BUILD_DIR="${PROJECT_ROOT}/build"
ARTIFACTS_DIR="${BUILD_DIR}/artifacts"
DEPLOYMENT_DIR="${BUILD_DIR}/deployment"
LOG_DIR="${BUILD_DIR}/logs"
TARGET_PLATFORM="aarch64-unknown-linux-gnu"  # Orange Pi CM5 target
DEPLOYMENT_ENV=${1:-"staging"}  # Default to staging, can be: dev, staging, production
DEPLOYMENT_TYPE=${2:-"full"}  # Default to full, can be: full, core_api, cli, web
VERBOSE=${3:-"false"}  # Default to non-verbose output
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
DEPLOYMENT_LOG="${LOG_DIR}/deployment_${TIMESTAMP}.log"
DEPLOYMENT_REPORT="${DEPLOYMENT_DIR}/deployment_report_${TIMESTAMP}.json"

# Target device configuration
TARGET_USER="orangepi"
TARGET_HOST="orangepi-cm5"  # Can be overridden by environment variable TARGET_HOST
TARGET_PORT="22"
TARGET_DIR="/opt/vr_headset"
TARGET_BACKUP_DIR="/opt/vr_headset/backups"

# Override target host if environment variable is set
if [ -n "${TARGET_HOST_OVERRIDE}" ]; then
    TARGET_HOST="${TARGET_HOST_OVERRIDE}"
fi

# Create necessary directories
mkdir -p "${BUILD_DIR}"
mkdir -p "${DEPLOYMENT_DIR}"
mkdir -p "${LOG_DIR}"

# Log function
log() {
    local message="$1"
    local level=${2:-"INFO"}
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[${timestamp}] [${level}] ${message}" | tee -a "${DEPLOYMENT_LOG}"
}

# Error handler
error_handler() {
    log "Deployment automation failed at line $1" "ERROR"
    exit 1
}

trap 'error_handler $LINENO' ERR

# Print deployment information
log "Starting deployment automation for VR Headset Project"
log "Target platform: ${TARGET_PLATFORM}"
log "Deployment environment: ${DEPLOYMENT_ENV}"
log "Deployment type: ${DEPLOYMENT_TYPE}"
log "Target host: ${TARGET_HOST}"
log "Project root: ${PROJECT_ROOT}"

# Check for required tools
log "Checking for required tools..."
for tool in ssh scp rsync zip unzip jq; do
    if ! command -v $tool &> /dev/null; then
        log "Required tool not found: $tool" "ERROR"
        exit 1
    fi
done
log "All required tools are available"

# Check SSH connection to target
check_ssh_connection() {
    log "Checking SSH connection to target: ${TARGET_USER}@${TARGET_HOST}:${TARGET_PORT}"
    
    if ssh -q -o BatchMode=yes -o ConnectTimeout=5 -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" exit; then
        log "SSH connection successful"
        return 0
    else
        log "SSH connection failed" "ERROR"
        return 1
    fi
}

# Prepare deployment package
prepare_deployment_package() {
    log "Preparing deployment package for ${DEPLOYMENT_TYPE} deployment"
    
    local package_dir="${DEPLOYMENT_DIR}/package_${TIMESTAMP}"
    mkdir -p "${package_dir}"
    
    # Copy artifacts based on deployment type
    if [ "${DEPLOYMENT_TYPE}" = "full" ] || [ "${DEPLOYMENT_TYPE}" = "core_api" ]; then
        log "Including Core API in deployment package"
        mkdir -p "${package_dir}/core_api"
        cp -r "${ARTIFACTS_DIR}/core_api/"* "${package_dir}/core_api/" || log "No Core API artifacts found" "WARNING"
    fi
    
    if [ "${DEPLOYMENT_TYPE}" = "full" ] || [ "${DEPLOYMENT_TYPE}" = "cli" ]; then
        log "Including CLI Interface in deployment package"
        mkdir -p "${package_dir}/cli"
        cp -r "${ARTIFACTS_DIR}/cli/"* "${package_dir}/cli/" || log "No CLI artifacts found" "WARNING"
    fi
    
    if [ "${DEPLOYMENT_TYPE}" = "full" ] || [ "${DEPLOYMENT_TYPE}" = "web" ]; then
        log "Including Web Interface in deployment package"
        mkdir -p "${package_dir}/web"
        cp -r "${ARTIFACTS_DIR}/web/"* "${package_dir}/web/" || log "No Web Interface artifacts found" "WARNING"
    fi
    
    # Create deployment metadata
    cat > "${package_dir}/deployment.json" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "environment": "${DEPLOYMENT_ENV}",
  "type": "${DEPLOYMENT_TYPE}",
  "version": "${TIMESTAMP}",
  "components": [
EOF

    # Add component information
    if [ "${DEPLOYMENT_TYPE}" = "full" ] || [ "${DEPLOYMENT_TYPE}" = "core_api" ]; then
        echo '    {"name": "core_api", "version": "'${TIMESTAMP}'"},' >> "${package_dir}/deployment.json"
    fi
    
    if [ "${DEPLOYMENT_TYPE}" = "full" ] || [ "${DEPLOYMENT_TYPE}" = "cli" ]; then
        echo '    {"name": "cli", "version": "'${TIMESTAMP}'"},' >> "${package_dir}/deployment.json"
    fi
    
    if [ "${DEPLOYMENT_TYPE}" = "full" ] || [ "${DEPLOYMENT_TYPE}" = "web" ]; then
        echo '    {"name": "web", "version": "'${TIMESTAMP}'"}'  >> "${package_dir}/deployment.json"
    fi
    
    # Close the JSON
    echo '  ]' >> "${package_dir}/deployment.json"
    echo '}' >> "${package_dir}/deployment.json"
    
    # Create deployment package
    local package_file="${DEPLOYMENT_DIR}/vr_headset_${DEPLOYMENT_ENV}_${DEPLOYMENT_TYPE}_${TIMESTAMP}.zip"
    cd "${package_dir}"
    zip -r "${package_file}" ./*
    
    log "Deployment package created: ${package_file}"
    echo "${package_file}"
}

# Create backup on target
create_remote_backup() {
    log "Creating backup on target device"
    
    # Create backup directory if it doesn't exist
    ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "mkdir -p ${TARGET_BACKUP_DIR}"
    
    # Create backup of current deployment
    local backup_name="backup_${TIMESTAMP}.tar.gz"
    ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "if [ -d ${TARGET_DIR}/current ]; then tar -czf ${TARGET_BACKUP_DIR}/${backup_name} -C ${TARGET_DIR} current; fi"
    
    log "Remote backup created: ${TARGET_BACKUP_DIR}/${backup_name}"
}

# Deploy package to target
deploy_package() {
    local package_file="$1"
    log "Deploying package to target: ${package_file}"
    
    # Create deployment directories on target
    ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "mkdir -p ${TARGET_DIR}/releases/${TIMESTAMP}"
    
    # Copy package to target
    scp -P "${TARGET_PORT}" "${package_file}" "${TARGET_USER}@${TARGET_HOST}:${TARGET_DIR}/releases/${TIMESTAMP}/package.zip"
    
    # Extract package on target
    ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "cd ${TARGET_DIR}/releases/${TIMESTAMP} && unzip package.zip && rm package.zip"
    
    # Update current symlink
    ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "ln -sfn ${TARGET_DIR}/releases/${TIMESTAMP} ${TARGET_DIR}/current"
    
    # Set permissions
    ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "chmod -R 755 ${TARGET_DIR}/releases/${TIMESTAMP}"
    
    # If CLI component was deployed, create symlinks to binaries
    if [ "${DEPLOYMENT_TYPE}" = "full" ] || [ "${DEPLOYMENT_TYPE}" = "cli" ]; then
        ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "if [ -d ${TARGET_DIR}/current/cli ]; then mkdir -p /usr/local/bin && ln -sf ${TARGET_DIR}/current/cli/vr_cli /usr/local/bin/vr_cli; fi"
    fi
    
    log "Deployment completed successfully"
}

# Verify deployment
verify_deployment() {
    log "Verifying deployment"
    
    # Check if current symlink exists
    if ! ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "[ -L ${TARGET_DIR}/current ]"; then
        log "Current symlink does not exist" "ERROR"
        return 1
    fi
    
    # Check if deployment metadata exists
    if ! ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "[ -f ${TARGET_DIR}/current/deployment.json ]"; then
        log "Deployment metadata does not exist" "ERROR"
        return 1
    fi
    
    # Verify components based on deployment type
    if [ "${DEPLOYMENT_TYPE}" = "full" ] || [ "${DEPLOYMENT_TYPE}" = "core_api" ]; then
        if ! ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "[ -d ${TARGET_DIR}/current/core_api ]"; then
            log "Core API component not found" "ERROR"
            return 1
        fi
    fi
    
    if [ "${DEPLOYMENT_TYPE}" = "full" ] || [ "${DEPLOYMENT_TYPE}" = "cli" ]; then
        if ! ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "[ -d ${TARGET_DIR}/current/cli ]"; then
            log "CLI Interface component not found" "ERROR"
            return 1
        fi
    fi
    
    if [ "${DEPLOYMENT_TYPE}" = "full" ] || [ "${DEPLOYMENT_TYPE}" = "web" ]; then
        if ! ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "[ -d ${TARGET_DIR}/current/web ]"; then
            log "Web Interface component not found" "ERROR"
            return 1
        fi
    fi
    
    log "Deployment verification successful"
    return 0
}

# Rollback deployment
rollback_deployment() {
    log "Rolling back deployment" "WARNING"
    
    # Find previous release
    local previous_release=$(ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "ls -1t ${TARGET_DIR}/releases | grep -v ${TIMESTAMP} | head -1")
    
    if [ -z "${previous_release}" ]; then
        log "No previous release found for rollback" "ERROR"
        return 1
    fi
    
    log "Rolling back to previous release: ${previous_release}"
    
    # Update current symlink to previous release
    ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "ln -sfn ${TARGET_DIR}/releases/${previous_release} ${TARGET_DIR}/current"
    
    # If CLI component was deployed, update symlinks to binaries
    if [ "${DEPLOYMENT_TYPE}" = "full" ] || [ "${DEPLOYMENT_TYPE}" = "cli" ]; then
        ssh -p "${TARGET_PORT}" "${TARGET_USER}@${TARGET_HOST}" "if [ -d ${TARGET_DIR}/current/cli ]; then mkdir -p /usr/local/bin && ln -sf ${TARGET_DIR}/current/cli/vr_cli /usr/local/bin/vr_cli; fi"
    fi
    
    log "Rollback completed"
    return 0
}

# Generate deployment report
generate_deployment_report() {
    local status="$1"
    log "Generating deployment report"
    
    # Create deployment report
    cat > "${DEPLOYMENT_REPORT}" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "environment": "${DEPLOYMENT_ENV}",
  "type": "${DEPLOYMENT_TYPE}",
  "version": "${TIMESTAMP}",
  "status": "${status}",
  "target_host": "${TARGET_HOST}",
  "target_dir": "${TARGET_DIR}",
  "components": [
EOF

    # Add component information
    if [ "${DEPLOYMENT_TYPE}" = "full" ] || [ "${DEPLOYMENT_TYPE}" = "core_api" ]; then
        echo '    {"name": "core_api", "version": "'${TIMESTAMP}'", "status": "'${status}'"},' >> "${DEPLOYMENT_REPORT}"
    fi
    
    if [ "${DEPLOYMENT_TYPE}" = "full" ] || [ "${DEPLOYMENT_TYPE}" = "cli" ]; then
        echo '    {"name": "cli", "version": "'${TIMESTAMP}'", "status": "'${status}'"},' >> "${DEPLOYMENT_REPORT}"
    fi
    
    if [ "${DEPLOYMENT_TYPE}" = "full" ] || [ "${DEPLOYMENT_TYPE}" = "web" ]; then
        echo '    {"name": "web", "version": "'${TIMESTAMP}'", "status": "'${status}'"}'  >> "${DEPLOYMENT_REPORT}"
    fi
    
    # Close the JSON
    echo '  ]' >> "${DEPLOYMENT_REPORT}"
    echo '}' >> "${DEPLOYMENT_REPORT}"
    
    log "Deployment report generated: ${DEPLOYMENT_REPORT}"
}

# Main deployment process
main() {
    log "Starting main deployment process"
    
    # Check SSH connection
    if ! check_ssh_connection; then
        log "Cannot proceed with deployment due to SSH connection failure" "ERROR"
        generate_deployment_report "failed"
        exit 1
    fi
    
    # Prepare deployment package
    local package_file=$(prepare_deployment_package)
    
    # Create backup on target
    create_remote_backup
    
    # Deploy package to target
    deploy_package "${package_file}"
    
    # Verify deployment
    if verify_deployment; then
        log "Deployment successful"
        generate_deployment_report "success"
    else
        log "Deployment verification failed, initiating rollback" "ERROR"
        rollback_deployment
        generate_deployment_report "rolled_back"
        exit 1
    fi
    
    log "Deployment automation completed successfully"
}

# Execute main function
main
