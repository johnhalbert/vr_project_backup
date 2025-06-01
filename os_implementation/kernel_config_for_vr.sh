#!/bin/bash
# Orange Pi CM5 Kernel Configuration for VR
# This script configures the Linux kernel for low-latency operation and CPU isolation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration variables
KERNEL_VERSION="5.10.110"
BUILD_DIR="${HOME}/orangepi_os_build"
LOG_DIR="${HOME}/orangepi_os_logs"

# Print banner
echo -e "${BLUE}=======================================================${NC}"
echo -e "${BLUE}      Orange Pi CM5 Kernel Configuration for VR        ${NC}"
echo -e "${BLUE}=======================================================${NC}"
echo -e "${GREEN}Kernel version: ${KERNEL_VERSION}${NC}"
echo -e "${GREEN}Build directory: ${BUILD_DIR}${NC}"
echo -e "${BLUE}=======================================================${NC}"

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
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [${level}] ${message}" >> "${LOG_DIR}/kernel_config.log"
}

# Function to check if kernel source exists
check_kernel_source() {
    log "INFO" "Checking kernel source..."
    
    if [ ! -d "${BUILD_DIR}/linux-${KERNEL_VERSION}" ]; then
        log "ERROR" "Kernel source not found at ${BUILD_DIR}/linux-${KERNEL_VERSION}"
        log "ERROR" "Please run the OS setup script first."
        exit 1
    fi
    
    log "INFO" "Kernel source found."
}

# Function to configure kernel for low-latency
configure_low_latency() {
    log "INFO" "Configuring kernel for low-latency operation..."
    
    cd "${BUILD_DIR}/linux-${KERNEL_VERSION}"
    
    # Ensure we have a config file
    if [ ! -f ".config" ]; then
        log "INFO" "No config file found, using Orange Pi default config..."
        cp -f "${BUILD_DIR}/orangepi-build/kernel/arch/arm64/configs/orangepi_defconfig" .config
    fi
    
    # Enable PREEMPT_RT
    log "INFO" "Enabling PREEMPT_RT..."
    scripts/config --enable PREEMPT
    scripts/config --set-val PREEMPT_RT y
    scripts/config --disable PREEMPT_VOLUNTARY
    scripts/config --disable PREEMPT_NONE
    
    # Enable high-resolution timers
    log "INFO" "Enabling high-resolution timers..."
    scripts/config --enable HIGH_RES_TIMERS
    scripts/config --enable NO_HZ_FULL
    scripts/config --enable RCU_NOCB_CPU
    
    # Disable features that can cause latency spikes
    log "INFO" "Disabling features that can cause latency spikes..."
    scripts/config --disable CPU_FREQ_STAT
    scripts/config --disable SCHED_DEBUG
    scripts/config --disable DEBUG_PREEMPT
    scripts/config --disable SLUB_DEBUG
    scripts/config --disable PROFILING
    scripts/config --disable KPROBES
    scripts/config --disable FTRACE
    
    # Configure scheduler for low-latency
    log "INFO" "Configuring scheduler for low-latency..."
    scripts/config --enable SCHED_AUTOGROUP
    scripts/config --enable RT_GROUP_SCHED
    scripts/config --enable CGROUP_SCHED
    
    # Configure interrupt handling
    log "INFO" "Configuring interrupt handling..."
    scripts/config --enable IRQ_TIME_ACCOUNTING
    scripts/config --enable IRQSOFF_TRACER
    scripts/config --enable PREEMPT_TRACER
    
    # Save the configuration
    yes "" | make ARCH=arm64 oldconfig
    
    log "INFO" "Kernel configured for low-latency operation."
}

