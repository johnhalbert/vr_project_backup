#!/bin/bash
# Orange Pi CM5 OS Setup Script
# This script sets up the Orange Pi OS image with PREEMPT_RT patches for VR applications

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration variables
KERNEL_VERSION="5.10.110"
PREEMPT_RT_PATCH_VERSION="5.10.110-rt63"
BUILD_DIR="${HOME}/orangepi_os_build"
OUTPUT_DIR="${HOME}/orangepi_os_output"
LOG_DIR="${HOME}/orangepi_os_logs"
ORANGEPI_VERSION="ubuntu-22.04"
ORANGEPI_MIRROR="http://www.orangepi.org/downloadresources/orangepicm5"
ORANGEPI_IMAGE="orangepi_${ORANGEPI_VERSION}.img.xz"
THREADS=$(nproc)

# Print banner
echo -e "${BLUE}=======================================================${NC}"
echo -e "${BLUE}      Orange Pi CM5 OS Setup with PREEMPT_RT           ${NC}"
echo -e "${BLUE}=======================================================${NC}"
echo -e "${GREEN}Build directory: ${BUILD_DIR}${NC}"
echo -e "${GREEN}Output directory: ${OUTPUT_DIR}${NC}"
echo -e "${GREEN}Using ${THREADS} threads for compilation${NC}"
echo -e "${BLUE}=======================================================${NC}"

# Create directories
mkdir -p "${BUILD_DIR}"
mkdir -p "${OUTPUT_DIR}"
mkdir -p "${LOG_DIR}"

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
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [${level}] ${message}" >> "${LOG_DIR}/os_setup.log"
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
        libnuma-dev \
        qemu-user-static \
        debootstrap \
        binfmt-support
    
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
        git clone https://github.com/orangepi-xunlong/orangepi-build.git --depth 1
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
        wget -c "https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-${KERNEL_VERSION}.tar.xz"
        tar -xf "linux-${KERNEL_VERSION}.tar.xz"
    else
        log "INFO" "Kernel source already downloaded."
    fi
    
    log "INFO" "Linux kernel downloaded successfully."
}

