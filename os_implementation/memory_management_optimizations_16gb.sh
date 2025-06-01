#!/bin/bash
# Orange Pi CM5 Memory Management Optimizations for 16GB RAM
# This script implements memory management optimizations for VR applications

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration variables
BUILD_DIR="${HOME}/orangepi_os_build"
OUTPUT_DIR="${HOME}/orangepi_os_output"
LOG_DIR="${HOME}/orangepi_os_logs"

# Print banner
echo -e "${BLUE}=======================================================${NC}"
echo -e "${BLUE}  Orange Pi CM5 Memory Management Optimizations for 16GB RAM ${NC}"
echo -e "${BLUE}=======================================================${NC}"
echo -e "${GREEN}Build directory: ${BUILD_DIR}${NC}"
echo -e "${GREEN}Output directory: ${OUTPUT_DIR}${NC}"
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
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [${level}] ${message}" >> "${LOG_DIR}/memory_management.log"
}

# Function to check if OS image exists
check_os_image() {
    log "INFO" "Checking OS image..."
    
    if [ ! -f "${OUTPUT_DIR}/orangepi_vr_os.img" ]; then
        log "ERROR" "OS image not found at ${OUTPUT_DIR}/orangepi_vr_os.img"
        log "ERROR" "Please run the OS setup script first."
        exit 1
    fi
    
    log "INFO" "OS image found."
}

# Function to mount OS image
mount_os_image() {
    log "INFO" "Mounting OS image..."
    
    # Create mount points
    mkdir -p "${BUILD_DIR}/mnt/boot"
    mkdir -p "${BUILD_DIR}/mnt/rootfs"
    
    # Mount image
    LOOP_DEVICE=$(sudo losetup -f)
    sudo losetup -P ${LOOP_DEVICE} "${OUTPUT_DIR}/orangepi_vr_os.img"
    
    # Mount partitions
    sudo mount ${LOOP_DEVICE}p1 "${BUILD_DIR}/mnt/boot"
    sudo mount ${LOOP_DEVICE}p2 "${BUILD_DIR}/mnt/rootfs"
    
    log "INFO" "OS image mounted successfully."
}

# Function to configure kernel memory parameters
configure_kernel_memory_parameters() {
    log "INFO" "Configuring kernel memory parameters..."
    
    # Create kernel memory parameters configuration
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/sysctl.d/99-memory-parameters.conf << EOF
# Memory parameters for VR headset with 16GB RAM
vm.swappiness=10
vm.vfs_cache_pressure=50
vm.dirty_ratio=10
vm.dirty_background_ratio=5
vm.dirty_writeback_centisecs=1500
vm.dirty_expire_centisecs=3000
vm.zone_reclaim_mode=0
vm.min_free_kbytes=262144
vm.overcommit_memory=1
vm.overcommit_ratio=80
vm.page-cluster=0
vm.stat_interval=10
vm.watermark_scale_factor=200
vm.compaction_proactiveness=1
vm.compact_unevictable_allowed=1
vm.page_lock_unfairness=1
vm.oom_dump_tasks=0
vm.oom_kill_allocating_task=1
vm.laptop_mode=0
vm.lowmem_reserve_ratio=256 256 32
vm.mmap_min_addr=65536
vm.max_map_count=1048576
EOF"
    
    log "INFO" "Kernel memory parameters configured successfully."
}

