# BNO085 IMU Driver Implementation Guide

## Overview

This document provides implementation guidance for the BNO085 IMU driver for the Linux kernel. It includes code structure, key implementation details, and best practices for developing the driver according to the design specifications.

## Driver Structure

The driver follows a modular structure with separate files for core functionality and transport layers:

```
drivers/iio/imu/bno085/
├── bno085_core.c     # Core driver functionality
├── bno085_core.h     # Internal driver header
├── bno085_i2c.c      # I2C transport layer
├── bno085_spi.c      # SPI transport layer
├── bno085_uart.c     # UART transport layer
├── bno085_dts.h      # Device tree bindings
├── Kconfig           # Kernel configuration options
└── Makefile          # Build system integration
```

## Core Header Implementation (bno085_core.h)

```c
/* SPDX-License-Identifier: GPL-2.0 */
/*
 * BNO085 IMU driver
 *
 * Copyright (C) 2025 VR Headset Project
 */

#ifndef _BNO085_CORE_H_
#define _BNO085_CORE_H_

#include <linux/types.h>
#include <linux/iio/iio.h>
#include <linux/regmap.h>

/* BNO085 Register Map */
#define BNO085_REG_CHIP_ID          0x00
#define BNO085_REG_RESET            0x01
#define BNO085_REG_STATUS           0x02
#define BNO085_REG_COMMAND          0x03
#define BNO085_REG_RESPONSE         0x04
#define BNO085_REG_DATA_BUFFER      0x05
/* ... additional registers ... */

/* BNO085 Constants */
#define BNO085_CHIP_ID              0x83
#define BNO085_RESET_COMMAND        0x01
#define BNO085_MAX_TRANSFER_SIZE    32
#define BNO085_FIFO_SIZE            1024

/* BNO085 Operation Modes */
enum bno085_operation_mode {
    BNO085_MODE_CONFIG              = 0x00,
    BNO085_MODE_IMU                 = 0x01,
    BNO085_MODE_NDOF                = 0x02,
    BNO085_MODE_NDOF_FMC_OFF        = 0x03,
    BNO085_MODE_GYRO_ONLY           = 0x04,
    BNO085_MODE_ACCEL_ONLY          = 0x05,
    BNO085_MODE_MAG_ONLY            = 0x06,
    BNO085_MODE_AR_VR_STABILIZED    = 0x07,
    BNO085_MODE_AR_VR_PREDICTIVE    = 0x08,
};

/* BNO085 Device State */
enum bno085_state {
    BNO085_STATE_DISABLED           = 0,
    BNO085_STATE_INITIALIZING       = 1,
    BNO085_STATE_INITIALIZED        = 2,
    BNO085_STATE_RUNNING            = 3,
    BNO085_STATE_ERROR              = 4,
};

/* BNO085 Sensor Features */
enum bno085_sensor_feature {
    BNO085_FEATURE_ACCELEROMETER    = BIT(0),
    BNO085_FEATURE_GYROSCOPE        = BIT(1),
    BNO085_FEATURE_MAGNETOMETER     = BIT(2),
    BNO085_FEATURE_ORIENTATION      = BIT(3),
    BNO085_FEATURE_ROTATION_VECTOR  = BIT(4),
    BNO085_FEATURE_GAME_ROTATION    = BIT(5),
    BNO085_FEATURE_LINEAR_ACCEL     = BIT(6),
    BNO085_FEATURE_GRAVITY          = BIT(7),
    BNO085_FEATURE_TEMPERATURE      = BIT(8),
};

/* BNO085 Transport Interface */
struct bno085_transport {
    int (*read)(struct device *dev, u8 reg, u8 *data, int len);
    int (*write)(struct device *dev, u8 reg, const u8 *data, int len);
    int (*read_fifo)(struct device *dev, u8 *data, int len);
};

/* BNO085 Device Structure */
struct bno085_device {
    struct device *dev;
    struct iio_dev *indio_dev;
    struct regmap *regmap;
    struct mutex lock;
    struct bno085_transport transport;
    
    /* Device state */
    enum bno085_state state;
    enum bno085_operation_mode mode;
    u32 enabled_features;
    u32 sampling_frequency;
    
    /* Calibration data */
    bool calibrated;
    u8 accel_calib[6];
    u8 gyro_calib[6];
    u8 mag_calib[6];
    
    /* Interrupt handling */
    int irq;
    bool irq_enabled;
    struct work_struct irq_work;
    
    /* IIO buffer and trigger */
    struct iio_trigger *trig;
    bool buffer_enabled;
    
    /* Data buffers */
    s16 accel_data[3];
    s16 gyro_data[3];
    s16 mag_data[3];
    s16 quaternion_data[4];
    s16 temperature_data;
    
    /* Timestamps */
    s64 timestamp;
    ktime_t last_sample_time;
    
    /* Debug */
    struct dentry *debugfs_root;
};

/* Core driver functions */
int bno085_core_probe(struct device *dev, struct bno085_transport *transport, int irq);
int bno085_core_remove(struct device *dev);
int bno085_core_init(struct bno085_device *dev);
int bno085_set_mode(struct bno085_device *dev, enum bno085_operation_mode mode);
int bno085_set_feature(struct bno085_device *dev, enum bno085_sensor_feature feature, bool enable);
int bno085_set_sampling_frequency(struct bno085_device *dev, u32 frequency);
int bno085_read_data(struct bno085_device *dev);
int bno085_reset(struct bno085_device *dev);
int bno085_suspend(struct device *dev);
int bno085_resume(struct device *dev);

/* Transport layer registration */
int bno085_i2c_probe(struct i2c_client *client, const struct i2c_device_id *id);
int bno085_i2c_remove(struct i2c_client *client);
int bno085_spi_probe(struct spi_device *spi);
int bno085_spi_remove(struct spi_device *spi);

#endif /* _BNO085_CORE_H_ */
```