# Function to download PREEMPT_RT patch
download_preempt_rt_patch() {
    log "INFO" "Downloading PREEMPT_RT patch..."
    
    cd "${BUILD_DIR}"
    
    if [ ! -f "patch-${PREEMPT_RT_PATCH_VERSION}.patch.xz" ]; then
        log "INFO" "Downloading PREEMPT_RT patch..."
        wget -c "https://cdn.kernel.org/pub/linux/kernel/projects/rt/5.10/older/patch-${PREEMPT_RT_PATCH_VERSION}.patch.xz"
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
    xzcat "../patch-${PREEMPT_RT_PATCH_VERSION}.patch.xz" | patch -p1
    
    # Verify patch application
    if grep -q "PREEMPT_RT" Makefile; then
        log "INFO" "PREEMPT_RT patch applied successfully."
    else
        log "ERROR" "Failed to apply PREEMPT_RT patch."
        exit 1
    fi
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
    
    # Disable features that can cause latency spikes
    scripts/config --disable CPU_FREQ_STAT
    scripts/config --disable SCHED_DEBUG
    scripts/config --disable DEBUG_PREEMPT
    
    # Enable real-time group scheduling
    scripts/config --enable RT_GROUP_SCHED
    
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

# Function to extract and modify Orange Pi OS image
extract_and_modify_image() {
    log "INFO" "Extracting and modifying Orange Pi OS image..."
    
    cd "${BUILD_DIR}"
    
    # Extract the image
    if [ ! -f "${ORANGEPI_IMAGE%.xz}" ]; then
        log "INFO" "Extracting Orange Pi OS image..."
        xz -d -k "${ORANGEPI_IMAGE}"
    else
        log "INFO" "Orange Pi OS image already extracted."
    fi
    
    # Copy image to output directory
    cp "${ORANGEPI_IMAGE%.xz}" "${OUTPUT_DIR}/orangepi_vr_os.img"
    
    # Mount image
    LOOP_DEVICE=$(sudo losetup -f)
    sudo losetup -P ${LOOP_DEVICE} "${OUTPUT_DIR}/orangepi_vr_os.img"
    
    # Create mount points
    mkdir -p "${BUILD_DIR}/mnt/boot"
    mkdir -p "${BUILD_DIR}/mnt/rootfs"
    
    # Mount partitions
    sudo mount ${LOOP_DEVICE}p1 "${BUILD_DIR}/mnt/boot"
    sudo mount ${LOOP_DEVICE}p2 "${BUILD_DIR}/mnt/rootfs"
    
    log "INFO" "Orange Pi OS image extracted and mounted successfully."
}

# Function to install RT kernel to image
install_rt_kernel_to_image() {
    log "INFO" "Installing RT kernel to image..."
    
    # Copy kernel
    sudo cp "${BUILD_DIR}/linux-${KERNEL_VERSION}/arch/arm64/boot/Image" "${BUILD_DIR}/mnt/boot/"
    
    # Copy device tree
    sudo cp "${BUILD_DIR}/linux-${KERNEL_VERSION}/arch/arm64/boot/dts/rockchip/rk3588s-orangepi-cm5.dtb" "${BUILD_DIR}/mnt/boot/"
    
    # Copy modules
    sudo make -C "${BUILD_DIR}/linux-${KERNEL_VERSION}" ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- INSTALL_MOD_PATH="${BUILD_DIR}/mnt/rootfs" modules_install
    
    # Update module dependencies
    sudo depmod -a -b "${BUILD_DIR}/mnt/rootfs" $(cat ${BUILD_DIR}/linux-${KERNEL_VERSION}/include/config/kernel.release)
    
    log "INFO" "RT kernel installed to image successfully."
}

# Function to configure OS for VR
configure_os_for_vr() {
    log "INFO" "Configuring OS for VR..."
    
    # Configure boot parameters
    sudo bash -c "cat > ${BUILD_DIR}/mnt/boot/orangepiEnv.txt << EOF
verbosity=7
bootlogo=false
overlay_prefix=rockchip
rootdev=UUID=\$(blkid -s UUID -o value \${LOOP_DEVICE}p2)
rootfstype=ext4
overlays=rk3588s-orangepi-cm5-vr
extraargs=cma=1G hugepagesz=2M hugepages=512 isolcpus=0,1 nohz_full=0,1 rcu_nocbs=0,1
EOF"
    
    # Create real-time process configuration
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/security/limits.d/99-realtime.conf << EOF
# Real-time process configuration for VR
*               -       rtprio          99
*               -       nice            -20
*               -       memlock         unlimited
@realtime       -       rtprio          99
@realtime       -       nice            -20
@realtime       -       memlock         unlimited
EOF"
    
    # Create realtime group
    sudo chroot "${BUILD_DIR}/mnt/rootfs" /bin/bash -c "groupadd -f realtime"
    
    # Configure CPU affinity for system services
    sudo mkdir -p "${BUILD_DIR}/mnt/rootfs/etc/systemd/system.conf.d/"
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/systemd/system.conf.d/10-cpu-affinity.conf << EOF
[Manager]
CPUAffinity=2-7
EOF"
    
    # Configure memory management
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/sysctl.d/99-vr-performance.conf << EOF
# Memory management settings for VR
vm.swappiness=10
kernel.sched_min_granularity_ns=10000000
kernel.sched_wakeup_granularity_ns=15000000
vm.dirty_ratio=10
vm.dirty_background_ratio=5
EOF"
    
    # Configure network stack for low latency
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/sysctl.d/99-network-performance.conf << EOF
# Network performance settings for VR
net.core.rmem_max=16777216
net.core.wmem_max=16777216
net.ipv4.tcp_rmem=4096 87380 16777216
net.ipv4.tcp_wmem=4096 65536 16777216
net.ipv4.tcp_congestion_control=bbr
net.core.netdev_max_backlog=5000
net.ipv4.tcp_fastopen=3
EOF"
    
    # Create VR configuration directory
    sudo mkdir -p "${BUILD_DIR}/mnt/rootfs/etc/vr"
    
    # Create default VR configuration
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/vr/config.json << EOF
{
  \"system\": {
    \"rt_priority\": 80,
    \"cpu_affinity\": [0, 1],
    \"memory_limit\": \"8G\",
    \"gpu_performance_mode\": true
  },
  \"display\": {
    \"refresh_rate\": 90,
    \"persistence_time\": 2,
    \"vsync\": true,
    \"low_latency_mode\": true
  },
  \"tracking\": {
    \"camera_fps\": 90,
    \"imu_rate\": 1000,
    \"fusion_algorithm\": \"vins\",
    \"prediction_time_ms\": 10
  },
  \"network\": {
    \"latency_optimization\": true,
    \"bandwidth_reservation\": \"20M\",
    \"qos_enabled\": true
  },
  \"tpu\": {
    \"performance_mode\": true,
    \"zero_copy\": true,
    \"model_cache_size\": \"1G\"
  }
}
EOF"
    
    # Disable unnecessary services
    SERVICES_TO_DISABLE=(
      "apt-daily.service"
      "apt-daily-upgrade.service"
      "apt-daily.timer"
      "apt-daily-upgrade.timer"
      "bluetooth.service"
      "ModemManager.service"
      "networkd-dispatcher.service"
      "systemd-timesyncd.service"
      "snapd.service"
      "snapd.socket"
    )
    
    for service in "${SERVICES_TO_DISABLE[@]}"; do
      sudo chroot "${BUILD_DIR}/mnt/rootfs" /bin/bash -c "systemctl disable $service || true"
    done
    
    # Reduce logging
    sudo sed -i 's/#Storage=auto/Storage=volatile/' "${BUILD_DIR}/mnt/rootfs/etc/systemd/journald.conf"
    sudo sed -i 's/#RuntimeMaxUse=/RuntimeMaxUse=64M/' "${BUILD_DIR}/mnt/rootfs/etc/systemd/journald.conf"
    
    log "INFO" "OS configured for VR successfully."
}

