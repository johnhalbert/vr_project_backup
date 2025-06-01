# Orange Pi CM5 Build System Configuration

This file contains the configuration parameters for the Orange Pi CM5 VR headset build system.

```bash
# Build system configuration
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
IMU_DRIVER_ENABLED=true
IMU_DRIVER_VERSION="1.0.0"
DISPLAY_DRIVER_ENABLED=true
DISPLAY_DRIVER_VERSION="1.0.0"
WIFI_DRIVER_ENABLED=true
WIFI_DRIVER_VERSION="1.0.0"
TPU_DRIVER_ENABLED=true
TPU_DRIVER_VERSION="1.0.0"

# Output image settings
OUTPUT_IMAGE_NAME="orangepi_vr_headset.img"
OUTPUT_IMAGE_COMPRESS=true
OUTPUT_IMAGE_COMPRESSION_LEVEL=9

# Documentation settings
DOCS_ENABLED=true
DOCS_OUTPUT_DIR="${OUTPUT_DIR}/docs"
```

This configuration file is sourced by the build.sh script and defines all the parameters used during the build process. You can modify these settings to customize the build according to your requirements.

## Configuration Parameters Explanation

### Build System Configuration
- `BUILD_SYSTEM_VERSION`: Version of the build system
- `BUILD_DATE`: Automatically set to the current date

### Orange Pi OS Configuration
- `ORANGEPI_VERSION`: Version of the Orange Pi OS to use
- `ORANGEPI_MIRROR`: URL of the Orange Pi OS mirror
- `ORANGEPI_IMAGE`: Filename of the Orange Pi OS image
- `ORANGEPI_BUILD_REPO`: URL of the Orange Pi build system repository
- `ORANGEPI_BUILD_BRANCH`: Branch of the Orange Pi build system repository

### Kernel Configuration
- `KERNEL_VERSION`: Version of the Linux kernel to use
- `KERNEL_MIRROR`: URL of the kernel mirror
- `KERNEL_SOURCE`: Filename of the kernel source
- `PREEMPT_RT_PATCH_VERSION`: Version of the PREEMPT_RT patch
- `PREEMPT_RT_MIRROR`: URL of the PREEMPT_RT patch mirror
- `PREEMPT_RT_PATCH`: Filename of the PREEMPT_RT patch

### Build Directories
- `BUILD_DIR`: Directory for building the system
- `OUTPUT_DIR`: Directory for output files
- `TOOLCHAIN_DIR`: Directory for the cross-compilation toolchain
- `LOG_DIR`: Directory for log files

### Compilation Settings
- `THREADS`: Number of threads to use for compilation
- `CROSS_COMPILE`: Cross-compilation prefix
- `ARCH`: Target architecture

### VR-Specific Settings
- `VR_MODE_ENABLED`: Whether to enable VR mode
- `VR_CPU_ISOLATION`: CPUs to isolate for VR processing
- `VR_MEMORY_RESERVATION`: Memory to reserve for VR processing
- `VR_DMA_BUFFER_SIZE`: Size of DMA buffers for VR
- `VR_LATENCY_TARGET_US`: Target latency in microseconds
- `VR_POWER_MODE`: Power mode for VR operation

### Driver Settings
- `CAMERA_DRIVER_ENABLED`: Whether to enable the camera driver
- `CAMERA_DRIVER_VERSION`: Version of the camera driver
- `IMU_DRIVER_ENABLED`: Whether to enable the IMU driver
- `IMU_DRIVER_VERSION`: Version of the IMU driver
- `DISPLAY_DRIVER_ENABLED`: Whether to enable the display driver
- `DISPLAY_DRIVER_VERSION`: Version of the display driver
- `WIFI_DRIVER_ENABLED`: Whether to enable the WiFi driver
- `WIFI_DRIVER_VERSION`: Version of the WiFi driver
- `TPU_DRIVER_ENABLED`: Whether to enable the TPU driver
- `TPU_DRIVER_VERSION`: Version of the TPU driver

### Output Image Settings
- `OUTPUT_IMAGE_NAME`: Name of the output image
- `OUTPUT_IMAGE_COMPRESS`: Whether to compress the output image
- `OUTPUT_IMAGE_COMPRESSION_LEVEL`: Compression level for the output image

### Documentation Settings
- `DOCS_ENABLED`: Whether to generate documentation
- `DOCS_OUTPUT_DIR`: Directory for documentation output
