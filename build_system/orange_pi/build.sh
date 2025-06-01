#!/bin/bash
# Orange Pi CM5 VR Headset Build System
# This script builds the complete Orange Pi CM5 VR headset system

set -e

# Load configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/config.sh"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print banner
echo -e "${BLUE}=======================================================${NC}"
echo -e "${BLUE}      Orange Pi CM5 VR Headset Build System           ${NC}"
echo -e "${BLUE}=======================================================${NC}"
echo -e "${GREEN}Build system version: ${BUILD_SYSTEM_VERSION}${NC}"
echo -e "${GREEN}Build date: ${BUILD_DATE}${NC}"
echo -e "${GREEN}Build directory: ${BUILD_DIR}${NC}"
echo -e "${GREEN}Output directory: ${OUTPUT_DIR}${NC}"
echo -e "${GREEN}Using ${THREADS} threads for compilation${NC}"
echo -e "${BLUE}=======================================================${NC}"

# Create directories
mkdir -p "${BUILD_DIR}"
mkdir -p "${OUTPUT_DIR}"
mkdir -p "${TOOLCHAIN_DIR}"
mkdir -p "${LOG_DIR}"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to log messages
log() {
    local level="$1"
    local message="$2"
    local color="${NC}"
    
    case "$level" in
        "INFO") color="${GREEN}" ;;
        "WARNING") color="${YELLOW}" ;;
        "ERROR") color="${RED}" ;;
        *) color="${BLUE}" ;;
    esac
    
    echo -e "${color}[$(date '+%Y-%m-%d %H:%M:%S')] [${level}] ${message}${NC}"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [${level}] ${message}" >> "${LOG_DIR}/build.log"
}

# Function to install dependencies
install_dependencies() {
    log "INFO" "Installing build dependencies..."
    
    sudo apt-get update
    sudo apt-get install -y \
        build-essential \
        gcc-aarch64-linux-gnu \
        g++-aarch64-linux-gnu \
        bison \
        flex \
        libssl-dev \
        libncurses-dev \
        libelf-dev \
        bc \
        lzop \
        device-tree-compiler \
        u-boot-tools \
        python3 \
        python3-pip \
        python3-dev \
        python3-setuptools \
        python3-wheel \
        git \
        wget \
        curl \
        rsync \
        cpio \
        file \
        kmod \
        unzip \
        libusb-1.0-0-dev \
        libfdt-dev \
        libpixman-1-dev \
        zlib1g-dev \
        libnuma-dev
    
    # Install additional Python packages
    pip3 install --user pycrypto pyelftools
    
    log "INFO" "Dependencies installed successfully."
}

# Function to download Orange Pi OS
download_orangepi_os() {
    log "INFO" "Downloading Orange Pi OS..."
    
    cd "${BUILD_DIR}"
    
    if [ ! -f "${ORANGEPI_IMAGE}" ]; then
        log "INFO" "Downloading Orange Pi OS image..."
        wget -c "${ORANGEPI_MIRROR}/${ORANGEPI_IMAGE}"
    else
        log "INFO" "Orange Pi OS image already downloaded."
    fi
    
    if [ ! -d "orangepi-build" ]; then
        log "INFO" "Cloning Orange Pi build system..."
        git clone "${ORANGEPI_BUILD_REPO}" --branch "${ORANGEPI_BUILD_BRANCH}" --depth 1
    else
        log "INFO" "Orange Pi build system already cloned."
        cd orangepi-build
        git pull
        cd ..
    fi
    
    log "INFO" "Orange Pi OS downloaded successfully."
}

# Function to download Linux kernel
download_kernel() {
    log "INFO" "Downloading Linux kernel..."
    
    cd "${BUILD_DIR}"
    
    if [ ! -d "linux-${KERNEL_VERSION}" ]; then
        log "INFO" "Downloading kernel source..."
        wget -c "${KERNEL_MIRROR}/${KERNEL_SOURCE}"
        tar -xf "${KERNEL_SOURCE}"
    else
        log "INFO" "Kernel source already downloaded."
    fi
    
    log "INFO" "Linux kernel downloaded successfully."
}

