#!/bin/bash
# Orange Pi CM5 VR Headset Build System Configuration
# This file contains all configurable parameters for the Orange Pi CM5 build system

# Build system version
BUILD_SYSTEM_VERSION="1.0.0"
BUILD_DATE="$(date +%Y-%m-%d)"

# Orange Pi OS configuration
ORANGEPI_VERSION="ubuntu-22.04"
ORANGEPI_MIRROR="http://www.orangepi.org/downloadresources/orangepicm5"
ORANGEPI_IMAGE="orangepi_${ORANGEPI_VERSION}.img.xz"
ORANGEPI_BUILD_REPO="https://github.com/orangepi-xunlong/orangepi-build.git"
ORANGEPI_BUILD_BRANCH="main"

# Kernel configuration
KERNEL_VERSION="5.10.110"
KERNEL_MIRROR="https://cdn.kernel.org/pub/linux/kernel/v5.x"
KERNEL_SOURCE="linux-${KERNEL_VERSION}.tar.xz"
PREEMPT_RT_PATCH_VERSION="5.10.110-rt63"
PREEMPT_RT_MIRROR="https://cdn.kernel.org/pub/linux/kernel/projects/rt/5.10/older"
PREEMPT_RT_PATCH="patch-${PREEMPT_RT_PATCH_VERSION}.patch.xz"

# Build directories
BUILD_DIR="${HOME}/orangepi_build"
OUTPUT_DIR="${HOME}/orangepi_output"
TOOLCHAIN_DIR="${HOME}/toolchain"
LOG_DIR="${HOME}/orangepi_logs"
DRIVERS_DIR="${HOME}/orb_slam3_project/drivers/orange_pi"

# Compilation settings
THREADS=$(nproc)
CROSS_COMPILE="aarch64-linux-gnu-"
ARCH="arm64"

# VR-specific settings
VR_MODE_ENABLED=true
VR_CPU_ISOLATION="0,1"
VR_MEMORY_RESERVATION="512M"
VR_DMA_BUFFER_SIZE="256M"
VR_LATENCY_TARGET_US=5000
VR_POWER_MODE="performance"

# Driver settings
CAMERA_DRIVER_ENABLED=true
CAMERA_DRIVER_VERSION="1.0.0"
CAMERA_DRIVER_DIR="${DRIVERS_DIR}/camera"

IMU_DRIVER_ENABLED=true
IMU_DRIVER_VERSION="1.0.0"
IMU_DRIVER_DIR="${DRIVERS_DIR}/imu"

DISPLAY_DRIVER_ENABLED=true
DISPLAY_DRIVER_VERSION="1.0.0"
DISPLAY_DRIVER_DIR="${DRIVERS_DIR}/display"

WIFI_DRIVER_ENABLED=true
WIFI_DRIVER_VERSION="1.0.0"
WIFI_DRIVER_DIR="${DRIVERS_DIR}/wifi"

TPU_DRIVER_ENABLED=true
TPU_DRIVER_VERSION="1.0.0"
TPU_DRIVER_DIR="${DRIVERS_DIR}/tpu"

# Output image settings
OUTPUT_IMAGE_NAME="orangepi_vr_headset.img"
OUTPUT_IMAGE_COMPRESS=true
OUTPUT_IMAGE_COMPRESSION_LEVEL=9

# Documentation settings
DOCS_ENABLED=true
DOCS_OUTPUT_DIR="${OUTPUT_DIR}/docs"

# Device tree settings
DEVICE_TREE_FILE="rk3588s-orangepi-cm5-vr.dts"
DEVICE_TREE_DIR="${DRIVERS_DIR}/device_tree"

# Memory configuration for 16GB RAM
MEMORY_TOTAL="16G"
MEMORY_CMA_SIZE="1G"
MEMORY_HUGEPAGES_SIZE="2M"
MEMORY_HUGEPAGES_COUNT="512"

# Debug settings
DEBUG_ENABLED=false
DEBUG_VERBOSE=false
