#!/bin/bash
# Orange Pi CM5 VR Headset Build System
# This script sets up the build environment for the Orange Pi CM5 VR headset project

set -e

# Configuration variables
ORANGEPI_VERSION="ubuntu-22.04"
KERNEL_VERSION="5.10.110"
PREEMPT_RT_PATCH_VERSION="5.10.110-rt63"
BUILD_DIR="$(pwd)/orangepi_build"
OUTPUT_DIR="$(pwd)/orangepi_output"
TOOLCHAIN_DIR="$(pwd)/toolchain"
THREADS=$(nproc)

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
echo -e "${GREEN}Build directory: ${BUILD_DIR}${NC}"
echo -e "${GREEN}Output directory: ${OUTPUT_DIR}${NC}"
echo -e "${GREEN}Using ${THREADS} threads for compilation${NC}"
echo -e "${BLUE}=======================================================${NC}"

# Create directories
mkdir -p "${BUILD_DIR}"
mkdir -p "${OUTPUT_DIR}"
mkdir -p "${TOOLCHAIN_DIR}"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install dependencies
install_dependencies() {
    echo -e "${YELLOW}Installing build dependencies...${NC}"
    
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
    
    echo -e "${GREEN}Dependencies installed successfully.${NC}"
}

# Function to download Orange Pi OS
download_orangepi_os() {
    echo -e "${YELLOW}Downloading Orange Pi OS...${NC}"
    
    cd "${BUILD_DIR}"
    
    if [ ! -f "orangepi_${ORANGEPI_VERSION}.img.xz" ]; then
        echo -e "${BLUE}Downloading Orange Pi OS image...${NC}"
        wget -c "http://www.orangepi.org/downloadresources/orangepicm5/orangepi_${ORANGEPI_VERSION}.img.xz"
    else
        echo -e "${GREEN}Orange Pi OS image already downloaded.${NC}"
    fi
    
    if [ ! -d "orangepi-build" ]; then
        echo -e "${BLUE}Cloning Orange Pi build system...${NC}"
        git clone https://github.com/orangepi-xunlong/orangepi-build.git
    else
        echo -e "${GREEN}Orange Pi build system already cloned.${NC}"
        cd orangepi-build
        git pull
        cd ..
    fi
    
    echo -e "${GREEN}Orange Pi OS downloaded successfully.${NC}"
}

# Function to download Linux kernel
download_kernel() {
    echo -e "${YELLOW}Downloading Linux kernel...${NC}"
    
    cd "${BUILD_DIR}"
    
    if [ ! -d "linux-${KERNEL_VERSION}" ]; then
        echo -e "${BLUE}Downloading kernel source...${NC}"
        wget -c "https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-${KERNEL_VERSION}.tar.xz"
        tar -xf "linux-${KERNEL_VERSION}.tar.xz"
    else
        echo -e "${GREEN}Kernel source already downloaded.${NC}"
    fi
    
    echo -e "${GREEN}Linux kernel downloaded successfully.${NC}"
}

# Function to download PREEMPT_RT patch
download_preempt_rt_patch() {
    echo -e "${YELLOW}Downloading PREEMPT_RT patch...${NC}"
    
    cd "${BUILD_DIR}"
    
    if [ ! -f "patch-${PREEMPT_RT_PATCH_VERSION}.patch.xz" ]; then
        echo -e "${BLUE}Downloading PREEMPT_RT patch...${NC}"
        wget -c "https://cdn.kernel.org/pub/linux/kernel/projects/rt/5.10/older/patch-${PREEMPT_RT_PATCH_VERSION}.patch.xz"
    else
        echo -e "${GREEN}PREEMPT_RT patch already downloaded.${NC}"
    fi
    
    echo -e "${GREEN}PREEMPT_RT patch downloaded successfully.${NC}"
}