# Function to download PREEMPT_RT patch
download_preempt_rt_patch() {
    log "INFO" "Downloading PREEMPT_RT patch..."
    
    cd "${BUILD_DIR}"
    
    if [ ! -f "${PREEMPT_RT_PATCH}" ]; then
        log "INFO" "Downloading PREEMPT_RT patch..."
        wget -c "${PREEMPT_RT_MIRROR}/${PREEMPT_RT_PATCH}"
    else
        log "INFO" "PREEMPT_RT patch already downloaded."
    fi
    
    log "INFO" "PREEMPT_RT patch downloaded successfully."
}

# Function to apply PREEMPT_RT patch
apply_preempt_rt_patch() {
    log "INFO" "Applying PREEMPT_RT patch..."
    
    cd "${BUILD_DIR}/linux-${KERNEL_VERSION}"
    
    # Check if patch is already applied
    if grep -q "PREEMPT_RT" Makefile; then
        log "INFO" "PREEMPT_RT patch already applied."
        return
    fi
    
    # Apply the patch
    xzcat "../${PREEMPT_RT_PATCH}" | patch -p1
    
    log "INFO" "PREEMPT_RT patch applied successfully."
}

# Function to copy device tree files
copy_device_tree_files() {
    log "INFO" "Copying device tree files..."
    
    cd "${BUILD_DIR}/linux-${KERNEL_VERSION}"
    
    # Create directory if it doesn't exist
    mkdir -p "arch/arm64/boot/dts/rockchip"
    
    # Copy device tree file
    cp "${DEVICE_TREE_DIR}/${DEVICE_TREE_FILE}" "arch/arm64/boot/dts/rockchip/"
    
    # Create overlay directory if it doesn't exist
    mkdir -p "arch/arm64/boot/dts/rockchip/overlay"
    
    # Copy overlay files if they exist
    if [ -d "${DEVICE_TREE_DIR}/overlay" ]; then
        cp "${DEVICE_TREE_DIR}/overlay/"*.dts "arch/arm64/boot/dts/rockchip/overlay/"
    fi
    
    log "INFO" "Device tree files copied successfully."
}

# Function to configure kernel for VR
configure_kernel_for_vr() {
    log "INFO" "Configuring kernel for VR..."
    
    cd "${BUILD_DIR}/linux-${KERNEL_VERSION}"
    
    # Start with Orange Pi default config
    cp -f "${BUILD_DIR}/orangepi-build/kernel/arch/arm64/configs/orangepi_defconfig" .config
    
    # Enable PREEMPT_RT
    scripts/config --enable PREEMPT
    scripts/config --set-val PREEMPT_RT y
    scripts/config --disable PREEMPT_VOLUNTARY
    scripts/config --disable PREEMPT_NONE
    
    # Enable high-resolution timers
    scripts/config --enable HIGH_RES_TIMERS
    scripts/config --enable NO_HZ_FULL
    scripts/config --enable RCU_NOCB_CPU
    
    # Configure CPU isolation
    scripts/config --enable CPU_ISOLATION
    
    # Configure memory management
    scripts/config --enable TRANSPARENT_HUGEPAGE
    scripts/config --enable CLEANCACHE
    scripts/config --enable FRONTSWAP
    
    # Configure DMA
    scripts/config --enable DMA_CMA
    scripts/config --set-val CMA_SIZE_MBYTES 1024
    
    # Configure PCIe for TPU and WiFi
    scripts/config --enable PCI
    scripts/config --enable PCIEPORTBUS
    scripts/config --enable PCIE_ROCKCHIP_HOST
    
    # Configure V4L2 for cameras
    scripts/config --enable V4L_PLATFORM_DRIVERS
    scripts/config --enable VIDEO_V4L2
    scripts/config --enable VIDEO_OV9281
    
    # Configure DRM for displays
    scripts/config --enable DRM
    scripts/config --enable DRM_ROCKCHIP
    
    # Configure IIO for IMU
    scripts/config --enable IIO
    scripts/config --enable IIO_BUFFER
    scripts/config --enable IIO_TRIGGERED_BUFFER
    
    # Configure WiFi
    scripts/config --enable IWLWIFI
    scripts/config --enable IWLMVM
    
    # Configure USB for development
    scripts/config --enable USB_SUPPORT
    scripts/config --enable USB_XHCI_HCD
    scripts/config --enable USB_EHCI_HCD
    scripts/config --enable USB_OHCI_HCD
    
    # Save the configuration
    yes "" | make ARCH=arm64 oldconfig
    
    log "INFO" "Kernel configured for VR successfully."
}