## Core Driver Implementation (bno085_core.c)

```c
// SPDX-License-Identifier: GPL-2.0
/*
 * BNO085 IMU driver
 *
 * Copyright (C) 2025 VR Headset Project
 */

#include <linux/module.h>
#include <linux/init.h>
#include <linux/interrupt.h>
#include <linux/delay.h>
#include <linux/of.h>
#include <linux/of_irq.h>
#include <linux/of_device.h>
#include <linux/pm.h>
#include <linux/pm_runtime.h>
#include <linux/iio/iio.h>
#include <linux/iio/buffer.h>
#include <linux/iio/trigger.h>
#include <linux/iio/triggered_buffer.h>
#include <linux/iio/trigger_consumer.h>
#include <linux/regmap.h>
#include <linux/debugfs.h>

#include "bno085_core.h"

/* IIO channel specifications */
static const struct iio_chan_spec bno085_channels[] = {
    /* Accelerometer channels */
    {
        .type = IIO_ACCEL,
        .modified = 1,
        .channel2 = IIO_MOD_X,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | 
                             BIT(IIO_CHAN_INFO_SCALE) |
                             BIT(IIO_CHAN_INFO_SAMP_FREQ),
        .info_mask_shared_by_type = BIT(IIO_CHAN_INFO_CALIBBIAS),
        .scan_index = 0,
        .scan_type = {
            .sign = 's',
            .realbits = 16,
            .storagebits = 16,
            .endianness = IIO_LE,
        },
    },
    /* ... additional channels for Y, Z axes ... */
    
    /* Gyroscope channels */
    {
        .type = IIO_ANGL_VEL,
        .modified = 1,
        .channel2 = IIO_MOD_X,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | 
                             BIT(IIO_CHAN_INFO_SCALE) |
                             BIT(IIO_CHAN_INFO_SAMP_FREQ),
        .info_mask_shared_by_type = BIT(IIO_CHAN_INFO_CALIBBIAS),
        .scan_index = 3,
        .scan_type = {
            .sign = 's',
            .realbits = 16,
            .storagebits = 16,
            .endianness = IIO_LE,
        },
    },
    /* ... additional channels for Y, Z axes ... */
    
    /* Magnetometer channels */
    /* ... magnetometer channel definitions ... */
    
    /* Quaternion channels */
    /* ... quaternion channel definitions ... */
    
    /* Temperature channel */
    {
        .type = IIO_TEMP,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) |
                             BIT(IIO_CHAN_INFO_SCALE),
        .scan_index = 13,
        .scan_type = {
            .sign = 's',
            .realbits = 16,
            .storagebits = 16,
            .endianness = IIO_LE,
        },
    },
    
    /* Timestamp channel */
    IIO_CHAN_SOFT_TIMESTAMP(14),
};

/* IIO buffer setup */
static const struct iio_buffer_setup_ops bno085_buffer_setup_ops = {
    .postenable = bno085_buffer_postenable,
    .predisable = bno085_buffer_predisable,
};

/* IIO trigger handler */
static irqreturn_t bno085_trigger_handler(int irq, void *p)
{
    struct iio_poll_func *pf = p;
    struct iio_dev *indio_dev = pf->indio_dev;
    struct bno085_device *dev = iio_priv(indio_dev);
    int ret;
    
    /* Read sensor data */
    ret = bno085_read_data(dev);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to read sensor data: %d\n", ret);
        goto done;
    }
    
    /* Fill timestamp */
    dev->timestamp = iio_get_time_ns(indio_dev);
    
    /* Push data to IIO buffer */
    iio_push_to_buffers_with_timestamp(indio_dev, dev->buffer, dev->timestamp);
    
done:
    iio_trigger_notify_done(indio_dev->trig);
    return IRQ_HANDLED;
}

/* Hardware interrupt handler */
static irqreturn_t bno085_irq_handler(int irq, void *private)
{
    struct iio_dev *indio_dev = private;
    struct bno085_device *dev = iio_priv(indio_dev);
    
    /* Schedule bottom half processing */
    schedule_work(&dev->irq_work);
    
    return IRQ_HANDLED;
}

/* Interrupt work handler */
static void bno085_irq_work_handler(struct work_struct *work)
{
    struct bno085_device *dev = container_of(work, struct bno085_device, irq_work);
    u8 status;
    int ret;
    
    /* Read interrupt status */
    ret = dev->transport.read(dev->dev, BNO085_REG_STATUS, &status, 1);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to read interrupt status: %d\n", ret);
        return;
    }
    
    /* Handle different interrupt sources */
    if (status & BNO085_STATUS_DATA_READY) {
        /* Trigger data acquisition */
        if (dev->trig && dev->buffer_enabled)
            iio_trigger_poll(dev->trig);
    }
    
    if (status & BNO085_STATUS_CALIB_CHANGE) {
        /* Update calibration status */
        bno085_update_calibration(dev);
    }
    
    /* ... handle other interrupt sources ... */
}

/* IIO device info callbacks */
static int bno085_read_raw(struct iio_dev *indio_dev,
                         struct iio_chan_spec const *chan,
                         int *val, int *val2, long mask)
{
    struct bno085_device *dev = iio_priv(indio_dev);
    int ret;
    
    mutex_lock(&dev->lock);
    
    switch (mask) {
    case IIO_CHAN_INFO_RAW:
        /* Read raw sensor data based on channel type */
        switch (chan->type) {
        case IIO_ACCEL:
            *val = dev->accel_data[chan->channel2];
            ret = IIO_VAL_INT;
            break;
        case IIO_ANGL_VEL:
            *val = dev->gyro_data[chan->channel2];
            ret = IIO_VAL_INT;
            break;
        /* ... handle other channel types ... */
        default:
            ret = -EINVAL;
            break;
        }
        break;
        
    case IIO_CHAN_INFO_SCALE:
        /* Return scale factor based on channel type */
        switch (chan->type) {
        case IIO_ACCEL:
            /* Scale to m/s^2, 1 LSB = 0.0001 m/s^2 */
            *val = 0;
            *val2 = 100;
            ret = IIO_VAL_INT_PLUS_MICRO;
            break;
        /* ... handle other channel types ... */
        default:
            ret = -EINVAL;
            break;
        }
        break;
        
    case IIO_CHAN_INFO_SAMP_FREQ:
        *val = dev->sampling_frequency;
        ret = IIO_VAL_INT;
        break;
        
    /* ... handle other info types ... */
        
    default:
        ret = -EINVAL;
        break;
    }
    
    mutex_unlock(&dev->lock);
    return ret;
}

static int bno085_write_raw(struct iio_dev *indio_dev,
                          struct iio_chan_spec const *chan,
                          int val, int val2, long mask)
{
    struct bno085_device *dev = iio_priv(indio_dev);
    int ret;
    
    mutex_lock(&dev->lock);
    
    switch (mask) {
    case IIO_CHAN_INFO_SAMP_FREQ:
        ret = bno085_set_sampling_frequency(dev, val);
        break;
        
    /* ... handle other writable info types ... */
        
    default:
        ret = -EINVAL;
        break;
    }
    
    mutex_unlock(&dev->lock);
    return ret;
}

/* IIO device info operations */
static const struct iio_info bno085_info = {
    .read_raw = bno085_read_raw,
    .write_raw = bno085_write_raw,
    .attrs = &bno085_attribute_group,
    .debugfs_reg_access = &bno085_debugfs_reg_access,
};

/* Core initialization function */
int bno085_core_init(struct bno085_device *dev)
{
    u8 chip_id;
    int ret;
    
    /* Check device ID */
    ret = dev->transport.read(dev->dev, BNO085_REG_CHIP_ID, &chip_id, 1);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to read chip ID: %d\n", ret);
        return ret;
    }
    
    if (chip_id != BNO085_CHIP_ID) {
        dev_err(dev->dev, "Unexpected chip ID: 0x%02x (expected 0x%02x)\n",
                chip_id, BNO085_CHIP_ID);
        return -ENODEV;
    }
    
    /* Reset device */
    ret = bno085_reset(dev);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to reset device: %d\n", ret);
        return ret;
    }
    
    /* Configure default mode */
    ret = bno085_set_mode(dev, BNO085_MODE_NDOF);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to set default mode: %d\n", ret);
        return ret;
    }
    
    /* Enable default features */
    ret = bno085_set_feature(dev, BNO085_FEATURE_ACCELEROMETER, true);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to enable accelerometer: %d\n", ret);
        return ret;
    }
    
    ret = bno085_set_feature(dev, BNO085_FEATURE_GYROSCOPE, true);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to enable gyroscope: %d\n", ret);
        return ret;
    }
    
    /* ... enable other default features ... */
    
    /* Set default sampling frequency */
    ret = bno085_set_sampling_frequency(dev, 100);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to set sampling frequency: %d\n", ret);
        return ret;
    }
    
    /* Update device state */
    dev->state = BNO085_STATE_INITIALIZED;
    
    return 0;
}

/* Core probe function */
int bno085_core_probe(struct device *dev, struct bno085_transport *transport, int irq)
{
    struct bno085_device *bno085_dev;
    struct iio_dev *indio_dev;
    int ret;
    
    /* Allocate IIO device */
    indio_dev = devm_iio_device_alloc(dev, sizeof(*bno085_dev));
    if (!indio_dev) {
        dev_err(dev, "Failed to allocate IIO device\n");
        return -ENOMEM;
    }
    
    bno085_dev = iio_priv(indio_dev);
    bno085_dev->dev = dev;
    bno085_dev->indio_dev = indio_dev;
    bno085_dev->transport = *transport;
    bno085_dev->irq = irq;
    
    /* Initialize mutex */
    mutex_init(&bno085_dev->lock);
    
    /* Set up IIO device */
    indio_dev->name = "bno085";
    indio_dev->channels = bno085_channels;
    indio_dev->num_channels = ARRAY_SIZE(bno085_channels);
    indio_dev->info = &bno085_info;
    indio_dev->modes = INDIO_DIRECT_MODE | INDIO_BUFFER_TRIGGERED;
    
    /* Initialize device */
    ret = bno085_core_init(bno085_dev);
    if (ret < 0) {
        dev_err(dev, "Failed to initialize device: %d\n", ret);
        return ret;
    }
    
    /* Set up interrupt handling */
    if (bno085_dev->irq > 0) {
        INIT_WORK(&bno085_dev->irq_work, bno085_irq_work_handler);
        
        ret = devm_request_irq(dev, bno085_dev->irq, bno085_irq_handler,
                              IRQF_TRIGGER_RISING, "bno085", indio_dev);
        if (ret < 0) {
            dev_err(dev, "Failed to request IRQ: %d\n", ret);
            return ret;
        }
        
        bno085_dev->irq_enabled = true;
    }
    
    /* Set up IIO triggered buffer */
    ret = devm_iio_triggered_buffer_setup(dev, indio_dev, NULL,
                                         bno085_trigger_handler,
                                         &bno085_buffer_setup_ops);
    if (ret < 0) {
        dev_err(dev, "Failed to setup triggered buffer: %d\n", ret);
        return ret;
    }
    
    /* Set up IIO trigger */
    ret = devm_iio_trigger_register(dev, bno085_dev->trig);
    if (ret < 0) {
        dev_err(dev, "Failed to register trigger: %d\n", ret);
        return ret;
    }
    
    /* Register IIO device */
    ret = devm_iio_device_register(dev, indio_dev);
    if (ret < 0) {
        dev_err(dev, "Failed to register IIO device: %d\n", ret);
        return ret;
    }
    
    /* Set up debugfs */
    bno085_dev->debugfs_root = debugfs_create_dir("bno085", indio_dev->debugfs_dentry);
    if (bno085_dev->debugfs_root) {
        debugfs_create_file("registers", 0644, bno085_dev->debugfs_root,
                           bno085_dev, &bno085_debugfs_reg_fops);
    }
    
    /* Enable runtime PM */
    pm_runtime_enable(dev);
    pm_runtime_set_active(dev);
    
    dev_info(dev, "BNO085 IMU initialized\n");
    
    return 0;
}

/* Core remove function */
int bno085_core_remove(struct device *dev)
{
    pm_runtime_disable(dev);
    return 0;
}

/* Module information */
MODULE_AUTHOR("VR Headset Project");
MODULE_DESCRIPTION("BNO085 IMU driver");
MODULE_LICENSE("GPL v2");
```