# Function to apply PREEMPT_RT patch
apply_preempt_rt_patch() {
    echo -e "${YELLOW}Applying PREEMPT_RT patch...${NC}"
    
    cd "${BUILD_DIR}/linux-${KERNEL_VERSION}"
    
    # Check if patch is already applied
    if grep -q "PREEMPT_RT" Makefile; then
        echo -e "${GREEN}PREEMPT_RT patch already applied.${NC}"
        return
    fi
    
    # Apply the patch
    xzcat "../patch-${PREEMPT_RT_PATCH_VERSION}.patch.xz" | patch -p1
    
    echo -e "${GREEN}PREEMPT_RT patch applied successfully.${NC}"
}

# Function to configure kernel for VR
configure_kernel_for_vr() {
    echo -e "${YELLOW}Configuring kernel for VR...${NC}"
    
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
    
    # Configure CPU isolation
    scripts/config --enable CPU_ISOLATION
    
    # Configure memory management
    scripts/config --enable TRANSPARENT_HUGEPAGE
    scripts/config --enable CLEANCACHE
    scripts/config --enable FRONTSWAP
    
    # Configure DMA
    scripts/config --enable DMA_CMA
    scripts/config --set-val CMA_SIZE_MBYTES 256
    
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
    
    echo -e "${GREEN}Kernel configured for VR successfully.${NC}"
}

# Function to build kernel
build_kernel() {
    echo -e "${YELLOW}Building kernel...${NC}"
    
    cd "${BUILD_DIR}/linux-${KERNEL_VERSION}"
    
    # Build kernel
    make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- -j${THREADS}
    
    # Build modules
    make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- modules -j${THREADS}
    
    # Build device tree
    make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- dtbs -j${THREADS}
    
    echo -e "${GREEN}Kernel built successfully.${NC}"
}

