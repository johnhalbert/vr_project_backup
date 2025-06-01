#!/bin/bash
# Orange Pi CM5 Device Tree Modifications for VR
# This script implements device tree modifications for VR applications

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
DTS_DIR="${HOME}/orb_slam3_project/drivers/orange_pi/device_tree"

# Print banner
echo -e "${BLUE}=======================================================${NC}"
echo -e "${BLUE}      Orange Pi CM5 Device Tree Modifications for VR   ${NC}"
echo -e "${BLUE}=======================================================${NC}"
echo -e "${GREEN}Build directory: ${BUILD_DIR}${NC}"
echo -e "${GREEN}Output directory: ${OUTPUT_DIR}${NC}"
echo -e "${BLUE}=======================================================${NC}"

# Create directories
mkdir -p "${BUILD_DIR}"
mkdir -p "${OUTPUT_DIR}"
mkdir -p "${LOG_DIR}"
mkdir -p "${BUILD_DIR}/dts"

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
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [${level}] ${message}" >> "${LOG_DIR}/device_tree_modifications.log"
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

# Function to check if device tree source exists
check_device_tree_source() {
    log "INFO" "Checking device tree source..."
    
    if [ ! -f "${DTS_DIR}/rk3588s-orangepi-cm5-vr.dts" ]; then
        log "ERROR" "Device tree source not found at ${DTS_DIR}/rk3588s-orangepi-cm5-vr.dts"
        exit 1
    fi
    
    log "INFO" "Device tree source found."
}

# Function to copy device tree source
copy_device_tree_source() {
    log "INFO" "Copying device tree source..."
    
    # Copy device tree source to build directory
    cp "${DTS_DIR}/rk3588s-orangepi-cm5-vr.dts" "${BUILD_DIR}/dts/"
    
    # Check if kernel device tree directory exists
    if [ -d "${BUILD_DIR}/mnt/rootfs/usr/src/linux-headers-$(uname -r)/arch/arm64/boot/dts/rockchip" ]; then
        # Copy device tree source to kernel directory
        sudo cp "${DTS_DIR}/rk3588s-orangepi-cm5-vr.dts" "${BUILD_DIR}/mnt/rootfs/usr/src/linux-headers-$(uname -r)/arch/arm64/boot/dts/rockchip/"
    fi
    
    log "INFO" "Device tree source copied successfully."
}

# Function to compile device tree
compile_device_tree() {
    log "INFO" "Compiling device tree..."
    
    # Compile device tree
    dtc -I dts -O dtb -o "${BUILD_DIR}/dts/rk3588s-orangepi-cm5-vr.dtb" "${BUILD_DIR}/dts/rk3588s-orangepi-cm5-vr.dts"
    
    # Copy compiled device tree to boot partition
    sudo cp "${BUILD_DIR}/dts/rk3588s-orangepi-cm5-vr.dtb" "${BUILD_DIR}/mnt/boot/dtbs/rockchip/rk3588s-orangepi-cm5-vr.dtb"
    
    log "INFO" "Device tree compiled successfully."
}

# Function to update boot configuration
update_boot_configuration() {
    log "INFO" "Updating boot configuration..."
    
    # Update boot configuration to use the new device tree
    sudo sed -i 's/overlay_prefix=rockchip/overlay_prefix=rockchip\nfdtfile=rockchip\/rk3588s-orangepi-cm5-vr.dtb/' "${BUILD_DIR}/mnt/boot/orangepiEnv.txt"
    
    # Update overlays
    sudo sed -i 's/overlays=.*/overlays=rk3588s-orangepi-cm5-vr/' "${BUILD_DIR}/mnt/boot/orangepiEnv.txt"
    
    log "INFO" "Boot configuration updated successfully."
}

