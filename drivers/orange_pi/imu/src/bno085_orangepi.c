/*
 * BNO085 IMU Driver for Orange Pi CM5 VR
 *
 * Copyright (c) 2025 VR Headset Project
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License version 2 as
 * published by the Free Software Foundation.
 */

#include <linux/module.h>
#include <linux/i2c.h>
#include <linux/spi/spi.h>
#include <linux/iio/iio.h>
#include <linux/iio/sysfs.h>
#include <linux/iio/trigger.h>
#include <linux/iio/buffer.h>
#include <linux/iio/triggered_buffer.h>
#include <linux/iio/trigger_consumer.h>
#include <linux/of.h>
#include <linux/of_gpio.h>
#include <linux/of_irq.h>
#include <linux/interrupt.h>
#include <linux/gpio/consumer.h>
#include <linux/regulator/consumer.h>

// Include the original BNO085 driver header
#include "bno085.h"

/* BNO085 Register Map for Orange Pi CM5 VR */
#define BNO085_REG_HOST_INTERFACE_CTRL    0x01
#define BNO085_REG_OPERATING_MODE         0x02
#define BNO085_REG_INT_MASK               0x03
#define BNO085_REG_ACCEL_CONFIG           0x04
#define BNO085_REG_GYRO_CONFIG            0x05

/* BNO085 Operating Modes */
#define BNO085_MODE_NORMAL                0x00
#define BNO085_MODE_VR                    0x01
#define BNO085_MODE_LOW_POWER             0x02

/* BNO085 Interrupt Flags */
#define BNO085_INT_GYRO_READY             0x01
#define BNO085_INT_ACCEL_READY            0x02
#define BNO085_INT_MAG_READY              0x04
#define BNO085_INT_FUSION_READY           0x08

/* BNO085 Accelerometer Sample Rates */
#define BNO085_ACCEL_RATE_100HZ           0x00
#define BNO085_ACCEL_RATE_200HZ           0x01
#define BNO085_ACCEL_RATE_400HZ           0x02
#define BNO085_ACCEL_RATE_1000HZ          0x03

/* BNO085 Gyroscope Sample Rates */
#define BNO085_GYRO_RATE_100HZ            0x00
#define BNO085_GYRO_RATE_200HZ            0x01
#define BNO085_GYRO_RATE_400HZ            0x02
#define BNO085_GYRO_RATE_1000HZ           0x03

/* Orange Pi CM5 specific configuration */
struct bno085_orangepi_config {
    bool vr_mode_enabled;
    u32 sample_rate_hz;
    u32 fusion_rate_hz;
    bool low_latency_mode;
};

/* BNO085 device structure with Orange Pi CM5 extensions */
struct bno085_orangepi_device {
    struct bno085_device base_dev;
    struct bno085_orangepi_config vr_config;
    bool is_orangepi_cm5;
};

/* Forward declarations */
static int bno085_write_reg(struct bno085_device *dev, u8 reg, u8 value);
static int bno085_read_reg(struct bno085_device *dev, u8 reg, u8 *value);