# Function to build VR drivers
build_vr_drivers() {
    echo -e "${YELLOW}Building VR drivers...${NC}"
    
    # Create drivers directory
    mkdir -p "${BUILD_DIR}/vr_drivers"
    
    # Copy driver source files
    cp -r /home/ubuntu/orb_slam3_project/drivers/orange_pi/* "${BUILD_DIR}/vr_drivers/"
    
    # Build camera driver
    echo -e "${BLUE}Building camera driver...${NC}"
    cd "${BUILD_DIR}/vr_drivers/camera"
    make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- KERNEL_SRC="${BUILD_DIR}/linux-${KERNEL_VERSION}" -j${THREADS}
    
    # Build IMU driver
    echo -e "${BLUE}Building IMU driver...${NC}"
    cd "${BUILD_DIR}/vr_drivers/imu"
    make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- KERNEL_SRC="${BUILD_DIR}/linux-${KERNEL_VERSION}" -j${THREADS}
    
    # Build display driver
    echo -e "${BLUE}Building display driver...${NC}"
    cd "${BUILD_DIR}/vr_drivers/display"
    make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- KERNEL_SRC="${BUILD_DIR}/linux-${KERNEL_VERSION}" -j${THREADS}
    
    # Build WiFi driver
    echo -e "${BLUE}Building WiFi driver...${NC}"
    cd "${BUILD_DIR}/vr_drivers/wifi"
    make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- KERNEL_SRC="${BUILD_DIR}/linux-${KERNEL_VERSION}" -j${THREADS}
    
    # Build TPU driver
    echo -e "${BLUE}Building TPU driver...${NC}"
    cd "${BUILD_DIR}/vr_drivers/tpu"
    make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- KERNEL_SRC="${BUILD_DIR}/linux-${KERNEL_VERSION}" -j${THREADS}
    
    echo -e "${GREEN}VR drivers built successfully.${NC}"
}

# Function to create output image
create_output_image() {
    echo -e "${YELLOW}Creating output image...${NC}"
    
    # Extract base image
    cd "${BUILD_DIR}"
    xz -d -k "orangepi_${ORANGEPI_VERSION}.img.xz"
    
    # Copy image to output directory
    cp "orangepi_${ORANGEPI_VERSION}.img" "${OUTPUT_DIR}/orangepi_vr_headset.img"
    
    # Mount image
    LOOP_DEVICE=$(sudo losetup -f)
    sudo losetup -P ${LOOP_DEVICE} "${OUTPUT_DIR}/orangepi_vr_headset.img"
    
    # Create mount points
    mkdir -p "${BUILD_DIR}/mnt/boot"
    mkdir -p "${BUILD_DIR}/mnt/rootfs"
    
    # Mount partitions
    sudo mount ${LOOP_DEVICE}p1 "${BUILD_DIR}/mnt/boot"
    sudo mount ${LOOP_DEVICE}p2 "${BUILD_DIR}/mnt/rootfs"
    
    # Copy kernel
    sudo cp "${BUILD_DIR}/linux-${KERNEL_VERSION}/arch/arm64/boot/Image" "${BUILD_DIR}/mnt/boot/"
    
    # Copy device tree
    sudo cp "${BUILD_DIR}/linux-${KERNEL_VERSION}/arch/arm64/boot/dts/rockchip/rk3588s-orangepi-cm5.dtb" "${BUILD_DIR}/mnt/boot/"
    
    # Copy modules
    sudo make -C "${BUILD_DIR}/linux-${KERNEL_VERSION}" ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- INSTALL_MOD_PATH="${BUILD_DIR}/mnt/rootfs" modules_install
    
    # Copy VR drivers
    sudo mkdir -p "${BUILD_DIR}/mnt/rootfs/lib/modules/$(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)/extra"
    sudo cp "${BUILD_DIR}/vr_drivers/camera/*.ko" "${BUILD_DIR}/mnt/rootfs/lib/modules/$(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)/extra/"
    sudo cp "${BUILD_DIR}/vr_drivers/imu/*.ko" "${BUILD_DIR}/mnt/rootfs/lib/modules/$(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)/extra/"
    sudo cp "${BUILD_DIR}/vr_drivers/display/*.ko" "${BUILD_DIR}/mnt/rootfs/lib/modules/$(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)/extra/"
    sudo cp "${BUILD_DIR}/vr_drivers/wifi/*.ko" "${BUILD_DIR}/mnt/rootfs/lib/modules/$(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)/extra/"
    sudo cp "${BUILD_DIR}/vr_drivers/tpu/*.ko" "${BUILD_DIR}/mnt/rootfs/lib/modules/$(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)/extra/"
    
    # Update module dependencies
    sudo depmod -a -b "${BUILD_DIR}/mnt/rootfs" $(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)
    
    # Copy device tree overlay
    sudo mkdir -p "${BUILD_DIR}/mnt/boot/overlays"
    sudo cp "${BUILD_DIR}/linux-${KERNEL_VERSION}/arch/arm64/boot/dts/rockchip/overlay/rk3588s-orangepi-cm5-vr.dtbo" "${BUILD_DIR}/mnt/boot/overlays/"
    
    # Update boot configuration
    sudo sed -i 's/^kernel=.*/kernel=Image/' "${BUILD_DIR}/mnt/boot/config.txt"
    sudo sed -i 's/^dtoverlay=.*/dtoverlay=rk3588s-orangepi-cm5-vr/' "${BUILD_DIR}/mnt/boot/config.txt"
    
    # Unmount partitions
    sudo umount "${BUILD_DIR}/mnt/boot"
    sudo umount "${BUILD_DIR}/mnt/rootfs"
    
    # Detach loop device
    sudo losetup -d ${LOOP_DEVICE}
    
    # Compress output image
    cd "${OUTPUT_DIR}"
    xz -z -9 -k "orangepi_vr_headset.img"
    
    echo -e "${GREEN}Output image created successfully: ${OUTPUT_DIR}/orangepi_vr_headset.img.xz${NC}"
}

