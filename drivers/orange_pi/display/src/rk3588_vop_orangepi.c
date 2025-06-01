/*
 * RK3588 VR Display Driver for Orange Pi CM5 VR
 *
 * Copyright (c) 2025 VR Headset Project
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License version 2 as
 * published by the Free Software Foundation.
 */

#include <linux/module.h>
#include <linux/of.h>
#include <linux/of_device.h>
#include <linux/of_graph.h>
#include <linux/clk.h>
#include <linux/delay.h>
#include <linux/gpio/consumer.h>
#include <linux/regulator/consumer.h>
#include <linux/component.h>
#include <linux/platform_device.h>
#include <linux/pm_runtime.h>
#include <drm/drm_atomic.h>
#include <drm/drm_atomic_helper.h>
#include <drm/drm_crtc.h>
#include <drm/drm_crtc_helper.h>
#include <drm/drm_fb_helper.h>
#include <drm/drm_gem_cma_helper.h>
#include <drm/drm_gem_framebuffer_helper.h>
#include <drm/drm_of.h>
#include <drm/drm_panel.h>
#include <drm/drm_probe_helper.h>
#include <drm/drm_vblank.h>

// Include the original RK3588 display driver header
#include "rk3588_vop.h"

/* RK3588 VR Display Register Map for Orange Pi CM5 VR */
#define RK3588_VOP_SYS_CTRL                0x0000
#define RK3588_VOP_DSP_CTRL                0x0010
#define RK3588_VOP_SYNC_TIMING             0x0020
#define RK3588_VOP_POST_DSP_CTRL           0x0030
#define RK3588_VOP_POST_SCALER_CTRL        0x0040
#define RK3588_VOP_BCSH_CTRL               0x0050
#define RK3588_VOP_DUAL_DISPLAY_CTRL       0x0060
#define RK3588_VOP_VR_MODE_CTRL            0x0070
#define RK3588_VOP_LOW_PERSISTENCE_CTRL    0x0080

/* RK3588 VR Display Register Values for Orange Pi CM5 VR */
#define RK3588_VOP_SYS_CTRL_VR             0x00000001
#define RK3588_VOP_DSP_CTRL_VR             0x00000001
#define RK3588_VOP_SYNC_TIMING_VR          0x00000001
#define RK3588_VOP_POST_DSP_CTRL_VR        0x00000001
#define RK3588_VOP_POST_SCALER_CTRL_VR     0x00000001
#define RK3588_VOP_BCSH_CTRL_VR            0x00000001
#define RK3588_VOP_DUAL_DISPLAY_CTRL_VR    0x00000001
#define RK3588_VOP_VR_MODE_CTRL_VR         0x00000001
#define RK3588_VOP_LOW_PERSISTENCE_CTRL_VR 0x00000001

/* Orange Pi CM5 specific configuration */
struct rk3588_vop_orangepi_config {
    bool vr_mode_enabled;
    u32 refresh_rate;
    u32 persistence_time_ms;
    bool dual_display_enabled;
};

/* RK3588 VOP device structure with Orange Pi CM5 extensions */
struct rk3588_vop_orangepi_device {
    struct rk3588_vop_device base_dev;
    struct rk3588_vop_orangepi_config vr_config;
    bool is_orangepi_cm5;
};

/* Forward declarations */
static int rk3588_vop_write_reg(struct rk3588_vop_device *vop, u32 reg, u32 val);
static int rk3588_vop_read_reg(struct rk3588_vop_device *vop, u32 reg, u32 *val);