# Function to configure huge pages
configure_huge_pages() {
    log "INFO" "Configuring huge pages..."
    
    # Update kernel command line parameters
    sudo sed -i 's/hugepagesz=[^ ]*/hugepagesz=2M/' "${BUILD_DIR}/mnt/boot/orangepiEnv.txt"
    sudo sed -i 's/hugepages=[^ ]*/hugepages=2048/' "${BUILD_DIR}/mnt/boot/orangepiEnv.txt"
    
    # Add additional huge pages parameters
    if ! grep -q "transparent_hugepage" "${BUILD_DIR}/mnt/boot/orangepiEnv.txt"; then
        sudo sed -i 's/extraargs=/extraargs=transparent_hugepage=madvise /' "${BUILD_DIR}/mnt/boot/orangepiEnv.txt"
    fi
    
    # Create huge pages configuration
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/sysctl.d/99-hugepages.conf << EOF
# Huge pages settings for VR headset with 16GB RAM
vm.nr_hugepages=2048
vm.nr_overcommit_hugepages=1024
vm.hugetlb_shm_group=1001
EOF"
    
    # Create huge pages mount point
    sudo mkdir -p "${BUILD_DIR}/mnt/rootfs/mnt/huge"
    
    # Add huge pages mount to fstab
    sudo bash -c "echo 'hugetlbfs /mnt/huge hugetlbfs mode=1770,gid=1001 0 0' >> ${BUILD_DIR}/mnt/rootfs/etc/fstab"
    
    # Create huge pages initialization script
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/usr/local/bin/hugepages-init << EOF
#!/bin/bash
# Initialize huge pages for VR applications

# Create huge pages group if it doesn't exist
getent group hugepages > /dev/null || groupadd -g 1001 hugepages

# Set permissions on huge pages mount
chmod 1770 /mnt/huge
chgrp hugepages /mnt/huge

# Add VR user to huge pages group
if id -u vr > /dev/null 2>&1; then
  usermod -a -G hugepages vr
fi

# Configure transparent huge pages
echo madvise > /sys/kernel/mm/transparent_hugepage/enabled
echo madvise > /sys/kernel/mm/transparent_hugepage/defrag
echo 1 > /sys/kernel/mm/transparent_hugepage/khugepaged/defrag
echo 1 > /sys/kernel/mm/transparent_hugepage/khugepaged/alloc_sleep_millisecs
echo 60000 > /sys/kernel/mm/transparent_hugepage/khugepaged/scan_sleep_millisecs

# Reserve huge pages for VR applications
echo 2048 > /proc/sys/vm/nr_hugepages
EOF"
    
    # Make script executable
    sudo chmod +x "${BUILD_DIR}/mnt/rootfs/usr/local/bin/hugepages-init"
    
    # Create huge pages service
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/systemd/system/hugepages-init.service << EOF
[Unit]
Description=Initialize huge pages for VR applications
After=local-fs.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/hugepages-init
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF"
    
    # Enable huge pages service
    sudo chroot "${BUILD_DIR}/mnt/rootfs" /bin/bash -c "systemctl enable hugepages-init.service"
    
    log "INFO" "Huge pages configured successfully."
}

# Function to configure ZRAM
configure_zram() {
    log "INFO" "Configuring ZRAM..."
    
    # Create zram configuration
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/modules-load.d/zram.conf << EOF
zram
EOF"
    
    # Create zram setup script
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/usr/local/bin/zram-setup << EOF
#!/bin/bash
# Setup ZRAM for VR applications

# Remove any existing zram devices
swapoff /dev/zram0 2>/dev/null || true
echo 1 > /sys/block/zram0/reset 2>/dev/null || true

# Load zram module if not already loaded
modprobe zram

# Set up zram device
echo lz4 > /sys/block/zram0/comp_algorithm
echo 8G > /sys/block/zram0/disksize
mkswap /dev/zram0
swapon -p 100 /dev/zram0

# Configure swap parameters
echo 10 > /proc/sys/vm/swappiness
echo 0 > /proc/sys/vm/page-cluster
EOF"
    
    # Make script executable
    sudo chmod +x "${BUILD_DIR}/mnt/rootfs/usr/local/bin/zram-setup"
    
    # Create zram service
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/systemd/system/zram-setup.service << EOF
[Unit]
Description=Setup ZRAM for VR applications
After=local-fs.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/zram-setup
ExecStop=/bin/bash -c 'swapoff /dev/zram0 && echo 1 > /sys/block/zram0/reset'
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF"
    
    # Enable zram service
    sudo chroot "${BUILD_DIR}/mnt/rootfs" /bin/bash -c "systemctl enable zram-setup.service"
    
    log "INFO" "ZRAM configured successfully."
}