# Function to configure CPU isolation
configure_cpu_isolation() {
    log "INFO" "Configuring CPU isolation..."
    
    cd "${BUILD_DIR}/linux-${KERNEL_VERSION}"
    
    # Enable CPU isolation
    log "INFO" "Enabling CPU isolation..."
    scripts/config --enable CPU_ISOLATION
    
    # Configure CPU frequency scaling
    log "INFO" "Configuring CPU frequency scaling..."
    scripts/config --enable CPU_FREQ
    scripts/config --enable CPU_FREQ_GOV_PERFORMANCE
    scripts/config --enable CPU_FREQ_GOV_USERSPACE
    scripts/config --disable CPU_FREQ_GOV_ONDEMAND
    scripts/config --disable CPU_FREQ_GOV_CONSERVATIVE
    
    # Configure CPU idle
    log "INFO" "Configuring CPU idle..."
    scripts/config --enable CPU_IDLE
    scripts/config --enable CPU_IDLE_GOV_LADDER
    scripts/config --enable CPU_IDLE_GOV_MENU
    
    # Save the configuration
    yes "" | make ARCH=arm64 oldconfig
    
    log "INFO" "CPU isolation configured."
}

# Function to configure memory management
configure_memory_management() {
    log "INFO" "Configuring memory management..."
    
    cd "${BUILD_DIR}/linux-${KERNEL_VERSION}"
    
    # Configure huge pages
    log "INFO" "Configuring huge pages..."
    scripts/config --enable TRANSPARENT_HUGEPAGE
    scripts/config --enable TRANSPARENT_HUGEPAGE_ALWAYS
    scripts/config --disable TRANSPARENT_HUGEPAGE_MADVISE
    
    # Configure CMA
    log "INFO" "Configuring CMA..."
    scripts/config --enable DMA_CMA
    scripts/config --set-val CMA_SIZE_MBYTES 1024
    
    # Configure memory allocator
    log "INFO" "Configuring memory allocator..."
    scripts/config --enable SLUB
    scripts/config --disable SLOB
    scripts/config --enable SLUB_CPU_PARTIAL
    
    # Configure memory compaction
    log "INFO" "Configuring memory compaction..."
    scripts/config --enable COMPACTION
    scripts/config --enable MIGRATION
    
    # Configure swap
    log "INFO" "Configuring swap..."
    scripts/config --enable SWAP
    scripts/config --enable FRONTSWAP
    scripts/config --enable ZSWAP
    
    # Save the configuration
    yes "" | make ARCH=arm64 oldconfig
    
    log "INFO" "Memory management configured."
}

# Function to configure device drivers for VR
configure_device_drivers() {
    log "INFO" "Configuring device drivers for VR..."
    
    cd "${BUILD_DIR}/linux-${KERNEL_VERSION}"
    
    # Configure V4L2 for cameras
    log "INFO" "Configuring V4L2 for cameras..."
    scripts/config --enable V4L_PLATFORM_DRIVERS
    scripts/config --enable VIDEO_V4L2
    scripts/config --enable VIDEO_OV9281
    scripts/config --enable MEDIA_CONTROLLER
    scripts/config --enable MEDIA_CAMERA_SUPPORT
    
    # Configure DRM for displays
    log "INFO" "Configuring DRM for displays..."
    scripts/config --enable DRM
    scripts/config --enable DRM_ROCKCHIP
    scripts/config --enable DRM_PANEL
    scripts/config --enable DRM_DISPLAY_CONNECTOR
    scripts/config --enable DRM_KMS_HELPER
    scripts/config --enable DRM_KMS_FB_HELPER
    
    # Configure IIO for IMU
    log "INFO" "Configuring IIO for IMU..."
    scripts/config --enable IIO
    scripts/config --enable IIO_BUFFER
    scripts/config --enable IIO_TRIGGERED_BUFFER
    scripts/config --enable IIO_KFIFO_BUF
    scripts/config --enable IIO_TRIGGER
    
    # Configure PCIe for TPU and WiFi
    log "INFO" "Configuring PCIe for TPU and WiFi..."
    scripts/config --enable PCI
    scripts/config --enable PCIEPORTBUS
    scripts/config --enable PCIE_ROCKCHIP_HOST
    scripts/config --enable PCI_MSI
    
    # Configure WiFi
    log "INFO" "Configuring WiFi..."
    scripts/config --enable IWLWIFI
    scripts/config --enable IWLMVM
    scripts/config --enable CFG80211
    scripts/config --enable MAC80211
    
    # Save the configuration
    yes "" | make ARCH=arm64 oldconfig
    
    log "INFO" "Device drivers configured for VR."
}

