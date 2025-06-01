# Orange Pi CM5 VR Headset Build System

This directory contains the build system for the Orange Pi CM5 VR headset platform. The build system is designed to create a complete OS image with all necessary drivers and configurations for the VR headset project.

## Overview

The Orange Pi CM5 build system creates a custom OS image based on Orange Pi OS (Ubuntu 22.04) with the following enhancements:

- PREEMPT_RT real-time kernel patches for low-latency operation
- Custom device tree for VR headset hardware configuration
- Optimized memory management for 16GB RAM
- CPU isolation for critical VR threads
- Custom drivers for cameras, IMU, displays, WiFi, and TPU

## Files

- `build.sh` - Main build script that orchestrates the entire build process
- `config.sh` - Configuration file with all customizable parameters
- `README.md` - This documentation file

## Usage

To build the Orange Pi CM5 VR headset image:

1. Ensure you have all prerequisites installed:
   ```bash
   sudo apt-get update
   sudo apt-get install build-essential gcc-aarch64-linux-gnu g++-aarch64-linux-gnu bison flex libssl-dev libncurses-dev libelf-dev bc lzop device-tree-compiler u-boot-tools python3 python3-pip git wget curl rsync cpio file kmod unzip
   ```

2. Run the build script:
   ```bash
   ./build.sh
   ```

3. The output image will be created in the configured output directory (default: `~/orangepi_output`).

## Configuration

All build parameters can be customized by editing the `config.sh` file. Key configuration options include:

- `ORANGEPI_VERSION` - Version of Orange Pi OS to use
- `KERNEL_VERSION` - Linux kernel version
- `PREEMPT_RT_PATCH_VERSION` - PREEMPT_RT patch version
- `BUILD_DIR`, `OUTPUT_DIR` - Build and output directories
- `VR_CPU_ISOLATION` - CPU cores to isolate for VR processing
- `MEMORY_TOTAL`, `MEMORY_CMA_SIZE` - Memory configuration for 16GB RAM
- Driver-specific settings for each component

## Build Process

The build process consists of the following steps:

1. Install dependencies
2. Download Orange Pi OS
3. Download Linux kernel
4. Download PREEMPT_RT patch
5. Apply PREEMPT_RT patch
6. Copy device tree files
7. Configure kernel for VR
8. Build kernel
9. Build VR drivers
10. Create output image
11. Create documentation

Each step is executed in sequence, with comprehensive logging and error handling.

## Customization

To customize the build for specific hardware configurations:

1. Modify the device tree files in the `drivers/orange_pi/device_tree` directory
2. Adjust kernel configuration in the `configure_kernel_for_vr` function
3. Update driver settings in `config.sh`

## Troubleshooting

If the build fails, check the log file in the configured log directory (default: `~/orangepi_logs/build.log`).

Common issues:
- Insufficient disk space
- Missing dependencies
- Network connectivity problems
- Incompatible kernel or patch versions

## License

This build system is part of the VR headset project and is subject to the project's licensing terms.