# Function to create VR initialization script
create_vr_init_script() {
    log "INFO" "Creating VR initialization script..."
    
    # Create VR runtime directory
    sudo mkdir -p "${BUILD_DIR}/mnt/rootfs/opt/vr/bin"
    
    # Create VR initialization script
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/opt/vr/bin/vr-init << EOF
#!/bin/bash
# VR initialization script

# Set CPU governor to performance
for cpu in /sys/devices/system/cpu/cpu[0-7]; do
  echo performance > \\\$cpu/cpufreq/scaling_governor
done

# Set GPU to performance mode
echo performance > /sys/class/devfreq/ff9a0000.gpu/governor

# Load VR drivers
modprobe ov9281 || true
modprobe bno085 || true
modprobe iwlwifi || true
modprobe gasket || true

# Initialize VR devices
echo \"VR system initialized\"
EOF"
    
    # Make script executable
    sudo chmod +x "${BUILD_DIR}/mnt/rootfs/opt/vr/bin/vr-init"
    
    # Create VR service
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/systemd/system/vr-init.service << EOF
[Unit]
Description=VR Initialization Service
After=network.target

[Service]
Type=oneshot
ExecStart=/opt/vr/bin/vr-init
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF"
    
    # Enable VR service
    sudo chroot "${BUILD_DIR}/mnt/rootfs" /bin/bash -c "systemctl enable vr-init.service"
    
    log "INFO" "VR initialization script created successfully."
}

# Function to finalize image
finalize_image() {
    log "INFO" "Finalizing image..."
    
    # Unmount partitions
    sudo umount "${BUILD_DIR}/mnt/boot"
    sudo umount "${BUILD_DIR}/mnt/rootfs"
    
    # Detach loop device
    sudo losetup -d ${LOOP_DEVICE}
    
    # Compress output image
    cd "${OUTPUT_DIR}"
    xz -z -9 -k "orangepi_vr_os.img"
    
    log "INFO" "Image finalized successfully: ${OUTPUT_DIR}/orangepi_vr_os.img.xz"
}

# Function to create documentation
create_documentation() {
    log "INFO" "Creating documentation..."
    
    # Create documentation directory
    mkdir -p "${OUTPUT_DIR}/docs"
    
    # Create README
    cat > "${OUTPUT_DIR}/docs/README.md" << EOF
# Orange Pi CM5 VR OS

This is a custom Orange Pi OS image with PREEMPT_RT patches for VR applications.

## Image Details

- Base OS: Orange Pi OS (${ORANGEPI_VERSION})
- Kernel Version: ${KERNEL_VERSION}
- PREEMPT_RT Patch: ${PREEMPT_RT_PATCH_VERSION}

## VR-Specific Features

- PREEMPT_RT real-time kernel for low-latency operation
- CPU isolation for critical VR threads (cores 0-1)
- Optimized memory management for 16GB RAM
- Real-time process priority configuration
- Network stack optimization for low latency
- VR initialization service

## Installation

1. Flash the image to an SD card:
   \`\`\`
   xzcat orangepi_vr_os.img.xz | sudo dd of=/dev/sdX bs=4M status=progress
   \`\`\`

2. Insert the SD card into the Orange Pi CM5 and boot.

## Configuration

The system is pre-configured for VR operation, but you can adjust settings in:

- \`/boot/orangepiEnv.txt\` - Boot configuration
- \`/etc/vr/config.json\` - VR-specific settings

## Troubleshooting

If you encounter issues, check the following:

- System logs: \`journalctl -b\`
- Kernel logs: \`dmesg\`
- VR initialization logs: \`journalctl -u vr-init.service\`
EOF
    
    log "INFO" "Documentation created successfully."
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

# Main function
main() {
    log "INFO" "Starting Orange Pi OS setup with PREEMPT_RT..."
    
    # Run build steps
    run_step "Install Dependencies" install_dependencies
    run_step "Download Orange Pi OS" download_orangepi_os
    run_step "Download Linux Kernel" download_kernel
    run_step "Download PREEMPT_RT Patch" download_preempt_rt_patch
    run_step "Apply PREEMPT_RT Patch" apply_preempt_rt_patch
    run_step "Configure Kernel for VR" configure_kernel_for_vr
    run_step "Build Kernel" build_kernel
    run_step "Extract and Modify Image" extract_and_modify_image
    run_step "Install RT Kernel to Image" install_rt_kernel_to_image
    run_step "Configure OS for VR" configure_os_for_vr
    run_step "Create VR Initialization Script" create_vr_init_script
    run_step "Finalize Image" finalize_image
    run_step "Create Documentation" create_documentation
    
    log "INFO" "Orange Pi OS setup with PREEMPT_RT completed successfully."
    log "INFO" "Output image: ${OUTPUT_DIR}/orangepi_vr_os.img.xz"
    log "INFO" "Documentation: ${OUTPUT_DIR}/docs/README.md"
}

# Run main function
main "$@"