# Function to configure memory pressure handling
configure_memory_pressure_handling() {
    log "INFO" "Configuring memory pressure handling..."
    
    # Create memory pressure handling script
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/usr/local/bin/memory-pressure-handler << EOF
#!/bin/bash
# Handle memory pressure for VR applications

# Check memory usage
MEM_AVAILABLE=\$(grep MemAvailable /proc/meminfo | awk '{print \$2}')
MEM_TOTAL=\$(grep MemTotal /proc/meminfo | awk '{print \$2}')
MEM_PERCENT=\$(( \$MEM_AVAILABLE * 100 / \$MEM_TOTAL ))

# If memory usage is high (less than 20% available)
if [ \$MEM_PERCENT -lt 20 ]; then
  # Drop caches
  echo 1 > /proc/sys/vm/drop_caches
  
  # Compact memory
  echo 1 > /proc/sys/vm/compact_memory
  
  # Adjust OOM killer to prefer killing non-VR processes
  for pid in \$(pgrep -v 'vr|slam'); do
    if [ -f /proc/\$pid/oom_score_adj ]; then
      echo 500 > /proc/\$pid/oom_score_adj
    fi
  done
  
  # Protect VR processes from OOM killer
  for pid in \$(pgrep 'vr|slam'); do
    if [ -f /proc/\$pid/oom_score_adj ]; then
      echo -1000 > /proc/\$pid/oom_score_adj
    fi
  done
  
  # Clean up temporary files
  find /tmp -type f -atime +1 -delete
  find /var/tmp -type f -atime +1 -delete
  
  # Clean up VR cache
  find /var/lib/vr/cache -type f -atime +1 -delete
  
  # Log memory pressure event
  echo "[WARNING] Memory pressure detected: \${MEM_PERCENT}% available" >> /var/log/vr/memory-pressure.log
fi
EOF"
    
    # Make script executable
    sudo chmod +x "${BUILD_DIR}/mnt/rootfs/usr/local/bin/memory-pressure-handler"
    
    # Create memory pressure service
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/systemd/system/memory-pressure.service << EOF
[Unit]
Description=Memory Pressure Handler for VR
After=local-fs.target

[Service]
Type=simple
ExecStart=/bin/bash -c 'while true; do /usr/local/bin/memory-pressure-handler; sleep 10; done'
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF"
    
    # Create log directory
    sudo mkdir -p "${BUILD_DIR}/mnt/rootfs/var/log/vr"
    
    # Enable memory pressure service
    sudo chroot "${BUILD_DIR}/mnt/rootfs" /bin/bash -c "systemctl enable memory-pressure.service"
    
    log "INFO" "Memory pressure handling configured successfully."
}

# Function to configure CMA (Contiguous Memory Allocator)
configure_cma() {
    log "INFO" "Configuring CMA..."
    
    # Update kernel command line parameters
    sudo sed -i 's/cma=[^ ]*/cma=512M/' "${BUILD_DIR}/mnt/boot/orangepiEnv.txt"
    
    # Create CMA configuration script
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/usr/local/bin/cma-setup << EOF
#!/bin/bash
# Setup CMA for VR applications

# Set CMA permissions
if [ -f /dev/cma ]; then
  chmod 0660 /dev/cma
  chgrp video /dev/cma
fi

# Set DMA permissions
for dma in /dev/dma*; do
  if [ -c \$dma ]; then
    chmod 0660 \$dma
    chgrp video \$dma
  fi
done

# Add VR user to video group
if id -u vr > /dev/null 2>&1; then
  usermod -a -G video vr
fi
EOF"
    
    # Make script executable
    sudo chmod +x "${BUILD_DIR}/mnt/rootfs/usr/local/bin/cma-setup"
    
    # Create CMA service
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/systemd/system/cma-setup.service << EOF
[Unit]
Description=Setup CMA for VR applications
After=local-fs.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/cma-setup
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF"
    
    # Enable CMA service
    sudo chroot "${BUILD_DIR}/mnt/rootfs" /bin/bash -c "systemctl enable cma-setup.service"
    
    log "INFO" "CMA configured successfully."
}