## I2C Transport Implementation (bno085_i2c.c)

```c
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

/* I2C probe function */
static int bno085_i2c_probe(struct i2c_client *client,
                          const struct i2c_device_id *id)
{
    struct bno085_transport transport = {
        .read = bno085_i2c_read,
        .write = bno085_i2c_write,
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
        .pm = &bno085_pm_ops,
    },
    .probe = bno085_i2c_probe,
    .remove = bno085_i2c_remove,
    .id_table = bno085_i2c_id,
};

module_i2c_driver(bno085_i2c_driver);

MODULE_AUTHOR("VR Headset Project");
MODULE_DESCRIPTION("BNO085 IMU I2C driver");
MODULE_LICENSE("GPL v2");
```

## SPI Transport Implementation (bno085_spi.c)

```c
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

/* SPI probe function */
static int bno085_spi_probe(struct spi_device *spi)
{
    struct bno085_transport transport = {
        .read = bno085_spi_read,
        .write = bno085_spi_write,
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
        .pm = &bno085_pm_ops,
    },
    .probe = bno085_spi_probe,
    .remove = bno085_spi_remove,
};

module_spi_driver(bno085_spi_driver);

MODULE_AUTHOR("VR Headset Project");
MODULE_DESCRIPTION("BNO085 IMU SPI driver");
MODULE_LICENSE("GPL v2");
```