# Function to create device tree overlay
create_device_tree_overlay() {
    log "INFO" "Creating device tree overlay..."
    
    # Create device tree overlay source
    cat > "${BUILD_DIR}/dts/rk3588s-orangepi-cm5-vr-overlay.dts" << EOF
/dts-v1/;
/plugin/;

/ {
    compatible = "rockchip,rk3588s-orangepi-cm5";
    
    fragment@0 {
        target = <&reserved_memory>;
        __overlay__ {
            #address-cells = <2>;
            #size-cells = <2>;
            ranges;
            
            vr_reserved: vr@10000000 {
                compatible = "shared-dma-pool";
                reg = <0x0 0x10000000 0x0 0x10000000>;
                no-map;
            };
            
            cma_reserved: cma@20000000 {
                compatible = "shared-dma-pool";
                reusable;
                reg = <0x0 0x20000000 0x0 0x20000000>;
                linux,cma-default;
            };
        };
    };
    
    fragment@1 {
        target = <&i2c1>;
        __overlay__ {
            status = "okay";
            #address-cells = <1>;
            #size-cells = <0>;
            
            bno085: bno085@4a {
                compatible = "orangepi,bno085-vr";
                reg = <0x4a>;
                interrupt-parent = <&gpio1>;
                interrupts = <RK_PA0 IRQ_TYPE_LEVEL_HIGH>;
                reset-gpios = <&gpio1 RK_PA1 GPIO_ACTIVE_LOW>;
                status = "okay";
            };
        };
    };
    
    fragment@2 {
        target = <&i2c5>;
        __overlay__ {
            status = "okay";
            #address-cells = <1>;
            #size-cells = <0>;
            
            ov9281_0: ov9281@60 {
                compatible = "orangepi,ov9281-vr";
                reg = <0x60>;
                clocks = <&cru CLK_MIPI_CAMAOUT_M1>;
                clock-names = "xvclk";
                power-domains = <&power RK3588_PD_VI>;
                pinctrl-names = "default";
                pinctrl-0 = <&cam_clk_m1>;
                reset-gpios = <&gpio1 RK_PB0 GPIO_ACTIVE_LOW>;
                pwdn-gpios = <&gpio1 RK_PB1 GPIO_ACTIVE_HIGH>;
                status = "okay";
                
                port {
                    ov9281_out0: endpoint {
                        remote-endpoint = <&mipi_in_ov9281_0>;
                        data-lanes = <1 2>;
                    };
                };
            };
            
            ov9281_1: ov9281@61 {
                compatible = "orangepi,ov9281-vr";
                reg = <0x61>;
                clocks = <&cru CLK_MIPI_CAMAOUT_M2>;
                clock-names = "xvclk";
                power-domains = <&power RK3588_PD_VI>;
                pinctrl-names = "default";
                pinctrl-0 = <&cam_clk_m2>;
                reset-gpios = <&gpio1 RK_PB2 GPIO_ACTIVE_LOW>;
                pwdn-gpios = <&gpio1 RK_PB3 GPIO_ACTIVE_HIGH>;
                status = "okay";
                
                port {
                    ov9281_out1: endpoint {
                        remote-endpoint = <&mipi_in_ov9281_1>;
                        data-lanes = <1 2>;
                    };
                };
            };
        };
    };
    
    fragment@3 {
        target = <&mipi_csi2_0>;
        __overlay__ {
            status = "okay";
            #address-cells = <1>;
            #size-cells = <0>;
            
            ports {
                #address-cells = <1>;
                #size-cells = <0>;
                
                port@0 {
                    reg = <0>;
                    #address-cells = <1>;
                    #size-cells = <0>;
                    
                    mipi_in_ov9281_0: endpoint@0 {
                        reg = <0>;
                        remote-endpoint = <&ov9281_out0>;
                        data-lanes = <1 2>;
                    };
                };
                
                port@1 {
                    reg = <1>;
                    #address-cells = <1>;
                    #size-cells = <0>;
                    
                    mipi_csi2_0_out: endpoint@0 {
                        reg = <0>;
                        remote-endpoint = <&csi2_0_in>;
                    };
                };
            };
        };
    };
    
    fragment@4 {
        target = <&mipi_csi2_1>;
        __overlay__ {
            status = "okay";
            #address-cells = <1>;
            #size-cells = <0>;
            
            ports {
                #address-cells = <1>;
                #size-cells = <0>;
                
                port@0 {
                    reg = <0>;
                    #address-cells = <1>;
                    #size-cells = <0>;
                    
                    mipi_in_ov9281_1: endpoint@0 {
                        reg = <0>;
                        remote-endpoint = <&ov9281_out1>;
                        data-lanes = <1 2>;
                    };
                };
                
                port@1 {
                    reg = <1>;
                    #address-cells = <1>;
                    #size-cells = <0>;
                    
                    mipi_csi2_1_out: endpoint@0 {
                        reg = <0>;
                        remote-endpoint = <&csi2_1_in>;
                    };
                };
            };
        };
    };
    
    fragment@5 {
        target = <&rkvdec>;
        __overlay__ {
            status = "okay";
            memory-region = <&vr_reserved>;
        };
    };
    
    fragment@6 {
        target = <&rkvenc>;
        __overlay__ {
            status = "okay";
            memory-region = <&vr_reserved>;
        };
    };
    
    fragment@7 {
        target = <&vop>;
        __overlay__ {
            status = "okay";
            compatible = "orangepi,rk3588-vop-vr";
            memory-region = <&vr_reserved>;
        };
    };
    
    fragment@8 {
        target = <&pcie2x1l0>;
        __overlay__ {
            status = "okay";
            reset-gpios = <&gpio4 RK_PA2 GPIO_ACTIVE_HIGH>;
            vpcie3v3-supply = <&vcc3v3_pcie>;
            
            pcie-coral-tpu {
                compatible = "orangepi,coral-tpu-vr";
                status = "okay";
            };
        };
    };
    
    fragment@9 {
        target = <&pcie2x1l1>;
        __overlay__ {
            status = "okay";
            reset-gpios = <&gpio4 RK_PA4 GPIO_ACTIVE_HIGH>;
            vpcie3v3-supply = <&vcc3v3_pcie>;
            
            pcie-intel-ax210 {
                compatible = "orangepi,intel-ax210-vr";
                status = "okay";
            };
        };
    };
    
    fragment@10 {
        target = <&cpu_l0>;
        __overlay__ {
            cpu-supply = <&vdd_cpu_lit_s0>;
            operating-points-v2 = <&cluster0_opp>;
            vr-cores;
        };
    };
    
    fragment@11 {
        target = <&cpu_l1>;
        __overlay__ {
            cpu-supply = <&vdd_cpu_lit_s0>;
            operating-points-v2 = <&cluster0_opp>;
            vr-cores;
        };
    };
    
    fragment@12 {
        target = <&cpu_l2>;
        __overlay__ {
            cpu-supply = <&vdd_cpu_lit_s0>;
            operating-points-v2 = <&cluster0_opp>;
        };
    };
    
    fragment@13 {
        target = <&cpu_l3>;
        __overlay__ {
            cpu-supply = <&vdd_cpu_lit_s0>;
            operating-points-v2 = <&cluster0_opp>;
        };
    };
    
    fragment@14 {
        target = <&cluster0_opp>;
        __overlay__ {
            opp-2400000000 {
                opp-hz = /bits/ 64 <2400000000>;
                opp-microvolt = <950000 950000 950000>;
                clock-latency-ns = <40000>;
            };
        };
    };
    
    fragment@15 {
        target = <&dsi0>;
        __overlay__ {
            status = "okay";
            #address-cells = <1>;
            #size-cells = <0>;
            
            vr_panel0: panel@0 {
                compatible = "orangepi,vr-display";
                reg = <0>;
                backlight = <&backlight>;
                status = "okay";
                
                port {
                    panel0_in: endpoint {
                        remote-endpoint = <&dsi0_out>;
                    };
                };
            };
            
            ports {
                #address-cells = <1>;
                #size-cells = <0>;
                
                port@1 {
                    reg = <1>;
                    dsi0_out: endpoint {
                        remote-endpoint = <&panel0_in>;
                    };
                };
            };
        };
    };
    
    fragment@16 {
        target = <&dsi1>;
        __overlay__ {
            status = "okay";
            #address-cells = <1>;
            #size-cells = <0>;
            
            vr_panel1: panel@0 {
                compatible = "orangepi,vr-display";
                reg = <0>;
                backlight = <&backlight>;
                status = "okay";
                
                port {
                    panel1_in: endpoint {
                        remote-endpoint = <&dsi1_out>;
                    };
                };
            };
            
            ports {
                #address-cells = <1>;
                #size-cells = <0>;
                
                port@1 {
                    reg = <1>;
                    dsi1_out: endpoint {
                        remote-endpoint = <&panel1_in>;
                    };
                };
            };
        };
    };
};
EOF
    
    # Compile device tree overlay
    dtc -I dts -O dtb -o "${BUILD_DIR}/dts/rk3588s-orangepi-cm5-vr-overlay.dtbo" "${BUILD_DIR}/dts/rk3588s-orangepi-cm5-vr-overlay.dts"
    
    # Copy compiled device tree overlay to boot partition
    sudo mkdir -p "${BUILD_DIR}/mnt/boot/dtbs/rockchip/overlay"
    sudo cp "${BUILD_DIR}/dts/rk3588s-orangepi-cm5-vr-overlay.dtbo" "${BUILD_DIR}/mnt/boot/dtbs/rockchip/overlay/rk3588s-orangepi-cm5-vr.dtbo"
    
    log "INFO" "Device tree overlay created successfully."
}

