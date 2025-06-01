# Orange Pi CM5 Display Driver Adaptation

This file contains the necessary adaptations to the RK3588 display driver for the Orange Pi CM5 platform.

```c
// drivers/gpu/drm/rockchip/rockchip_drm_vop_orangepi.c

#include <linux/module.h>
#include <linux/of.h>
#include <linux/of_device.h>
#include <linux/of_graph.h>
#include <linux/component.h>
#include <linux/clk.h>
#include <linux/delay.h>
#include <linux/gpio/consumer.h>
#include <linux/regulator/consumer.h>
#include <drm/drm_atomic.h>
#include <drm/drm_atomic_helper.h>
#include <drm/drm_bridge.h>
#include <drm/drm_crtc.h>
#include <drm/drm_crtc_helper.h>
#include <drm/drm_encoder.h>
#include <drm/drm_of.h>
#include <drm/drm_panel.h>
#include <drm/drm_probe_helper.h>

// Include the original Rockchip DRM driver headers
#include "rockchip_drm_drv.h"
#include "rockchip_drm_vop.h"

// Orange Pi CM5 specific VR display configuration
struct orangepi_vr_display_config {
    bool dual_display;
    bool low_persistence;
    bool synchronized;
    u32 refresh_rate;
    u32 persistence_time_us;
};

// Orange Pi CM5 specific configuration
static int rockchip_vop_configure_orangepi_cm5(struct rockchip_vop *vop)
{
    struct device *dev = vop->dev;
    struct orangepi_vr_display_config *vr_config;
    int ret;

    dev_info(dev, "Configuring VOP for Orange Pi CM5 VR display\n");

    // Allocate VR configuration
    vr_config = devm_kzalloc(dev, sizeof(*vr_config), GFP_KERNEL);
    if (!vr_config)
        return -ENOMEM;

    // Parse VR-specific device tree properties
    if (of_property_read_bool(dev->of_node, "vr,dual-display"))
        vr_config->dual_display = true;

    if (of_property_read_bool(dev->of_node, "vr,low-persistence"))
        vr_config->low_persistence = true;

    if (of_property_read_bool(dev->of_node, "vr,synchronized"))
        vr_config->synchronized = true;

    of_property_read_u32(dev->of_node, "vr,refresh-rate", &vr_config->refresh_rate);
    of_property_read_u32(dev->of_node, "vr,persistence-time-us", &vr_config->persistence_time_us);

    // Set default values if not specified
    if (!vr_config->refresh_rate)
        vr_config->refresh_rate = 90;

    if (!vr_config->persistence_time_us)
        vr_config->persistence_time_us = 2000; // 2ms persistence time

    // Configure VOP for VR mode
    if (vr_config->dual_display) {
        // Configure for dual display mode
        ret = rockchip_vop_write_reg(vop, VOP_SYS_CTRL, VOP_DUAL_CHANNEL_EN, 1);
        if (ret)
            return ret;
    }

    if (vr_config->synchronized) {
        // Configure for synchronized display mode
        ret = rockchip_vop_write_reg(vop, VOP_SYS_CTRL, VOP_SYNC_MODE_EN, 1);
        if (ret)
            return ret;
    }

    if (vr_config->low_persistence) {
        // Configure for low persistence mode
        u32 total_lines, active_lines, blank_lines;
        u32 persistence_lines;

        // Calculate persistence lines based on refresh rate and persistence time
        total_lines = vop->mode.vtotal;
        active_lines = vop->mode.vdisplay;
        blank_lines = total_lines - active_lines;

        // Convert persistence time to lines
        persistence_lines = (vr_config->persistence_time_us * vr_config->refresh_rate) / 1000000;
        persistence_lines = min(persistence_lines, active_lines);

        // Configure persistence timing
        ret = rockchip_vop_write_reg(vop, VOP_DSP_CTRL, VOP_PERSISTENCE_LINES, persistence_lines);
        if (ret)
            return ret;
    }

    // Store VR configuration in private data
    vop->vr_config = vr_config;

    dev_info(dev, "VOP configured for Orange Pi CM5 VR display: %s, %s, %s, %dHz, %dus persistence\n",
             vr_config->dual_display ? "dual-display" : "single-display",
             vr_config->low_persistence ? "low-persistence" : "full-persistence",
             vr_config->synchronized ? "synchronized" : "independent",
             vr_config->refresh_rate,
             vr_config->persistence_time_us);

    return 0;
}

// Update probe function to detect Orange Pi CM5
static int rockchip_vop_probe_orangepi(struct platform_device *pdev)
{
    struct device *dev = &pdev->dev;
    struct rockchip_vop *vop;
    int ret;

    // Call original probe function
    ret = rockchip_vop_probe(pdev);
    if (ret)
        return ret;

    // Get the vop device pointer
    vop = platform_get_drvdata(pdev);
    if (!vop)
        return -ENODEV;

    // Check if this is an Orange Pi CM5 device
    if (of_device_is_compatible(dev->of_node, "orangepi,rk3588-vop-vr")) {
        dev_info(dev, "Detected Orange Pi CM5 VR display controller\n");
        
        // Apply Orange Pi CM5 specific configuration
        ret = rockchip_vop_configure_orangepi_cm5(vop);
        if (ret) {
            dev_err(dev, "Failed to configure for Orange Pi CM5: %d\n", ret);
            return ret;
        }
        
        // Set Orange Pi CM5 specific flags
        vop->is_orangepi_cm5 = true;
    }

    return 0;
}

// Update the compatible strings to include Orange Pi variant
static const struct of_device_id rockchip_vop_dt_ids_orangepi[] = {
    { .compatible = "rockchip,rk3588-vop" },
    { .compatible = "orangepi,rk3588-vop-vr" },
    { /* sentinel */ }
};
MODULE_DEVICE_TABLE(of, rockchip_vop_dt_ids_orangepi);

// Update the platform_driver structure
static struct platform_driver rockchip_vop_driver_orangepi = {
    .probe = rockchip_vop_probe_orangepi,
    .remove = rockchip_vop_remove,
    .driver = {
        .name = "rockchip-vop-orangepi",
        .of_match_table = rockchip_vop_dt_ids_orangepi,
    },
};

module_platform_driver(rockchip_vop_driver_orangepi);

MODULE_DESCRIPTION("Rockchip VOP Driver for Orange Pi CM5 VR Display");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
```

