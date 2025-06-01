// SPDX-License-Identifier: GPL-2.0
/*
 * BNO085 IMU SPI driver
 *
 * Copyright (C) 2025 VR Headset Project
 */

#include <linux/module.h>
#include <linux/spi/spi.h>
#include <linux/of.h>
#include <linux/of_device.h>
#include <linux/regmap.h>

#include "bno085_core.h"

/* SPI read function */
static int bno085_spi_read(struct device *dev, u8 reg, u8 *data, int len)
{
    struct spi_device *spi = to_spi_device(dev);
    struct spi_transfer xfers[2];
    struct spi_message msg;
    u8 cmd = reg | 0x80; /* Set read bit */
    int ret;
    
    memset(xfers, 0, sizeof(xfers));
    spi_message_init(&msg);
    
    /* Setup command transfer */
    xfers[0].tx_buf = &cmd;
    xfers[0].len = 1;
    spi_message_add_tail(&xfers[0], &msg);
    
    /* Setup data transfer */
    xfers[1].rx_buf = data;
    xfers[1].len = len;
    spi_message_add_tail(&xfers[1], &msg);
    
    ret = spi_sync(spi, &msg);
    if (ret < 0)
        return ret;
    
    return 0;
}

/* SPI write function */
static int bno085_spi_write(struct device *dev, u8 reg, const u8 *data, int len)
{
    struct spi_device *spi = to_spi_device(dev);
    struct spi_transfer xfers[2];
    struct spi_message msg;
    u8 cmd = reg & 0x7F; /* Clear read bit */
    int ret;
    
    memset(xfers, 0, sizeof(xfers));
    spi_message_init(&msg);
    
    /* Setup command transfer */
    xfers[0].tx_buf = &cmd;
    xfers[0].len = 1;
    spi_message_add_tail(&xfers[0], &msg);
    
    /* Setup data transfer */
    xfers[1].tx_buf = data;
    xfers[1].len = len;
    spi_message_add_tail(&xfers[1], &msg);
    
    ret = spi_sync(spi, &msg);
    if (ret < 0)
        return ret;
    
    return 0;
}

/* SPI FIFO read function */
static int bno085_spi_read_fifo(struct device *dev, u8 *data, int len)
{
    struct spi_device *spi = to_spi_device(dev);
    struct spi_transfer xfers[2];
    struct spi_message msg;
    u8 cmd = BNO085_REG_DATA_BUFFER | 0x80; /* Set read bit */
    int ret;
    
    memset(xfers, 0, sizeof(xfers));
    spi_message_init(&msg);
    
    /* Setup command transfer */
    xfers[0].tx_buf = &cmd;
    xfers[0].len = 1;
    spi_message_add_tail(&xfers[0], &msg);
    
    /* Setup data transfer */
    xfers[1].rx_buf = data;
    xfers[1].len = len;
    spi_message_add_tail(&xfers[1], &msg);
    
    ret = spi_sync(spi, &msg);
    if (ret < 0)
        return ret;
    
    return 0;
}

/* SPI probe function */
static int bno085_spi_probe(struct spi_device *spi)
{
    struct bno085_transport transport = {
        .read = bno085_spi_read,
        .write = bno085_spi_write,
        .read_fifo = bno085_spi_read_fifo,
    };
    int irq;
    
    /* Configure SPI device */
    spi->mode = SPI_MODE_0;
    spi->bits_per_word = 8;
    spi_setup(spi);
    
    /* Get IRQ from device tree */
    irq = spi->irq;
    
    /* Call core probe function */
    return bno085_core_probe(&spi->dev, &transport, irq);
}

/* SPI remove function */
static int bno085_spi_remove(struct spi_device *spi)
{
    return bno085_core_remove(&spi->dev);
}

/* Device tree match table */
static const struct of_device_id bno085_spi_of_match[] = {
    { .compatible = "bosch,bno085" },
    { }
};
MODULE_DEVICE_TABLE(of, bno085_spi_of_match);

/* SPI driver structure */
static struct spi_driver bno085_spi_driver = {
    .driver = {
        .name = "bno085",
        .of_match_table = bno085_spi_of_match,
#ifdef CONFIG_PM
        .pm = &bno085_pm_ops,
#endif
    },
    .probe = bno085_spi_probe,
    .remove = bno085_spi_remove,
};

module_spi_driver(bno085_spi_driver);

MODULE_AUTHOR("VR Headset Project");
MODULE_DESCRIPTION("BNO085 IMU SPI driver");
MODULE_LICENSE("GPL v2");