# Function to create documentation
create_documentation() {
    echo -e "${YELLOW}Creating documentation...${NC}"
    
    # Create documentation directory
    mkdir -p "${OUTPUT_DIR}/docs"
    
    # Create README
    cat > "${OUTPUT_DIR}/docs/README.md" << EOF
# Orange Pi CM5 VR Headset Build System

This build system creates a custom Orange Pi CM5 image with PREEMPT_RT patches and VR-specific drivers for the VR headset project.

## Image Details

- Base OS: Orange Pi OS (${ORANGEPI_VERSION})
- Kernel Version: ${KERNEL_VERSION}
- PREEMPT_RT Patch: ${PREEMPT_RT_PATCH_VERSION}

## VR-Specific Features

- PREEMPT_RT real-time kernel for low-latency operation
- CPU isolation for critical VR threads
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
   xzcat orangepi_vr_headset.img.xz | sudo dd of=/dev/sdX bs=4M status=progress
   \`\`\`

2. Insert the SD card into the Orange Pi CM5 and boot.

## Configuration

The system is pre-configured for VR operation, but you can adjust settings in:

- \`/boot/config.txt\` - Boot configuration
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
    cat > "${OUTPUT_DIR}/docs/build_instructions.md" << EOF
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
   output/orangepi_vr_headset.img.xz
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
    cat > "${OUTPUT_DIR}/docs/troubleshooting.md" << EOF
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
- Check I2C bus:
  \`\`\`
  i2cdetect -y 4
  \`\`\`

### Display not working

- Check that the display is properly connected
- Verify that the display driver is loaded:
  \`\`\`
  lsmod | grep drm_rockchip
  \`\`\`
- Check display configuration:
  \`\`\`
  cat /boot/config.txt
  \`\`\`

### WiFi not working

- Check that the WiFi card is properly inserted
- Verify that the WiFi driver is loaded:
  \`\`\`
  lsmod | grep iwlwifi
  \`\`\`
- Check WiFi firmware:
  \`\`\`
  ls -l /lib/firmware/iwlwifi-*
  \`\`\`

### TPU not working

- Check that the TPU is properly connected
- Verify that the TPU driver is loaded:
  \`\`\`
  lsmod | grep apex
  \`\`\`
- Check PCIe bus:
  \`\`\`
  lspci
  \`\`\`

## Performance Issues

### High latency

- Check if CPU isolation is working:
  \`\`\`
  cat /proc/cmdline | grep isolcpus
  \`\`\`
- Verify that the PREEMPT_RT patch is applied:
  \`\`\`
  uname -a | grep rt
  \`\`\`
- Check for processes using high CPU:
  \`\`\`
  top
  \`\`\`

### System crashes

- Check system logs:
  \`\`\`
  dmesg
  journalctl -b
  \`\`\`
- Check temperature:
  \`\`\`
  cat /sys/class/thermal/thermal_zone*/temp
  \`\`\`
- Check memory usage:
  \`\`\`
  free -h
  \`\`\`

## Getting Help

If you can't resolve the issue, please file a bug report with:

1. Detailed description of the issue
2. Steps to reproduce
3. System logs (dmesg, journalctl)
4. Hardware configuration
EOF
    
    echo -e "${GREEN}Documentation created successfully.${NC}"
}

# Main function
main() {
    echo -e "${YELLOW}Starting build process...${NC}"
    
    # Check if running as root
    if [ "$(id -u)" = "0" ]; then
        echo -e "${RED}Error: This script should not be run as root${NC}"
        exit 1
    fi
    
    # Install dependencies
    install_dependencies
    
    # Download Orange Pi OS
    download_orangepi_os
    
    # Download Linux kernel
    download_kernel
    
    # Download PREEMPT_RT patch
    download_preempt_rt_patch
    
    # Apply PREEMPT_RT patch
    apply_preempt_rt_patch
    
    # Configure kernel for VR
    configure_kernel_for_vr
    
    # Build kernel
    build_kernel
    
    # Build VR drivers
    build_vr_drivers
    
    # Create output image
    create_output_image
    
    # Create documentation
    create_documentation
    
    echo -e "${GREEN}Build process completed successfully!${NC}"
    echo -e "${GREEN}Output image: ${OUTPUT_DIR}/orangepi_vr_headset.img.xz${NC}"
    echo -e "${GREEN}Documentation: ${OUTPUT_DIR}/docs/${NC}"
}

# Run main function
main "$@"