## Integration with Existing Driver

To integrate this adaptation with the existing Rockchip DRM driver, we need to:

1. Add the Orange Pi CM5 specific configuration to the existing driver
2. Update the device tree bindings to include the Orange Pi CM5 compatible string
3. Add the Orange Pi CM5 specific flags and VR configuration to the driver structure

## Device Tree Binding Updates

```
Required properties for Orange Pi CM5 VR display:
- compatible: Must include "orangepi,rk3588-vop-vr" for Orange Pi CM5 VR display
- reg: Register address range
- interrupts: Interrupt specifier
- clocks: References to the clocks
- clock-names: Clock names
- vr,dual-display: Boolean property indicating dual display mode
- vr,low-persistence: Boolean property indicating low persistence mode
- vr,synchronized: Boolean property indicating synchronized display mode
- vr,refresh-rate: Integer property specifying refresh rate in Hz
- vr,persistence-time-us: Integer property specifying persistence time in microseconds

Example:
&vop {
    compatible = "orangepi,rk3588-vop-vr";
    reg = <0x0 0xfdd90000 0x0 0x3000>;
    interrupts = <GIC_SPI 175 IRQ_TYPE_LEVEL_HIGH>;
    clocks = <&cru ACLK_VOP>, <&cru HCLK_VOP>, <&cru DCLK_VOP>;
    clock-names = "aclk", "hclk", "dclk";
    vr,dual-display;
    vr,low-persistence;
    vr,synchronized;
    vr,refresh-rate = <90>;
    vr,persistence-time-us = <2000>;
    status = "okay";
};
```

## Build System Integration

To integrate this into the build system, add the following to the Makefile:

```makefile
# drivers/gpu/drm/rockchip/Makefile
obj-$(CONFIG_DRM_ROCKCHIP) += rockchip_drm.o
rockchip_drm-y += rockchip_drm_vop.o
obj-$(CONFIG_DRM_ROCKCHIP_ORANGEPI) += rockchip_drm_vop_orangepi.o
```

And add the following to the Kconfig:

```kconfig
config DRM_ROCKCHIP_ORANGEPI
    tristate "Rockchip VOP driver for Orange Pi CM5 VR display"
    depends on DRM_ROCKCHIP
    help
      Choose this option if you have an Orange Pi CM5 board with
      VR display requirements. This driver provides VR-specific
      enhancements such as dual display synchronization, low
      persistence mode, and optimized refresh rates.

      To compile this driver as a module, choose M here: the
      module will be called rockchip_drm_vop_orangepi.
```