/* Orange Pi CM5 specific configuration */
static int bno085_configure_orangepi_cm5(struct bno085_device *bno085)
{
    struct device *dev = bno085->dev;
    struct bno085_orangepi_device *orangepi_dev = 
        container_of(bno085, struct bno085_orangepi_device, base_dev);
    int ret;

    dev_info(dev, "Configuring BNO085 for Orange Pi CM5\n");

    /* Orange Pi CM5 specific GPIO configuration */
    if (bno085->gpio_reset) {
        gpiod_set_value_cansleep(bno085->gpio_reset, 0);
        msleep(10);
        gpiod_set_value_cansleep(bno085->gpio_reset, 1);
        msleep(50);
    }

    /* Orange Pi CM5 specific I2C configuration */
    if (bno085->client) {
        /* Set I2C specific settings for Orange Pi CM5 */
        ret = bno085_write_reg(bno085, BNO085_REG_HOST_INTERFACE_CTRL, 0x01);
        if (ret)
            return ret;
    }

    /* Orange Pi CM5 specific SPI configuration */
    if (bno085->spi) {
        /* Set SPI specific settings for Orange Pi CM5 */
        ret = bno085_write_reg(bno085, BNO085_REG_HOST_INTERFACE_CTRL, 0x02);
        if (ret)
            return ret;
    }

    /* Orange Pi CM5 specific VR mode configuration */
    ret = bno085_write_reg(bno085, BNO085_REG_OPERATING_MODE, BNO085_MODE_VR);
    if (ret)
        return ret;

    /* Orange Pi CM5 specific interrupt configuration */
    ret = bno085_write_reg(bno085, BNO085_REG_INT_MASK, 
                          BNO085_INT_GYRO_READY | BNO085_INT_ACCEL_READY);
    if (ret)
        return ret;

    /* Orange Pi CM5 specific sample rate configuration (1000Hz) */
    ret = bno085_write_reg(bno085, BNO085_REG_ACCEL_CONFIG, BNO085_ACCEL_RATE_1000HZ);
    if (ret)
        return ret;
    
    ret = bno085_write_reg(bno085, BNO085_REG_GYRO_CONFIG, BNO085_GYRO_RATE_1000HZ);
    if (ret)
        return ret;

    /* Store VR configuration */
    orangepi_dev->vr_config.vr_mode_enabled = true;
    orangepi_dev->vr_config.sample_rate_hz = 1000;
    orangepi_dev->vr_config.fusion_rate_hz = 1000;
    orangepi_dev->vr_config.low_latency_mode = true;
    orangepi_dev->is_orangepi_cm5 = true;

    dev_info(dev, "BNO085 configured for Orange Pi CM5 VR mode\n");
    return 0;
}

/* Update probe function to detect Orange Pi CM5 */
static int bno085_probe_orangepi(struct i2c_client *client,
                               const struct i2c_device_id *id)
{
    struct device *dev = &client->dev;
    struct bno085_orangepi_device *orangepi_dev;
    int ret;

    dev_info(dev, "Probing BNO085 for Orange Pi CM5\n");

    /* Allocate device structure */
    orangepi_dev = devm_kzalloc(dev, sizeof(*orangepi_dev), GFP_KERNEL);
    if (!orangepi_dev)
        return -ENOMEM;

    /* Initialize base device */
    ret = bno085_probe(client, id, &orangepi_dev->base_dev);
    if (ret)
        return ret;

    /* Check if this is an Orange Pi CM5 device */
    if (of_device_is_compatible(dev->of_node, "orangepi,bno085-vr")) {
        dev_info(dev, "Detected Orange Pi CM5 VR IMU\n");
        
        /* Apply Orange Pi CM5 specific configuration */
        ret = bno085_configure_orangepi_cm5(&orangepi_dev->base_dev);
        if (ret) {
            dev_err(dev, "Failed to configure for Orange Pi CM5: %d\n", ret);
            return ret;
        }
    }

    return 0;
}

/* Update the compatible strings to include Orange Pi variant */
static const struct of_device_id bno085_of_match_orangepi[] = {
    { .compatible = "bosch,bno085" },
    { .compatible = "orangepi,bno085-vr" },
    { /* sentinel */ }
};
MODULE_DEVICE_TABLE(of, bno085_of_match_orangepi);

/* Update the i2c_driver structure */
static struct i2c_driver bno085_i2c_driver_orangepi = {
    .driver = {
        .name = "bno085_orangepi",
        .of_match_table = bno085_of_match_orangepi,
    },
    .probe = bno085_probe_orangepi,
    .remove = bno085_remove,
    .id_table = bno085_id,
};

module_i2c_driver(bno085_i2c_driver_orangepi);

MODULE_DESCRIPTION("BNO085 IMU Driver for Orange Pi CM5 VR");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