# Function to configure memory allocation for VR
configure_vr_memory_allocation() {
    log "INFO" "Configuring memory allocation for VR..."
    
    # Create VR memory allocation script
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/usr/local/bin/vr-memory-allocation << EOF
#!/bin/bash
# Configure memory allocation for VR applications

# Create memory cgroup for VR
mkdir -p /sys/fs/cgroup/memory/vr
echo 12G > /sys/fs/cgroup/memory/vr/memory.limit_in_bytes
echo 12G > /sys/fs/cgroup/memory/vr/memory.soft_limit_in_bytes
echo 1 > /sys/fs/cgroup/memory/vr/memory.oom_control
echo 1 > /sys/fs/cgroup/memory/vr/memory.use_hierarchy

# Create memory cgroup for system
mkdir -p /sys/fs/cgroup/memory/system
echo 4G > /sys/fs/cgroup/memory/system/memory.limit_in_bytes
echo 4G > /sys/fs/cgroup/memory/system/memory.soft_limit_in_bytes
echo 0 > /sys/fs/cgroup/memory/system/memory.oom_control

# Move VR processes to VR cgroup
for pid in \$(pgrep 'vr|slam'); do
  echo \$pid > /sys/fs/cgroup/memory/vr/tasks
done

# Move system processes to system cgroup
for pid in \$(ps -eo pid,comm | grep -v -E 'vr|slam' | awk '{print \$1}' | grep -v "PID"); do
  echo \$pid > /sys/fs/cgroup/memory/system/tasks 2>/dev/null || true
done

# Configure memory locking limits
ulimit -l unlimited

# Configure mlock for VR processes
for pid in \$(pgrep 'vr|slam'); do
  if [ -d /proc/\$pid ]; then
    # Use process_mlock if available (newer kernels)
    if [ -f /proc/\$pid/process_mlock ]; then
      echo 1 > /proc/\$pid/process_mlock 2>/dev/null || true
    fi
    
    # Use mlock syscall via command if available
    if command -v prlimit > /dev/null; then
      prlimit --pid=\$pid --memlock=unlimited:unlimited 2>/dev/null || true
    fi
  fi
done
EOF"
    
    # Make script executable
    sudo chmod +x "${BUILD_DIR}/mnt/rootfs/usr/local/bin/vr-memory-allocation"
    
    # Create VR memory allocation service
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/systemd/system/vr-memory-allocation.service << EOF
[Unit]
Description=Memory Allocation for VR
After=vr-init.service

[Service]
Type=simple
ExecStart=/bin/bash -c 'while true; do /usr/local/bin/vr-memory-allocation; sleep 30; done'
Restart=always
RestartSec=30

[Install]
WantedBy=multi-user.target
EOF"
    
    # Enable VR memory allocation service
    sudo chroot "${BUILD_DIR}/mnt/rootfs" /bin/bash -c "systemctl enable vr-memory-allocation.service"
    
    log "INFO" "Memory allocation for VR configured successfully."
}