## Makefile

```makefile
# SPDX-License-Identifier: GPL-2.0
#
# Makefile for BNO085 IMU driver
#

obj-$(CONFIG_IIO_BNO085) += bno085.o
bno085-y := bno085_core.o
bno085-$(CONFIG_IIO_BNO085_I2C) += bno085_i2c.o
bno085-$(CONFIG_IIO_BNO085_SPI) += bno085_spi.o
bno085-$(CONFIG_IIO_BNO085_UART) += bno085_uart.o
```

## Kconfig

```
# SPDX-License-Identifier: GPL-2.0
#
# BNO085 IMU driver configuration
#

config IIO_BNO085
	tristate "Bosch BNO085 9-axis IMU"
	depends on I2C || SPI
	select IIO_BUFFER
	select IIO_TRIGGERED_BUFFER
	help
	  Say Y here to build support for the Bosch BNO085 9-axis IMU with
	  built-in sensor fusion. This driver supports I2C and SPI interfaces.
	  
	  To compile this driver as a module, choose M here: the module
	  will be called bno085.

config IIO_BNO085_I2C
	bool "BNO085 I2C interface support"
	depends on IIO_BNO085 && I2C
	default y
	help
	  Say Y here to enable support for I2C interface to BNO085 IMU.

config IIO_BNO085_SPI
	bool "BNO085 SPI interface support"
	depends on IIO_BNO085 && SPI
	default y
	help
	  Say Y here to enable support for SPI interface to BNO085 IMU.

config IIO_BNO085_UART
	bool "BNO085 UART interface support"
	depends on IIO_BNO085 && TTY
	default n
	help
	  Say Y here to enable support for UART interface to BNO085 IMU.

config IIO_BNO085_VR_OPTIMIZATIONS
	bool "BNO085 VR-specific optimizations"
	depends on IIO_BNO085
	default y
	help
	  Say Y here to enable VR-specific optimizations for the BNO085 driver,
	  including high-rate sampling, low-latency interrupt handling, and
	  special AR/VR operation modes.
```