# Function to create device tree compiler script
create_dtc_script() {
    log "INFO" "Creating device tree compiler script..."
    
    # Create device tree compiler script
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/usr/local/bin/vr-dtc << EOF
#!/bin/bash
# Device tree compiler script for VR

# Compile device tree
dtc -I dts -O dtb -o /boot/dtbs/rockchip/rk3588s-orangepi-cm5-vr.dtb /boot/dtbs/rockchip/rk3588s-orangepi-cm5-vr.dts

# Compile device tree overlay
dtc -I dts -O dtb -o /boot/dtbs/rockchip/overlay/rk3588s-orangepi-cm5-vr.dtbo /boot/dtbs/rockchip/overlay/rk3588s-orangepi-cm5-vr.dts

# Update boot configuration
sed -i 's/overlays=.*/overlays=rk3588s-orangepi-cm5-vr/' /boot/orangepiEnv.txt
EOF"
    
    # Make script executable
    sudo chmod +x "${BUILD_DIR}/mnt/rootfs/usr/local/bin/vr-dtc"
    
    log "INFO" "Device tree compiler script created successfully."
}

# Function to create device tree documentation
create_device_tree_documentation() {
    log "INFO" "Creating device tree documentation..."
    
    # Create device tree documentation
    sudo bash -c "cat > ${BUILD_DIR}/mnt/rootfs/usr/share/doc/vr/device_tree.md << EOF
# Orange Pi CM5 Device Tree for VR

This document describes the device tree configuration for VR applications on the Orange Pi CM5.

## Memory Reservations

The following memory reservations have been made:

- VR Reserved Memory: 256MB at 0x10000000
- CMA Reserved Memory: 512MB at 0x20000000

## IMU Configuration

The BNO085 IMU has been configured with the following settings:

- I2C Address: 0x4a
- Interrupt GPIO: GPIO1_A0
- Reset GPIO: GPIO1_A1
- Compatible String: orangepi,bno085-vr

## Camera Configuration

Two OV9281 cameras have been configured with the following settings:

- Camera 0:
  - I2C Address: 0x60
  - Reset GPIO: GPIO1_B0
  - Power Down GPIO: GPIO1_B1
  - MIPI CSI-2 Interface: mipi_csi2_0
  - Data Lanes: 1, 2
  - Compatible String: orangepi,ov9281-vr

- Camera 1:
  - I2C Address: 0x61
  - Reset GPIO: GPIO1_B2
  - Power Down GPIO: GPIO1_B3
  - MIPI CSI-2 Interface: mipi_csi2_1
  - Data Lanes: 1, 2
  - Compatible String: orangepi,ov9281-vr

## Display Configuration

Two displays have been configured with the following settings:

- Display 0:
  - DSI Interface: dsi0
  - Compatible String: orangepi,vr-display

- Display 1:
  - DSI Interface: dsi1
  - Compatible String: orangepi,vr-display

## PCIe Configuration

Two PCIe interfaces have been configured with the following settings:

- PCIe 2x1 L0:
  - Reset GPIO: GPIO4_A2
  - Device: Coral TPU
  - Compatible String: orangepi,coral-tpu-vr

- PCIe 2x1 L1:
  - Reset GPIO: GPIO4_A4
  - Device: Intel AX210
  - Compatible String: orangepi,intel-ax210-vr

## CPU Configuration

The following CPU cores have been configured for VR:

- CPU L0 (Core 0): VR Core
- CPU L1 (Core 1): VR Core
- CPU L2 (Core 2): Normal Core
- CPU L3 (Core 3): Normal Core

The VR cores have been configured with a fixed frequency of 2.4GHz.

## Video Processing Configuration

The following video processing units have been configured to use VR reserved memory:

- RKVDEC: Video Decoder
- RKVENC: Video Encoder
- VOP: Video Output Processor (compatible: orangepi,rk3588-vop-vr)

## Device Tree Overlay

A device tree overlay has been created to enable all VR-specific hardware. The overlay is loaded at boot time and can be enabled or disabled by modifying the overlays parameter in /boot/orangepiEnv.txt.

## Device Tree Compiler Script

A device tree compiler script has been created to recompile the device tree and overlay if needed. The script can be run with the following command:

\`\`\`bash
sudo vr-dtc
\`\`\`

## Performance Impact

These device tree modifications result in:

- Proper hardware initialization for VR
- Reserved memory for VR applications
- Dedicated CPU cores for VR processing
- Optimized video processing for VR
- Proper camera and display configuration for VR
EOF"
    
    # Copy device tree documentation to output directory
    mkdir -p "${OUTPUT_DIR}/docs"
    cp "${BUILD_DIR}/mnt/rootfs/usr/share/doc/vr/device_tree.md" "${OUTPUT_DIR}/docs/device_tree_modifications.md"
    
    log "INFO" "Device tree documentation created successfully."
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

# Function to run a modification step with error handling
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
    log "INFO" "Starting device tree modifications for VR..."
    
    # Run modification steps
    run_step "Check OS Image" check_os_image
    run_step "Mount OS Image" mount_os_image
    run_step "Check Device Tree Source" check_device_tree_source
    run_step "Copy Device Tree Source" copy_device_tree_source
    run_step "Compile Device Tree" compile_device_tree
    run_step "Update Boot Configuration" update_boot_configuration
    run_step "Create Device Tree Overlay" create_device_tree_overlay
    run_step "Create Device Tree Compiler Script" create_dtc_script
    run_step "Create Device Tree Documentation" create_device_tree_documentation
    run_step "Unmount OS Image" unmount_os_image
    
    log "INFO" "Device tree modifications for VR completed successfully."
    log "INFO" "Documentation: ${OUTPUT_DIR}/docs/device_tree_modifications.md"
}

# Run main function
main "$@"