# Function to configure memory defragmentation
configure_memory_defragmentation() {
    log "INFO" "Configuring memory defragmentation..."
    
    # Create memory defragmentation script
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/usr/local/bin/memory-defrag << EOF
#!/bin/bash
# Memory defragmentation for VR applications

# Check if defragmentation is needed
FREE_HUGE=\$(grep HugePages_Free /proc/meminfo | awk '{print \$2}')
TOTAL_HUGE=\$(grep HugePages_Total /proc/meminfo | awk '{print \$2}')
HUGE_PERCENT=\$(( \$FREE_HUGE * 100 / \$TOTAL_HUGE ))

# If less than 10% of huge pages are free, defragment memory
if [ \$HUGE_PERCENT -lt 10 ]; then
  # Compact memory
  echo 1 > /proc/sys/vm/compact_memory
  
  # Drop caches
  echo 1 > /proc/sys/vm/drop_caches
  
  # Log defragmentation event
  echo "[INFO] Memory defragmentation performed: \${HUGE_PERCENT}% huge pages free" >> /var/log/vr/memory-defrag.log
fi

# Proactively compact memory every hour
MINUTES=\$(date +%M)
if [ "\$MINUTES" = "00" ]; then
  echo 1 > /proc/sys/vm/compact_memory
  echo "[INFO] Scheduled memory compaction performed" >> /var/log/vr/memory-defrag.log
fi
EOF"
    
    # Make script executable
    sudo chmod +x "${BUILD_DIR}/mnt/rootfs/usr/local/bin/memory-defrag"
    
    # Create memory defragmentation service
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/systemd/system/memory-defrag.service << EOF
[Unit]
Description=Memory Defragmentation for VR
After=local-fs.target

[Service]
Type=simple
ExecStart=/bin/bash -c 'while true; do /usr/local/bin/memory-defrag; sleep 300; done'
Restart=always
RestartSec=300

[Install]
WantedBy=multi-user.target
EOF"
    
    # Enable memory defragmentation service
    sudo chroot "${BUILD_DIR}/mnt/rootfs" /bin/bash -c "systemctl enable memory-defrag.service"
    
    log "INFO" "Memory defragmentation configured successfully."
}

# Function to configure memory monitoring
configure_memory_monitoring() {
    log "INFO" "Configuring memory monitoring..."
    
    # Create memory monitoring script
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/usr/local/bin/memory-monitor << EOF
#!/bin/bash
# Memory monitoring for VR applications

# Create log directory
mkdir -p /var/log/vr

# Get memory statistics
MEM_TOTAL=\$(grep MemTotal /proc/meminfo | awk '{print \$2}')
MEM_FREE=\$(grep MemFree /proc/meminfo | awk '{print \$2}')
MEM_AVAILABLE=\$(grep MemAvailable /proc/meminfo | awk '{print \$2}')
MEM_BUFFERS=\$(grep Buffers /proc/meminfo | awk '{print \$2}')
MEM_CACHED=\$(grep '^Cached' /proc/meminfo | awk '{print \$2}')
SWAP_TOTAL=\$(grep SwapTotal /proc/meminfo | awk '{print \$2}')
SWAP_FREE=\$(grep SwapFree /proc/meminfo | awk '{print \$2}')
HUGE_TOTAL=\$(grep HugePages_Total /proc/meminfo | awk '{print \$2}')
HUGE_FREE=\$(grep HugePages_Free /proc/meminfo | awk '{print \$2}')
ZRAM_USED=\$(grep zram0 /proc/swaps | awk '{print \$3}')
ZRAM_COMP=\$(grep zram0 /proc/swaps | awk '{print \$4}')

# Calculate percentages
MEM_USED_PCT=\$(( (MEM_TOTAL - MEM_AVAILABLE) * 100 / MEM_TOTAL ))
SWAP_USED_PCT=\$(( (SWAP_TOTAL - SWAP_FREE) * 100 / SWAP_TOTAL ))
HUGE_USED_PCT=\$(( (HUGE_TOTAL - HUGE_FREE) * 100 / HUGE_TOTAL ))

# Log memory statistics
echo "[INFO] Memory: \${MEM_USED_PCT}% used (\$((MEM_TOTAL/1024))MB total, \$((MEM_AVAILABLE/1024))MB available)" >> /var/log/vr/memory-stats.log
echo "[INFO] Swap: \${SWAP_USED_PCT}% used (\$((SWAP_TOTAL/1024))MB total, \$((SWAP_FREE/1024))MB free)" >> /var/log/vr/memory-stats.log
echo "[INFO] Huge Pages: \${HUGE_USED_PCT}% used (\${HUGE_TOTAL} total, \${HUGE_FREE} free)" >> /var/log/vr/memory-stats.log
echo "[INFO] ZRAM: \${ZRAM_USED}KB used, \${ZRAM_COMP}KB compressed" >> /var/log/vr/memory-stats.log

# Check for memory issues
if [ \$MEM_USED_PCT -gt 90 ]; then
  echo "[WARNING] High memory usage: \${MEM_USED_PCT}%" >> /var/log/vr/memory-stats.log
fi

if [ \$SWAP_USED_PCT -gt 50 ]; then
  echo "[WARNING] High swap usage: \${SWAP_USED_PCT}%" >> /var/log/vr/memory-stats.log
fi

if [ \$HUGE_USED_PCT -gt 90 ]; then
  echo "[WARNING] High huge pages usage: \${HUGE_USED_PCT}%" >> /var/log/vr/memory-stats.log
fi

# Rotate log file if it gets too large
if [ -f /var/log/vr/memory-stats.log ]; then
  LOG_SIZE=\$(stat -c%s /var/log/vr/memory-stats.log)
  if [ \$LOG_SIZE -gt 1048576 ]; then
    mv /var/log/vr/memory-stats.log /var/log/vr/memory-stats.log.old
  fi
fi
EOF"
    
    # Make script executable
    sudo chmod +x "${BUILD_DIR}/mnt/rootfs/usr/local/bin/memory-monitor"
    
    # Create memory monitoring service
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/systemd/system/memory-monitor.service << EOF
[Unit]
Description=Memory Monitoring for VR
After=local-fs.target

[Service]
Type=simple
ExecStart=/bin/bash -c 'while true; do /usr/local/bin/memory-monitor; sleep 60; done'
Restart=always
RestartSec=60

[Install]
WantedBy=multi-user.target
EOF"
    
    # Enable memory monitoring service
    sudo chroot "${BUILD_DIR}/mnt/rootfs" /bin/bash -c "systemctl enable memory-monitor.service"
    
    log "INFO" "Memory monitoring configured successfully."
}