## Implementation Best Practices

1. **Error Handling**:
   - Always check return values from functions
   - Provide meaningful error messages
   - Clean up resources on error paths

2. **Locking**:
   - Use appropriate locking mechanisms (mutex, spinlock)
   - Avoid nested locks to prevent deadlocks
   - Keep critical sections as short as possible

3. **Memory Management**:
   - Use devm_* functions for managed resources
   - Avoid dynamic memory allocation in fast paths
   - Use appropriate memory barriers for synchronization

4. **Performance Optimization**:
   - Minimize I/O operations
   - Use DMA for large transfers
   - Optimize interrupt handling
   - Use appropriate data structures for fast access

5. **Power Management**:
   - Implement suspend/resume functions
   - Support runtime PM
   - Implement power-efficient operation modes

6. **Debugging**:
   - Use dev_dbg() for debug messages
   - Implement debugfs interface for runtime debugging
   - Add appropriate log levels for different message types

7. **Documentation**:
   - Document all public functions
   - Add kernel-doc comments
   - Document device tree bindings

## Testing Recommendations

1. **Unit Testing**:
   - Test each function in isolation
   - Use mock objects for hardware access
   - Test error paths and edge cases

2. **Integration Testing**:
   - Test with real hardware
   - Test with different configurations
   - Test with different kernel versions

3. **Performance Testing**:
   - Measure interrupt latency
   - Measure data throughput
   - Measure CPU utilization

4. **Stress Testing**:
   - Test with high sampling rates
   - Test with multiple concurrent applications
   - Test with system under load

## Conclusion

This implementation guide provides a comprehensive approach to developing the BNO085 IMU driver for the Linux kernel. By following these guidelines and best practices, the driver will meet the requirements for VR applications, including high-rate sampling, low-latency interrupt handling, and special AR/VR operation modes.