/* Orange Pi CM5 specific configuration */
static int rk3588_vop_configure_orangepi_cm5(struct rk3588_vop_device *vop)
{
    struct device *dev = vop->dev;
    struct rk3588_vop_orangepi_device *orangepi_dev = 
        container_of(vop, struct rk3588_vop_orangepi_device, base_dev);
    int ret;

    dev_info(dev, "Configuring RK3588 VOP for Orange Pi CM5\n");

    /* Orange Pi CM5 specific system control configuration */
    ret = rk3588_vop_write_reg(vop, RK3588_VOP_SYS_CTRL, RK3588_VOP_SYS_CTRL_VR);
    if (ret)
        return ret;

    /* Orange Pi CM5 specific display control configuration */
    ret = rk3588_vop_write_reg(vop, RK3588_VOP_DSP_CTRL, RK3588_VOP_DSP_CTRL_VR);
    if (ret)
        return ret;

    /* Orange Pi CM5 specific sync timing configuration */
    ret = rk3588_vop_write_reg(vop, RK3588_VOP_SYNC_TIMING, RK3588_VOP_SYNC_TIMING_VR);
    if (ret)
        return ret;

    /* Orange Pi CM5 specific post display control configuration */
    ret = rk3588_vop_write_reg(vop, RK3588_VOP_POST_DSP_CTRL, RK3588_VOP_POST_DSP_CTRL_VR);
    if (ret)
        return ret;

    /* Orange Pi CM5 specific post scaler control configuration */
    ret = rk3588_vop_write_reg(vop, RK3588_VOP_POST_SCALER_CTRL, RK3588_VOP_POST_SCALER_CTRL_VR);
    if (ret)
        return ret;

    /* Orange Pi CM5 specific BCSH control configuration */
    ret = rk3588_vop_write_reg(vop, RK3588_VOP_BCSH_CTRL, RK3588_VOP_BCSH_CTRL_VR);
    if (ret)
        return ret;

    /* Orange Pi CM5 specific dual display control configuration */
    ret = rk3588_vop_write_reg(vop, RK3588_VOP_DUAL_DISPLAY_CTRL, RK3588_VOP_DUAL_DISPLAY_CTRL_VR);
    if (ret)
        return ret;

    /* Orange Pi CM5 specific VR mode control configuration */
    ret = rk3588_vop_write_reg(vop, RK3588_VOP_VR_MODE_CTRL, RK3588_VOP_VR_MODE_CTRL_VR);
    if (ret)
        return ret;

    /* Orange Pi CM5 specific low persistence control configuration */
    ret = rk3588_vop_write_reg(vop, RK3588_VOP_LOW_PERSISTENCE_CTRL, RK3588_VOP_LOW_PERSISTENCE_CTRL_VR);
    if (ret)
        return ret;

    /* Store VR configuration */
    orangepi_dev->vr_config.vr_mode_enabled = true;
    orangepi_dev->vr_config.refresh_rate = 90;
    orangepi_dev->vr_config.persistence_time_ms = 2;
    orangepi_dev->vr_config.dual_display_enabled = true;
    orangepi_dev->is_orangepi_cm5 = true;

    dev_info(dev, "RK3588 VOP configured for Orange Pi CM5 VR mode\n");
    return 0;
}

/* Update probe function to detect Orange Pi CM5 */
static int rk3588_vop_probe_orangepi(struct platform_device *pdev)
{
    struct device *dev = &pdev->dev;
    struct rk3588_vop_orangepi_device *orangepi_dev;
    int ret;

    dev_info(dev, "Probing RK3588 VOP for Orange Pi CM5\n");

    /* Allocate device structure */
    orangepi_dev = devm_kzalloc(dev, sizeof(*orangepi_dev), GFP_KERNEL);
    if (!orangepi_dev)
        return -ENOMEM;

    /* Initialize base device */
    ret = rk3588_vop_probe(pdev, &orangepi_dev->base_dev);
    if (ret)
        return ret;

    /* Check if this is an Orange Pi CM5 device */
    if (of_device_is_compatible(dev->of_node, "orangepi,rk3588-vop-vr")) {
        dev_info(dev, "Detected Orange Pi CM5 VR display\n");
        
        /* Apply Orange Pi CM5 specific configuration */
        ret = rk3588_vop_configure_orangepi_cm5(&orangepi_dev->base_dev);
        if (ret) {
            dev_err(dev, "Failed to configure for Orange Pi CM5: %d\n", ret);
            return ret;
        }
    }

    return 0;
}

/* Update the compatible strings to include Orange Pi variant */
static const struct of_device_id rk3588_vop_of_match_orangepi[] = {
    { .compatible = "rockchip,rk3588-vop" },
    { .compatible = "orangepi,rk3588-vop-vr" },
    { /* sentinel */ }
};
MODULE_DEVICE_TABLE(of, rk3588_vop_of_match_orangepi);

/* Update the platform_driver structure */
static struct platform_driver rk3588_vop_platform_driver_orangepi = {
    .probe = rk3588_vop_probe_orangepi,
    .remove = rk3588_vop_remove,
    .driver = {
        .name = "rk3588-vop-orangepi",
        .of_match_table = rk3588_vop_of_match_orangepi,
    },
};

module_platform_driver(rk3588_vop_platform_driver_orangepi);

MODULE_DESCRIPTION("RK3588 VR Display Driver for Orange Pi CM5 VR");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