# Function to build kernel
build_kernel() {
    log "INFO" "Building kernel..."
    
    cd "${BUILD_DIR}/linux-${KERNEL_VERSION}"
    
    # Build kernel
    make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- -j${THREADS}
    
    # Build modules
    make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- modules -j${THREADS}
    
    # Build device tree
    make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- dtbs -j${THREADS}
    
    log "INFO" "Kernel built successfully."
}

# Function to build VR drivers
build_vr_drivers() {
    log "INFO" "Building VR drivers..."
    
    # Create drivers directory
    mkdir -p "${BUILD_DIR}/vr_drivers"
    
    # Build camera driver if enabled
    if [ "${CAMERA_DRIVER_ENABLED}" = true ]; then
        log "INFO" "Building camera driver..."
        cd "${CAMERA_DRIVER_DIR}"
        make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- KERNEL_SRC="${BUILD_DIR}/linux-${KERNEL_VERSION}" -j${THREADS}
    fi
    
    # Build IMU driver if enabled
    if [ "${IMU_DRIVER_ENABLED}" = true ]; then
        log "INFO" "Building IMU driver..."
        cd "${IMU_DRIVER_DIR}"
        make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- KERNEL_SRC="${BUILD_DIR}/linux-${KERNEL_VERSION}" -j${THREADS}
    fi
    
    # Build display driver if enabled
    if [ "${DISPLAY_DRIVER_ENABLED}" = true ]; then
        log "INFO" "Building display driver..."
        cd "${DISPLAY_DRIVER_DIR}"
        make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- KERNEL_SRC="${BUILD_DIR}/linux-${KERNEL_VERSION}" -j${THREADS}
    fi
    
    # Build WiFi driver if enabled
    if [ "${WIFI_DRIVER_ENABLED}" = true ]; then
        log "INFO" "Building WiFi driver..."
        cd "${WIFI_DRIVER_DIR}"
        make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- KERNEL_SRC="${BUILD_DIR}/linux-${KERNEL_VERSION}" -j${THREADS}
    fi
    
    # Build TPU driver if enabled
    if [ "${TPU_DRIVER_ENABLED}" = true ]; then
        log "INFO" "Building TPU driver..."
        cd "${TPU_DRIVER_DIR}"
        make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- KERNEL_SRC="${BUILD_DIR}/linux-${KERNEL_VERSION}" -j${THREADS}
    fi
    
    log "INFO" "VR drivers built successfully."
}