# Function to configure memory limits
configure_memory_limits() {
    log "INFO" "Configuring memory limits..."
    
    # Create memory limits configuration
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/security/limits.d/99-memory-limits.conf << EOF
# Memory limits for VR applications
*               soft    memlock         unlimited
*               hard    memlock         unlimited
@realtime       soft    memlock         unlimited
@realtime       hard    memlock         unlimited
@hugepages      soft    memlock         unlimited
@hugepages      hard    memlock         unlimited
EOF"
    
    # Create PAM configuration for memory limits
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/etc/pam.d/common-session << EOF
# /etc/pam.d/common-session - session-related modules common to all services
#
# This file is included from other service-specific PAM config files,
# and should contain a list of modules that define tasks to be performed
# at the start and end of sessions of *any* kind (both interactive and
# non-interactive).
#
# As of pam 1.0.1-6, this file is managed by pam-auth-update by default.
# To take advantage of this, it is recommended that you configure any
# local modules either before or after the default block, and use
# pam-auth-update to manage selection of other modules.  See
# pam-auth-update(8) for details.

# here are the per-package modules (the "Primary" block)
session [default=1]                     pam_permit.so
# here's the fallback if no module succeeds
session requisite                       pam_deny.so
# prime the stack with a positive return value
session required                        pam_permit.so
# and here are more per-package modules (the "Additional" block)
session required        pam_unix.so
session optional        pam_systemd.so
# end of pam-auth-update config
session required        pam_limits.so
EOF"
    
    log "INFO" "Memory limits configured successfully."
}

# Function to unmount OS image
unmount_os_image() {
    log "INFO" "Unmounting OS image..."
    
    # Unmount partitions
    sudo umount "${BUILD_DIR}/mnt/boot"
    sudo umount "${BUILD_DIR}/mnt/rootfs"
    
    # Detach loop device
    sudo losetup -d ${LOOP_DEVICE}
    
    log "INFO" "OS image unmounted successfully."
}

