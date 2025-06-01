/*
 * OV9281 Camera Driver for Orange Pi CM5 VR
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
#include <linux/i2c.h>
#include <linux/regulator/consumer.h>
#include <linux/videodev2.h>
#include <media/v4l2-async.h>
#include <media/v4l2-ctrls.h>
#include <media/v4l2-device.h>
#include <media/v4l2-event.h>
#include <media/v4l2-fwnode.h>
#include <media/v4l2-subdev.h>

// Include the original OV9281 driver header
#include "ov9281.h"

/* OV9281 Register Map for Orange Pi CM5 VR */
#define OV9281_MIPI_CTRL_REG           0x3034
#define OV9281_CLOCK_REG               0x3035
#define OV9281_LANE_CONFIG_REG         0x3036
#define OV9281_TIMING_REG              0x303c
#define OV9281_POWER_REG               0x3106

/* OV9281 Register Values for Orange Pi CM5 VR */
#define OV9281_MIPI_CTRL_VR            0x0a
#define OV9281_CLOCK_VR                0x21
#define OV9281_LANE_CONFIG_VR          0x60
#define OV9281_TIMING_VR               0x11
#define OV9281_POWER_VR                0x11

/* Orange Pi CM5 specific configuration */
struct ov9281_orangepi_config {
    bool vr_mode_enabled;
    u32 frame_rate;
    u32 exposure_time_us;
    bool zero_copy_enabled;
};

/* OV9281 device structure with Orange Pi CM5 extensions */
struct ov9281_orangepi_device {
    struct ov9281_device base_dev;
    struct ov9281_orangepi_config vr_config;
    bool is_orangepi_cm5;
};

/* Forward declarations */
static int ov9281_write_reg(struct i2c_client *client, u16 reg, u8 val);
static int ov9281_read_reg(struct i2c_client *client, u16 reg, u8 *val);

/* Orange Pi CM5 specific configuration */
static int ov9281_configure_orangepi_cm5(struct ov9281_device *ov9281)
{
    struct device *dev = &ov9281->client->dev;
    struct ov9281_orangepi_device *orangepi_dev = 
        container_of(ov9281, struct ov9281_orangepi_device, base_dev);
    int ret;

    dev_info(dev, "Configuring OV9281 for Orange Pi CM5\n");

    /* Orange Pi CM5 specific MIPI configuration */
    ret = ov9281_write_reg(ov9281->client, OV9281_MIPI_CTRL_REG, OV9281_MIPI_CTRL_VR);
    if (ret)
        return ret;

    /* Orange Pi CM5 specific clock configuration */
    ret = ov9281_write_reg(ov9281->client, OV9281_CLOCK_REG, OV9281_CLOCK_VR);
    if (ret)
        return ret;

    /* Orange Pi CM5 specific lane configuration */
    ret = ov9281_write_reg(ov9281->client, OV9281_LANE_CONFIG_REG, OV9281_LANE_CONFIG_VR);
    if (ret)
        return ret;

    /* Orange Pi CM5 specific timing configuration */
    ret = ov9281_write_reg(ov9281->client, OV9281_TIMING_REG, OV9281_TIMING_VR);
    if (ret)
        return ret;

    /* Orange Pi CM5 specific power optimization */
    ret = ov9281_write_reg(ov9281->client, OV9281_POWER_REG, OV9281_POWER_VR);
    if (ret)
        return ret;

    /* Store VR configuration */
    orangepi_dev->vr_config.vr_mode_enabled = true;
    orangepi_dev->vr_config.frame_rate = 90;
    orangepi_dev->vr_config.exposure_time_us = 5000;
    orangepi_dev->vr_config.zero_copy_enabled = true;
    orangepi_dev->is_orangepi_cm5 = true;

    dev_info(dev, "OV9281 configured for Orange Pi CM5 VR mode\n");
    return 0;
}

/* Update probe function to detect Orange Pi CM5 */
static int ov9281_probe_orangepi(struct i2c_client *client,
                               const struct i2c_device_id *id)
{
    struct device *dev = &client->dev;
    struct ov9281_orangepi_device *orangepi_dev;
    int ret;

    dev_info(dev, "Probing OV9281 for Orange Pi CM5\n");

    /* Allocate device structure */
    orangepi_dev = devm_kzalloc(dev, sizeof(*orangepi_dev), GFP_KERNEL);
    if (!orangepi_dev)
        return -ENOMEM;

    /* Initialize base device */
    ret = ov9281_probe(client, id, &orangepi_dev->base_dev);
    if (ret)
        return ret;

    /* Check if this is an Orange Pi CM5 device */
    if (of_device_is_compatible(dev->of_node, "orangepi,ov9281-vr")) {
        dev_info(dev, "Detected Orange Pi CM5 VR camera\n");
        
        /* Apply Orange Pi CM5 specific configuration */
        ret = ov9281_configure_orangepi_cm5(&orangepi_dev->base_dev);
        if (ret) {
            dev_err(dev, "Failed to configure for Orange Pi CM5: %d\n", ret);
            return ret;
        }
    }

    return 0;
}

/* Update the compatible strings to include Orange Pi variant */
static const struct of_device_id ov9281_of_match_orangepi[] = {
    { .compatible = "ovti,ov9281" },
    { .compatible = "orangepi,ov9281-vr" },
    { /* sentinel */ }
};
MODULE_DEVICE_TABLE(of, ov9281_of_match_orangepi);

/* Update the i2c_driver structure */
static struct i2c_driver ov9281_i2c_driver_orangepi = {
    .driver = {
        .name = "ov9281_orangepi",
        .of_match_table = ov9281_of_match_orangepi,
    },
    .probe = ov9281_probe_orangepi,
    .remove = ov9281_remove,
    .id_table = ov9281_id,
};

module_i2c_driver(ov9281_i2c_driver_orangepi);

MODULE_DESCRIPTION("OV9281 Camera Driver for Orange Pi CM5 VR");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