# Function to create kernel command line parameters
create_kernel_cmdline() {
    log "INFO" "Creating kernel command line parameters..."
    
    # Create kernel command line file
    cat > "${BUILD_DIR}/kernel_cmdline.txt" << EOF
# Kernel command line parameters for VR
console=ttyS2,1500000 console=tty1 root=PARTUUID=XXX rootwait rw earlycon=uart8250,mmio32,0xfeb50000
cma=1G
hugepagesz=2M
hugepages=512
isolcpus=0,1
nohz_full=0,1
rcu_nocbs=0,1
intel_pstate=disable
processor.max_cstate=1
idle=poll
nosoftlockup
nowatchdog
skew_tick=1
clocksource=tsc
tsc=reliable
EOF
    
    log "INFO" "Kernel command line parameters created at ${BUILD_DIR}/kernel_cmdline.txt"
}

# Function to create boot configuration
create_boot_config() {
    log "INFO" "Creating boot configuration..."
    
    # Create boot configuration file
    cat > "${BUILD_DIR}/orangepiEnv.txt" << EOF
verbosity=7
bootlogo=false
overlay_prefix=rockchip
rootdev=PARTUUID=XXX
rootfstype=ext4
overlays=rk3588s-orangepi-cm5-vr
extraargs=cma=1G hugepagesz=2M hugepages=512 isolcpus=0,1 nohz_full=0,1 rcu_nocbs=0,1 intel_pstate=disable processor.max_cstate=1 idle=poll nosoftlockup nowatchdog skew_tick=1 clocksource=tsc tsc=reliable
EOF
    
    log "INFO" "Boot configuration created at ${BUILD_DIR}/orangepiEnv.txt"
}

# Function to create sysctl configuration
create_sysctl_config() {
    log "INFO" "Creating sysctl configuration..."
    
    # Create sysctl configuration file
    cat > "${BUILD_DIR}/99-vr-performance.conf" << EOF
# Memory management settings for VR
vm.swappiness=10
kernel.sched_min_granularity_ns=10000000
kernel.sched_wakeup_granularity_ns=15000000
vm.dirty_ratio=10
vm.dirty_background_ratio=5
kernel.sched_rt_runtime_us=-1
kernel.sched_rt_period_us=1000000
kernel.sched_autogroup_enabled=0
kernel.hung_task_timeout_secs=0
kernel.watchdog_thresh=0
EOF
    
    log "INFO" "Sysctl configuration created at ${BUILD_DIR}/99-vr-performance.conf"
}

# Function to create CPU isolation service
create_cpu_isolation_service() {
    log "INFO" "Creating CPU isolation service..."
    
    # Create CPU isolation service file
    cat > "${BUILD_DIR}/vr-cpu-isolation.service" << EOF
[Unit]
Description=VR CPU Isolation Service
After=network.target

[Service]
Type=oneshot
ExecStart=/bin/bash -c "echo performance > /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor"
ExecStart=/bin/bash -c "echo performance > /sys/devices/system/cpu/cpu1/cpufreq/scaling_governor"
ExecStart=/bin/bash -c "echo 1 > /sys/devices/system/cpu/cpu0/cpuidle/state0/disable"
ExecStart=/bin/bash -c "echo 1 > /sys/devices/system/cpu/cpu1/cpuidle/state0/disable"
ExecStart=/bin/bash -c "echo 0 > /proc/sys/kernel/watchdog"
ExecStart=/bin/bash -c "echo 0 > /proc/sys/kernel/nmi_watchdog"
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF
    
    log "INFO" "CPU isolation service created at ${BUILD_DIR}/vr-cpu-isolation.service"
}

