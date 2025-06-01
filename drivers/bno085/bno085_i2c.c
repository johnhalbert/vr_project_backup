// SPDX-License-Identifier: GPL-2.0
/*
 * BNO085 IMU I2C driver
 *
 * Copyright (C) 2025 VR Headset Project
 */

#include <linux/module.h>
#include <linux/i2c.h>
#include <linux/of.h>
#include <linux/of_device.h>
#include <linux/regmap.h>

#include "bno085_core.h"

/* I2C read function */
static int bno085_i2c_read(struct device *dev, u8 reg, u8 *data, int len)
{
    struct i2c_client *client = to_i2c_client(dev);
    int ret;
    
    ret = i2c_smbus_read_i2c_block_data(client, reg, len, data);
    if (ret < 0)
        return ret;
    
    return 0;
}

/* I2C write function */
static int bno085_i2c_write(struct device *dev, u8 reg, const u8 *data, int len)
{
    struct i2c_client *client = to_i2c_client(dev);
    int ret;
    
    ret = i2c_smbus_write_i2c_block_data(client, reg, len, data);
    if (ret < 0)
        return ret;
    
    return 0;
}

/* I2C FIFO read function */
static int bno085_i2c_read_fifo(struct device *dev, u8 *data, int len)
{
    struct i2c_client *client = to_i2c_client(dev);
    int ret;
    
    /* Read from FIFO register */
    ret = i2c_smbus_read_i2c_block_data(client, BNO085_REG_DATA_BUFFER, len, data);
    if (ret < 0)
        return ret;
    
    return 0;
}

/* I2C probe function */
static int bno085_i2c_probe(struct i2c_client *client,
                          const struct i2c_device_id *id)
{
    struct bno085_transport transport = {
        .read = bno085_i2c_read,
        .write = bno085_i2c_write,
        .read_fifo = bno085_i2c_read_fifo,
    };
    int irq;
    
    /* Get IRQ from device tree */
    irq = client->irq;
    
    /* Call core probe function */
    return bno085_core_probe(&client->dev, &transport, irq);
}

/* I2C remove function */
static int bno085_i2c_remove(struct i2c_client *client)
{
    return bno085_core_remove(&client->dev);
}

/* I2C device ID table */
static const struct i2c_device_id bno085_i2c_id[] = {
    { "bno085", 0 },
    { }
};
MODULE_DEVICE_TABLE(i2c, bno085_i2c_id);

/* Device tree match table */
static const struct of_device_id bno085_of_match[] = {
    { .compatible = "bosch,bno085" },
    { }
};
MODULE_DEVICE_TABLE(of, bno085_of_match);

/* I2C driver structure */
static struct i2c_driver bno085_i2c_driver = {
    .driver = {
        .name = "bno085",
        .of_match_table = bno085_of_match,
#ifdef CONFIG_PM
        .pm = &bno085_pm_ops,
#endif
    },
    .probe = bno085_i2c_probe,
    .remove = bno085_i2c_remove,
    .id_table = bno085_i2c_id,
};

module_i2c_driver(bno085_i2c_driver);

MODULE_AUTHOR("VR Headset Project");
MODULE_DESCRIPTION("BNO085 IMU I2C driver");
MODULE_LICENSE("GPL v2");