# Function to create documentation
create_documentation() {
    log "INFO" "Creating documentation..."
    
    # Create documentation directory
    mkdir -p "${OUTPUT_DIR}/docs"
    
    # Create README
    cat > "${OUTPUT_DIR}/docs/memory_management_optimizations.md" << EOF
# Orange Pi CM5 Memory Management Optimizations for 16GB RAM

This document describes the memory management optimizations for VR applications on the Orange Pi CM5 with 16GB RAM.

## Kernel Memory Parameters

The following kernel memory parameters have been optimized for VR applications:

- Swappiness reduced to 10
- VFS cache pressure reduced to 50
- Dirty ratio set to 10%
- Dirty background ratio set to 5%
- Zone reclaim mode disabled
- Minimum free memory set to 256MB
- Overcommit memory enabled
- Overcommit ratio set to 80%
- Page cluster set to 0
- Memory compaction proactiveness enabled
- OOM kill allocating task enabled
- Maximum map count increased to 1,048,576

## Huge Pages

Huge pages have been configured with the following settings:

- 2048 huge pages (2MB each, total 4GB)
- 1024 overcommit huge pages
- Huge pages group created
- Huge pages mounted at /mnt/huge
- Transparent huge pages set to 'madvise'
- Huge pages defragmentation enabled

## ZRAM

ZRAM has been configured with the following settings:

- 8GB compressed swap
- LZ4 compression algorithm
- Swap priority set to 100
- Swappiness set to 10
- Page cluster set to 0

## Memory Pressure Handling

Memory pressure handling has been implemented with the following features:

- Memory pressure detection
- Cache dropping
- Memory compaction
- OOM killer adjustment
- Temporary file cleanup
- VR cache cleanup
- Memory pressure logging

## CMA (Contiguous Memory Allocator)

CMA has been configured with the following settings:

- 512MB reserved for contiguous memory allocation
- CMA permissions set for VR access
- DMA permissions set for VR access
- VR user added to video group

## VR Memory Allocation

VR memory allocation has been configured with the following settings:

- 12GB memory limit for VR processes
- 4GB memory limit for system processes
- OOM control enabled for VR processes
- Memory locking enabled for VR processes
- VR processes moved to VR cgroup
- System processes moved to system cgroup

## Memory Defragmentation

Memory defragmentation has been implemented with the following features:

- Automatic defragmentation when huge pages are low
- Scheduled hourly memory compaction
- Memory defragmentation logging

## Memory Monitoring

Memory monitoring has been implemented with the following features:

- Memory usage monitoring
- Swap usage monitoring
- Huge pages usage monitoring
- ZRAM usage monitoring
- Memory issue detection
- Memory statistics logging

## Memory Limits

Memory limits have been configured with the following settings:

- Unlimited memory locking for all users
- Unlimited memory locking for realtime group
- Unlimited memory locking for hugepages group
- PAM configuration for memory limits

## Performance Impact

These optimizations result in:

- More efficient memory usage
- Reduced memory fragmentation
- Faster memory allocation
- Improved responsiveness under memory pressure
- Better handling of large memory allocations
- Reduced latency for VR applications
- More consistent performance
EOF
    
    log "INFO" "Documentation created at ${OUTPUT_DIR}/docs/memory_management_optimizations.md"
}

# Function to run an optimization step with error handling
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
    log "INFO" "Starting memory management optimizations for 16GB RAM..."
    
    # Run optimization steps
    run_step "Check OS Image" check_os_image
    run_step "Mount OS Image" mount_os_image
    run_step "Configure Kernel Memory Parameters" configure_kernel_memory_parameters
    run_step "Configure Huge Pages" configure_huge_pages
    run_step "Configure ZRAM" configure_zram
    run_step "Configure Memory Pressure Handling" configure_memory_pressure_handling
    run_step "Configure CMA" configure_cma
    run_step "Configure VR Memory Allocation" configure_vr_memory_allocation
    run_step "Configure Memory Defragmentation" configure_memory_defragmentation
    run_step "Configure Memory Monitoring" configure_memory_monitoring
    run_step "Configure Memory Limits" configure_memory_limits
    run_step "Unmount OS Image" unmount_os_image
    run_step "Create Documentation" create_documentation
    
    log "INFO" "Memory management optimizations for 16GB RAM completed successfully."
    log "INFO" "Documentation: ${OUTPUT_DIR}/docs/memory_management_optimizations.md"
}

# Run main function
main "$@"