# Function to create output image
create_output_image() {
    log "INFO" "Creating output image..."
    
    # Extract base image
    cd "${BUILD_DIR}"
    xz -d -k "${ORANGEPI_IMAGE}"
    
    # Copy image to output directory
    cp "${ORANGEPI_IMAGE%.xz}" "${OUTPUT_DIR}/${OUTPUT_IMAGE_NAME}"
    
    # Mount image
    LOOP_DEVICE=$(sudo losetup -f)
    sudo losetup -P ${LOOP_DEVICE} "${OUTPUT_DIR}/${OUTPUT_IMAGE_NAME}"
    
    # Create mount points
    mkdir -p "${BUILD_DIR}/mnt/boot"
    mkdir -p "${BUILD_DIR}/mnt/rootfs"
    
    # Mount partitions
    sudo mount ${LOOP_DEVICE}p1 "${BUILD_DIR}/mnt/boot"
    sudo mount ${LOOP_DEVICE}p2 "${BUILD_DIR}/mnt/rootfs"
    
    # Copy kernel
    sudo cp "${BUILD_DIR}/linux-${KERNEL_VERSION}/arch/arm64/boot/Image" "${BUILD_DIR}/mnt/boot/"
    
    # Copy device tree
    sudo cp "${BUILD_DIR}/linux-${KERNEL_VERSION}/arch/arm64/boot/dts/rockchip/${DEVICE_TREE_FILE%.dts}.dtb" "${BUILD_DIR}/mnt/boot/"
    
    # Copy modules
    sudo make -C "${BUILD_DIR}/linux-${KERNEL_VERSION}" ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- INSTALL_MOD_PATH="${BUILD_DIR}/mnt/rootfs" modules_install
    
    # Copy VR drivers
    sudo mkdir -p "${BUILD_DIR}/mnt/rootfs/lib/modules/$(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)/extra"
    
    # Copy camera driver if enabled
    if [ "${CAMERA_DRIVER_ENABLED}" = true ]; then
        sudo cp "${CAMERA_DRIVER_DIR}"/*.ko "${BUILD_DIR}/mnt/rootfs/lib/modules/$(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)/extra/"
    fi
    
    # Copy IMU driver if enabled
    if [ "${IMU_DRIVER_ENABLED}" = true ]; then
        sudo cp "${IMU_DRIVER_DIR}"/*.ko "${BUILD_DIR}/mnt/rootfs/lib/modules/$(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)/extra/"
    fi
    
    # Copy display driver if enabled
    if [ "${DISPLAY_DRIVER_ENABLED}" = true ]; then
        sudo cp "${DISPLAY_DRIVER_DIR}"/*.ko "${BUILD_DIR}/mnt/rootfs/lib/modules/$(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)/extra/"
    fi
    
    # Copy WiFi driver if enabled
    if [ "${WIFI_DRIVER_ENABLED}" = true ]; then
        sudo cp "${WIFI_DRIVER_DIR}"/*.ko "${BUILD_DIR}/mnt/rootfs/lib/modules/$(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)/extra/"
    fi
    
    # Copy TPU driver if enabled
    if [ "${TPU_DRIVER_ENABLED}" = true ]; then
        sudo cp "${TPU_DRIVER_DIR}"/*.ko "${BUILD_DIR}/mnt/rootfs/lib/modules/$(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)/extra/"
    fi
    
    # Update module dependencies
    sudo depmod -a -b "${BUILD_DIR}/mnt/rootfs" $(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)
    
    # Copy device tree overlay
    sudo mkdir -p "${BUILD_DIR}/mnt/boot/overlays"
    if [ -d "${BUILD_DIR}/linux-${KERNEL_VERSION}/arch/arm64/boot/dts/rockchip/overlay" ]; then
        sudo cp "${BUILD_DIR}/linux-${KERNEL_VERSION}/arch/arm64/boot/dts/rockchip/overlay/"*.dtbo "${BUILD_DIR}/mnt/boot/overlays/"
    fi
    
    # Update boot configuration
    sudo bash -c "cat > ${BUILD_DIR}/mnt/boot/orangepiEnv.txt << EOF
verbosity=7
bootlogo=false
overlay_prefix=rockchip
rootdev=UUID=\$(blkid -s UUID -o value \${LOOP_DEVICE}p2)
rootfstype=ext4
overlays=rk3588s-orangepi-cm5-vr
extraargs=cma=${MEMORY_CMA_SIZE} hugepagesz=${MEMORY_HUGEPAGES_SIZE} hugepages=${MEMORY_HUGEPAGES_COUNT} isolcpus=${VR_CPU_ISOLATION} nohz_full=${VR_CPU_ISOLATION} rcu_nocbs=${VR_CPU_ISOLATION}
EOF"
    
    # Unmount partitions
    sudo umount "${BUILD_DIR}/mnt/boot"
    sudo umount "${BUILD_DIR}/mnt/rootfs"
    
    # Detach loop device
    sudo losetup -d ${LOOP_DEVICE}
    
    # Compress output image if enabled
    if [ "${OUTPUT_IMAGE_COMPRESS}" = true ]; then
        cd "${OUTPUT_DIR}"
        xz -z -${OUTPUT_IMAGE_COMPRESSION_LEVEL} -k "${OUTPUT_IMAGE_NAME}"
        log "INFO" "Output image compressed successfully: ${OUTPUT_DIR}/${OUTPUT_IMAGE_NAME}.xz"
    fi
    
    log "INFO" "Output image created successfully: ${OUTPUT_DIR}/${OUTPUT_IMAGE_NAME}"
}

# Function to create documentation
create_documentation() {
    if [ "${DOCS_ENABLED}" = true ]; then
        log "INFO" "Creating documentation..."
        
        # Create documentation directory
        mkdir -p "${DOCS_OUTPUT_DIR}"
        
        # Create README
        cat > "${DOCS_OUTPUT_DIR}/README.md" << EOF
# Orange Pi CM5 VR Headset Build System

This build system creates a custom Orange Pi CM5 image with PREEMPT_RT patches and VR-specific drivers for the VR headset project.

## Image Details

- Base OS: Orange Pi OS (${ORANGEPI_VERSION})
- Kernel Version: ${KERNEL_VERSION}
- PREEMPT_RT Patch: ${PREEMPT_RT_PATCH_VERSION}
- Memory: ${MEMORY_TOTAL} RAM with ${MEMORY_CMA_SIZE} CMA

## VR-Specific Features

- PREEMPT_RT real-time kernel for low-latency operation
- CPU isolation for critical VR threads (cores ${VR_CPU_ISOLATION})
- Optimized memory management for 16GB RAM
- Custom drivers for:
  - OV9281 cameras with zero-copy buffer sharing
  - BNO085 IMU with 1000Hz sampling rate
  - Dual MIPI DSI displays with low persistence mode
  - Intel AX210 WiFi with latency optimization
  - Coral TPU with zero-copy buffer management

## Installation

1. Flash the image to an SD card:
   \`\`\`
   xzcat ${OUTPUT_IMAGE_NAME}.xz | sudo dd of=/dev/sdX bs=4M status=progress
   \`\`\`

2. Insert the SD card into the Orange Pi CM5 and boot.

## Configuration

The system is pre-configured for VR operation, but you can adjust settings in:

- \`/boot/orangepiEnv.txt\` - Boot configuration
- \`/etc/vr/config.json\` - VR-specific settings

## Development

To build custom drivers or applications:

1. Install development tools:
   \`\`\`
   sudo apt-get install build-essential git cmake
   \`\`\`

2. Clone the VR headset project repository:
   \`\`\`
   git clone https://github.com/vr-headset/orangepi-cm5-vr.git
   \`\`\`

3. Follow the build instructions in the repository README.

## Troubleshooting

See the troubleshooting guide in \`/docs/troubleshooting.md\`.
EOF
        
        # Create build instructions
        cat > "${DOCS_OUTPUT_DIR}/build_instructions.md" << EOF
# Build Instructions

This document describes how to build the Orange Pi CM5 VR headset image from source.

## Prerequisites

- Ubuntu 20.04 or newer
- At least 50GB of free disk space
- At least 8GB of RAM
- Internet connection

## Build Steps

1. Install dependencies:
   \`\`\`
   sudo apt-get update
   sudo apt-get install build-essential gcc-aarch64-linux-gnu g++-aarch64-linux-gnu bison flex libssl-dev libncurses-dev libelf-dev bc lzop device-tree-compiler u-boot-tools python3 python3-pip git wget curl rsync cpio file kmod unzip
   \`\`\`

2. Clone the build repository:
   \`\`\`
   git clone https://github.com/vr-headset/orangepi-cm5-vr-build.git
   cd orangepi-cm5-vr-build
   \`\`\`

3. Run the build script:
   \`\`\`
   ./build.sh
   \`\`\`

4. The output image will be in the \`output\` directory:
   \`\`\`
   output/${OUTPUT_IMAGE_NAME}.xz
   \`\`\`

## Customization

To customize the build:

1. Edit \`config.sh\` to change build parameters.
2. Edit \`kernel_config.sh\` to change kernel configuration.
3. Edit \`drivers/\` to modify driver source code.

## Troubleshooting

- If the build fails, check the log file in \`logs/build.log\`.
- Make sure you have enough disk space and RAM.
- If you encounter network issues, try using a different mirror in \`config.sh\`.
EOF
        
        # Create troubleshooting guide
        cat > "${DOCS_OUTPUT_DIR}/troubleshooting.md" << EOF
# Troubleshooting Guide

This guide helps diagnose and fix common issues with the Orange Pi CM5 VR headset.

## Boot Issues

### System doesn't boot

- Check that the SD card is properly inserted
- Verify the image was correctly written to the SD card
- Try a different SD card
- Check power supply (5V/3A recommended)

### Kernel panic during boot

- Boot with serial console to see error messages
- Check if the PREEMPT_RT patch is causing issues
- Try booting with an older kernel version

## Driver Issues

### Camera not working

- Check that the camera is properly connected
- Verify that the camera driver is loaded:
  \`\`\`
  lsmod | grep ov9281
  \`\`\`
- Check camera permissions:
  \`\`\`
  ls -l /dev/video*
  \`\`\`

### IMU not working

- Check that the IMU is properly connected
- Verify that the IMU driver is loaded:
  \`\`\`
  lsmod | grep bno085
  \`\`\`
- Check IMU permissions:
  \`\`\`
  ls -l /dev/iio*
  \`\`\`

### Display not working

- Check that the display is properly connected
- Verify that the display driver is loaded:
  \`\`\`
  lsmod | grep drm_kms_helper
  \`\`\`
- Check display permissions:
  \`\`\`
  ls -l /dev/dri/*
  \`\`\`

### WiFi not working

- Check that the WiFi card is properly connected
- Verify that the WiFi driver is loaded:
  \`\`\`
  lsmod | grep iwlwifi
  \`\`\`
- Check WiFi interface:
  \`\`\`
  ip link show
  \`\`\`

### TPU not working

- Check that the TPU is properly connected
- Verify that the TPU driver is loaded:
  \`\`\`
  lsmod | grep edgetpu
  \`\`\`
- Check TPU device:
  \`\`\`
  ls -l /dev/apex_*
  \`\`\`

## Performance Issues

### High latency

- Check CPU isolation:
  \`\`\`
  cat /proc/cmdline | grep isolcpus
  \`\`\`
- Check CPU frequency:
  \`\`\`
  cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
  \`\`\`
- Check thermal throttling:
  \`\`\`
  cat /sys/devices/virtual/thermal/thermal_zone*/temp
  \`\`\`

### Memory issues

- Check memory usage:
  \`\`\`
  free -h
  \`\`\`
- Check CMA allocation:
  \`\`\`
  cat /proc/meminfo | grep CMA
  \`\`\`
- Check huge pages:
  \`\`\`
  cat /proc/meminfo | grep Huge
  \`\`\`

## System Issues

### System crashes

- Check kernel logs:
  \`\`\`
  dmesg
  \`\`\`
- Check system logs:
  \`\`\`
  journalctl -b
  \`\`\`
- Check memory errors:
  \`\`\`
  cat /var/log/kern.log | grep -i error
  \`\`\`

### System hangs

- Try connecting via SSH
- Check if system is responsive to SysRq keys:
  \`\`\`
  echo b > /proc/sysrq-trigger
  \`\`\`
- Check if system is overheating:
  \`\`\`
  cat /sys/devices/virtual/thermal/thermal_zone*/temp
  \`\`\`
EOF
        
        log "INFO" "Documentation created successfully."
    else
        log "INFO" "Documentation generation disabled."
    fi
}

# Function to run a build step with error handling
run_step() {
    local step_name="$1"
    local step_function="$2"
    
    log "INFO" "Starting step: ${step_name}"
    
    if ${step_function}; then
        log "INFO" "Step completed successfully: ${step_name}"
        return 0
    else
        log "ERROR" "Step failed: ${step_name}"
        return 1
    fi
}

# Main build process
main() {
    log "INFO" "Starting Orange Pi CM5 VR headset build process..."
    
    # Create log directory
    mkdir -p "${LOG_DIR}"
    
    # Run build steps
    run_step "Install Dependencies" install_dependencies
    run_step "Download Orange Pi OS" download_orangepi_os
    run_step "Download Linux Kernel" download_kernel
    run_step "Download PREEMPT_RT Patch" download_preempt_rt_patch
    run_step "Apply PREEMPT_RT Patch" apply_preempt_rt_patch
    run_step "Copy Device Tree Files" copy_device_tree_files
    run_step "Configure Kernel for VR" configure_kernel_for_vr
    run_step "Build Kernel" build_kernel
    run_step "Build VR Drivers" build_vr_drivers
    run_step "Create Output Image" create_output_image
    run_step "Create Documentation" create_documentation
    
    log "INFO" "Orange Pi CM5 VR headset build process completed successfully."
    log "INFO" "Output image: ${OUTPUT_DIR}/${OUTPUT_IMAGE_NAME}"
    if [ "${OUTPUT_IMAGE_COMPRESS}" = true ]; then
        log "INFO" "Compressed image: ${OUTPUT_DIR}/${OUTPUT_IMAGE_NAME}.xz"
    fi
    log "INFO" "Documentation: ${DOCS_OUTPUT_DIR}"
}

# Run main function
main "$@"