# Function to create documentation
create_documentation() {
    log "INFO" "Creating documentation..."
    
    # Create documentation directory
    mkdir -p "${BUILD_DIR}/docs"
    
    # Create README
    cat > "${BUILD_DIR}/docs/kernel_config_readme.md" << EOF
# Orange Pi CM5 Kernel Configuration for VR

This document describes the kernel configuration for VR applications on the Orange Pi CM5.

## Low-Latency Configuration

The kernel is configured for low-latency operation with the following features:

- PREEMPT_RT patch for full kernel preemption
- High-resolution timers for precise timing
- Disabled features that can cause latency spikes
- Optimized scheduler for real-time performance
- Configured interrupt handling for low latency

## CPU Isolation

CPU cores 0 and 1 are isolated for VR processing with the following configuration:

- CPU isolation enabled in kernel
- CPU frequency scaling set to performance governor
- CPU idle states disabled
- Watchdog disabled
- NMI watchdog disabled

## Memory Management

Memory management is optimized for VR with the following configuration:

- Transparent huge pages enabled
- CMA size set to 1GB
- Memory allocator optimized for performance
- Memory compaction enabled
- Swap optimized for performance

## Device Drivers

Device drivers are configured for VR with the following configuration:

- V4L2 configured for OV9281 cameras
- DRM configured for dual displays
- IIO configured for BNO085 IMU
- PCIe configured for Coral TPU and Intel AX210 WiFi

## Kernel Command Line Parameters

The following kernel command line parameters are used:

\`\`\`
cma=1G hugepagesz=2M hugepages=512 isolcpus=0,1 nohz_full=0,1 rcu_nocbs=0,1 intel_pstate=disable processor.max_cstate=1 idle=poll nosoftlockup nowatchdog skew_tick=1 clocksource=tsc tsc=reliable
\`\`\`

## Sysctl Configuration

The following sysctl parameters are used:

\`\`\`
vm.swappiness=10
kernel.sched_min_granularity_ns=10000000
kernel.sched_wakeup_granularity_ns=15000000
vm.dirty_ratio=10
vm.dirty_background_ratio=5
kernel.sched_rt_runtime_us=-1
kernel.sched_rt_period_us=1000000
kernel.sched_autogroup_enabled=0
kernel.hung_task_timeout_secs=0
kernel.watchdog_thresh=0
\`\`\`

## CPU Isolation Service

A systemd service is used to configure CPU isolation at boot:

\`\`\`
[Unit]
Description=VR CPU Isolation Service
After=network.target

[Service]
Type=oneshot
ExecStart=/bin/bash -c "echo performance > /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor"
ExecStart=/bin/bash -c "echo performance > /sys/devices/system/cpu/cpu1/cpufreq/scaling_governor"
ExecStart=/bin/bash -c "echo 1 > /sys/devices/system/cpu/cpu0/cpuidle/state0/disable"
ExecStart=/bin/bash -c "echo 1 > /sys/devices/system/cpu/cpu1/cpuidle/state0/disable"
ExecStart=/bin/bash -c "echo 0 > /proc/sys/kernel/watchdog"
ExecStart=/bin/bash -c "echo 0 > /proc/sys/kernel/nmi_watchdog"
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
\`\`\`
EOF
    
    log "INFO" "Documentation created at ${BUILD_DIR}/docs/kernel_config_readme.md"
}

# Function to run a configuration step with error handling
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
    log "INFO" "Starting kernel configuration for VR..."
    
    # Create log directory
    mkdir -p "${LOG_DIR}"
    
    # Run configuration steps
    run_step "Check Kernel Source" check_kernel_source
    run_step "Configure Low-Latency" configure_low_latency
    run_step "Configure CPU Isolation" configure_cpu_isolation
    run_step "Configure Memory Management" configure_memory_management
    run_step "Configure Device Drivers" configure_device_drivers
    run_step "Create Kernel Command Line" create_kernel_cmdline
    run_step "Create Boot Configuration" create_boot_config
    run_step "Create Sysctl Configuration" create_sysctl_config
    run_step "Create CPU Isolation Service" create_cpu_isolation_service
    run_step "Create Documentation" create_documentation
    
    log "INFO" "Kernel configuration for VR completed successfully."
    log "INFO" "Documentation: ${BUILD_DIR}/docs/kernel_config_readme.md"
}

# Run main function
main "$@"
