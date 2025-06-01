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

/* Regmap configuration */
const struct regmap_config bno085_regmap_config = {
    .reg_bits = 8,
    .val_bits = 8,
    .max_register = 0xFF,
    .cache_type = REGCACHE_RBTREE,
};
EXPORT_SYMBOL_GPL(bno085_regmap_config);

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
    {
        .type = IIO_ACCEL,
        .modified = 1,
        .channel2 = IIO_MOD_Y,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | 
                             BIT(IIO_CHAN_INFO_SCALE) |
                             BIT(IIO_CHAN_INFO_SAMP_FREQ),
        .info_mask_shared_by_type = BIT(IIO_CHAN_INFO_CALIBBIAS),
        .scan_index = 1,
        .scan_type = {
            .sign = 's',
            .realbits = 16,
            .storagebits = 16,
            .endianness = IIO_LE,
        },
    },
    {
        .type = IIO_ACCEL,
        .modified = 1,
        .channel2 = IIO_MOD_Z,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | 
                             BIT(IIO_CHAN_INFO_SCALE) |
                             BIT(IIO_CHAN_INFO_SAMP_FREQ),
        .info_mask_shared_by_type = BIT(IIO_CHAN_INFO_CALIBBIAS),
        .scan_index = 2,
        .scan_type = {
            .sign = 's',
            .realbits = 16,
            .storagebits = 16,
            .endianness = IIO_LE,
        },
    },
    
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
    {
        .type = IIO_ANGL_VEL,
        .modified = 1,
        .channel2 = IIO_MOD_Y,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | 
                             BIT(IIO_CHAN_INFO_SCALE) |
                             BIT(IIO_CHAN_INFO_SAMP_FREQ),
        .info_mask_shared_by_type = BIT(IIO_CHAN_INFO_CALIBBIAS),
        .scan_index = 4,
        .scan_type = {
            .sign = 's',
            .realbits = 16,
            .storagebits = 16,
            .endianness = IIO_LE,
        },
    },
    {
        .type = IIO_ANGL_VEL,
        .modified = 1,
        .channel2 = IIO_MOD_Z,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | 
                             BIT(IIO_CHAN_INFO_SCALE) |
                             BIT(IIO_CHAN_INFO_SAMP_FREQ),
        .info_mask_shared_by_type = BIT(IIO_CHAN_INFO_CALIBBIAS),
        .scan_index = 5,
        .scan_type = {
            .sign = 's',
            .realbits = 16,
            .storagebits = 16,
            .endianness = IIO_LE,
        },
    },
    
    /* Magnetometer channels */
    {
        .type = IIO_MAGN,
        .modified = 1,
        .channel2 = IIO_MOD_X,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | 
                             BIT(IIO_CHAN_INFO_SCALE) |
                             BIT(IIO_CHAN_INFO_SAMP_FREQ),
        .info_mask_shared_by_type = BIT(IIO_CHAN_INFO_CALIBBIAS),
        .scan_index = 6,
        .scan_type = {
            .sign = 's',
            .realbits = 16,
            .storagebits = 16,
            .endianness = IIO_LE,
        },
    },
    {
        .type = IIO_MAGN,
        .modified = 1,
        .channel2 = IIO_MOD_Y,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | 
                             BIT(IIO_CHAN_INFO_SCALE) |
                             BIT(IIO_CHAN_INFO_SAMP_FREQ),
        .info_mask_shared_by_type = BIT(IIO_CHAN_INFO_CALIBBIAS),
        .scan_index = 7,
        .scan_type = {
            .sign = 's',
            .realbits = 16,
            .storagebits = 16,
            .endianness = IIO_LE,
        },
    },
    {
        .type = IIO_MAGN,
        .modified = 1,
        .channel2 = IIO_MOD_Z,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | 
                             BIT(IIO_CHAN_INFO_SCALE) |
                             BIT(IIO_CHAN_INFO_SAMP_FREQ),
        .info_mask_shared_by_type = BIT(IIO_CHAN_INFO_CALIBBIAS),
        .scan_index = 8,
        .scan_type = {
            .sign = 's',
            .realbits = 16,
            .storagebits = 16,
            .endianness = IIO_LE,
        },
    },
    
    /* Quaternion channels */
    {
        .type = IIO_ROT,
        .modified = 1,
        .channel2 = IIO_MOD_QUATERNION_W,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | 
                             BIT(IIO_CHAN_INFO_SCALE),
        .scan_index = 9,
        .scan_type = {
            .sign = 's',
            .realbits = 16,
            .storagebits = 16,
            .endianness = IIO_LE,
        },
    },
    {
        .type = IIO_ROT,
        .modified = 1,
        .channel2 = IIO_MOD_QUATERNION_X,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | 
                             BIT(IIO_CHAN_INFO_SCALE),
        .scan_index = 10,
        .scan_type = {
            .sign = 's',
            .realbits = 16,
            .storagebits = 16,
            .endianness = IIO_LE,
        },
    },
    {
        .type = IIO_ROT,
        .modified = 1,
        .channel2 = IIO_MOD_QUATERNION_Y,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | 
                             BIT(IIO_CHAN_INFO_SCALE),
        .scan_index = 11,
        .scan_type = {
            .sign = 's',
            .realbits = 16,
            .storagebits = 16,
            .endianness = IIO_LE,
        },
    },
    {
        .type = IIO_ROT,
        .modified = 1,
        .channel2 = IIO_MOD_QUATERNION_Z,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | 
                             BIT(IIO_CHAN_INFO_SCALE),
        .scan_index = 12,
        .scan_type = {
            .sign = 's',
            .realbits = 16,
            .storagebits = 16,
            .endianness = IIO_LE,
        },
    },
    
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
const struct iio_buffer_setup_ops bno085_buffer_setup_ops = {
    .postenable = bno085_buffer_postenable,
    .predisable = bno085_buffer_predisable,
};

/* IIO trigger handler */
irqreturn_t bno085_trigger_handler(int irq, void *p)
{
    struct iio_poll_func *pf = p;
    struct iio_dev *indio_dev = pf->indio_dev;
    struct bno085_device *dev = iio_priv(indio_dev);
    int ret;
    u8 buffer[ALIGN(14 * sizeof(s16) + sizeof(s64), sizeof(s64))];
    
    /* Read sensor data */
    ret = bno085_read_data(dev);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to read sensor data: %d\n", ret);
        goto done;
    }
    
    /* Fill buffer with data based on enabled channels */
    memset(buffer, 0, sizeof(buffer));
    
    if (test_bit(0, indio_dev->active_scan_mask))
        memcpy(&buffer[0], &dev->accel_data[0], sizeof(s16));
    if (test_bit(1, indio_dev->active_scan_mask))
        memcpy(&buffer[2], &dev->accel_data[1], sizeof(s16));
    if (test_bit(2, indio_dev->active_scan_mask))
        memcpy(&buffer[4], &dev->accel_data[2], sizeof(s16));
    
    if (test_bit(3, indio_dev->active_scan_mask))
        memcpy(&buffer[6], &dev->gyro_data[0], sizeof(s16));
    if (test_bit(4, indio_dev->active_scan_mask))
        memcpy(&buffer[8], &dev->gyro_data[1], sizeof(s16));
    if (test_bit(5, indio_dev->active_scan_mask))
        memcpy(&buffer[10], &dev->gyro_data[2], sizeof(s16));
    
    if (test_bit(6, indio_dev->active_scan_mask))
        memcpy(&buffer[12], &dev->mag_data[0], sizeof(s16));
    if (test_bit(7, indio_dev->active_scan_mask))
        memcpy(&buffer[14], &dev->mag_data[1], sizeof(s16));
    if (test_bit(8, indio_dev->active_scan_mask))
        memcpy(&buffer[16], &dev->mag_data[2], sizeof(s16));
    
    if (test_bit(9, indio_dev->active_scan_mask))
        memcpy(&buffer[18], &dev->quaternion_data[0], sizeof(s16));
    if (test_bit(10, indio_dev->active_scan_mask))
        memcpy(&buffer[20], &dev->quaternion_data[1], sizeof(s16));
    if (test_bit(11, indio_dev->active_scan_mask))
        memcpy(&buffer[22], &dev->quaternion_data[2], sizeof(s16));
    if (test_bit(12, indio_dev->active_scan_mask))
        memcpy(&buffer[24], &dev->quaternion_data[3], sizeof(s16));
    
    if (test_bit(13, indio_dev->active_scan_mask))
        memcpy(&buffer[26], &dev->temperature_data, sizeof(s16));
    
    /* Fill timestamp */
    dev->timestamp = iio_get_time_ns(indio_dev);
    
    /* Push data to IIO buffer */
    iio_push_to_buffers_with_timestamp(indio_dev, buffer, dev->timestamp);
    
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
    ret = dev->transport.read(dev->dev, BNO085_REG_INT_STATUS, &status, 1);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to read interrupt status: %d\n", ret);
        return;
    }
    
    /* Handle different interrupt sources */
    if (status & BNO085_INT_ACCEL || status & BNO085_INT_GYRO || 
        status & BNO085_INT_MAG || status & BNO085_INT_QUAT) {
        /* Trigger data acquisition */
        if (dev->trig && dev->buffer_enabled)
            iio_trigger_poll(dev->trig);
    }
    
    if (status & BNO085_INT_CALIB) {
        /* Update calibration status */
        bno085_update_calibration(dev);
    }
    
    if (status & BNO085_INT_ERROR) {
        /* Handle error condition */
        dev_err(dev->dev, "Device reported error condition\n");
        dev->state = BNO085_STATE_ERROR;
    }
    
    if (status & BNO085_INT_FIFO) {
        /* Handle FIFO overflow */
        dev_warn(dev->dev, "FIFO overflow detected\n");
    }
    
    /* Clear interrupt status */
    ret = dev->transport.write(dev->dev, BNO085_REG_INT_STATUS, &status, 1);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to clear interrupt status: %d\n", ret);
    }
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
        case IIO_MAGN:
            *val = dev->mag_data[chan->channel2];
            ret = IIO_VAL_INT;
            break;
        case IIO_ROT:
            *val = dev->quaternion_data[chan->channel2];
            ret = IIO_VAL_INT;
            break;
        case IIO_TEMP:
            *val = dev->temperature_data;
            ret = IIO_VAL_INT;
            break;
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
        case IIO_ANGL_VEL:
            /* Scale to rad/s, 1 LSB = 0.0001 rad/s */
            *val = 0;
            *val2 = 100;
            ret = IIO_VAL_INT_PLUS_MICRO;
            break;
        case IIO_MAGN:
            /* Scale to gauss, 1 LSB = 0.0001 gauss */
            *val = 0;
            *val2 = 100;
            ret = IIO_VAL_INT_PLUS_MICRO;
            break;
        case IIO_ROT:
            /* Quaternion units, 1 LSB = 1/2^14 */
            *val = 1;
            *val2 = 16384; /* 2^14 */
            ret = IIO_VAL_FRACTIONAL;
            break;
        case IIO_TEMP:
            /* Scale to degrees Celsius, 1 LSB = 0.01 degrees */
            *val = 0;
            *val2 = 10000;
            ret = IIO_VAL_INT_PLUS_MICRO;
            break;
        default:
            ret = -EINVAL;
            break;
        }
        break;
        
    case IIO_CHAN_INFO_SAMP_FREQ:
        *val = dev->sampling_frequency;
        ret = IIO_VAL_INT;
        break;
        
    case IIO_CHAN_INFO_CALIBBIAS:
        /* Return calibration bias based on channel type */
        switch (chan->type) {
        case IIO_ACCEL:
            *val = ((s16 *)dev->accel_calib)[chan->channel2];
            ret = IIO_VAL_INT;
            break;
        case IIO_ANGL_VEL:
            *val = ((s16 *)dev->gyro_calib)[chan->channel2];
            ret = IIO_VAL_INT;
            break;
        case IIO_MAGN:
            *val = ((s16 *)dev->mag_calib)[chan->channel2];
            ret = IIO_VAL_INT;
            break;
        default:
            ret = -EINVAL;
            break;
        }
        break;
        
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
        
    case IIO_CHAN_INFO_CALIBBIAS:
        /* Set calibration bias based on channel type */
        switch (chan->type) {
        case IIO_ACCEL:
            ((s16 *)dev->accel_calib)[chan->channel2] = val;
            ret = 0;
            break;
        case IIO_ANGL_VEL:
            ((s16 *)dev->gyro_calib)[chan->channel2] = val;
            ret = 0;
            break;
        case IIO_MAGN:
            ((s16 *)dev->mag_calib)[chan->channel2] = val;
            ret = 0;
            break;
        default:
            ret = -EINVAL;
            break;
        }
        break;
        
    default:
        ret = -EINVAL;
        break;
    }
    
    mutex_unlock(&dev->lock);
    return ret;
}

/* Sysfs attributes */
static ssize_t bno085_show_mode(struct device *dev,
                              struct device_attribute *attr,
                              char *buf)
{
    struct iio_dev *indio_dev = dev_to_iio_dev(dev);
    struct bno085_device *bno085_dev = iio_priv(indio_dev);
    
    return sprintf(buf, "%d\n", bno085_dev->mode);
}

static ssize_t bno085_store_mode(struct device *dev,
                               struct device_attribute *attr,
                               const char *buf, size_t len)
{
    struct iio_dev *indio_dev = dev_to_iio_dev(dev);
    struct bno085_device *bno085_dev = iio_priv(indio_dev);
    unsigned long mode;
    int ret;
    
    ret = kstrtoul(buf, 10, &mode);
    if (ret)
        return ret;
    
    if (mode > BNO085_MODE_AR_VR_PREDICTIVE)
        return -EINVAL;
    
    ret = bno085_set_mode(bno085_dev, mode);
    if (ret)
        return ret;
    
    return len;
}

static ssize_t bno085_show_calibration_status(struct device *dev,
                                           struct device_attribute *attr,
                                           char *buf)
{
    struct iio_dev *indio_dev = dev_to_iio_dev(dev);
    struct bno085_device *bno085_dev = iio_priv(indio_dev);
    u8 status;
    int ret;
    
    ret = bno085_dev->transport.read(bno085_dev->dev, BNO085_REG_CALIB_STATUS, &status, 1);
    if (ret < 0)
        return ret;
    
    return sprintf(buf, "0x%02x\n", status);
}

static ssize_t bno085_store_reset(struct device *dev,
                                struct device_attribute *attr,
                                const char *buf, size_t len)
{
    struct iio_dev *indio_dev = dev_to_iio_dev(dev);
    struct bno085_device *bno085_dev = iio_priv(indio_dev);
    unsigned long val;
    int ret;
    
    ret = kstrtoul(buf, 10, &val);
    if (ret)
        return ret;
    
    if (val != 1)
        return -EINVAL;
    
    ret = bno085_reset(bno085_dev);
    if (ret)
        return ret;
    
    return len;
}

static IIO_DEVICE_ATTR(mode, S_IRUGO | S_IWUSR,
                     bno085_show_mode, bno085_store_mode, 0);
static IIO_DEVICE_ATTR(calibration_status, S_IRUGO,
                     bno085_show_calibration_status, NULL, 0);
static IIO_DEVICE_ATTR(reset, S_IWUSR,
                     NULL, bno085_store_reset, 0);

static struct attribute *bno085_attributes[] = {
    &iio_dev_attr_mode.dev_attr.attr,
    &iio_dev_attr_calibration_status.dev_attr.attr,
    &iio_dev_attr_reset.dev_attr.attr,
    NULL
};

const struct attribute_group bno085_attribute_group = {
    .attrs = bno085_attributes,
};
EXPORT_SYMBOL_GPL(bno085_attribute_group);

/* IIO device info operations */
const struct iio_info bno085_info = {
    .read_raw = bno085_read_raw,
    .write_raw = bno085_write_raw,
    .attrs = &bno085_attribute_group,
    .debugfs_reg_access = bno085_debugfs_reg_access,
};
EXPORT_SYMBOL_GPL(bno085_info);

/* Buffer operations */
int bno085_buffer_postenable(struct iio_dev *indio_dev)
{
    struct bno085_device *dev = iio_priv(indio_dev);
    int ret;
    
    /* Enable data ready interrupt */
    u8 int_enable = BNO085_INT_ACCEL | BNO085_INT_GYRO | BNO085_INT_MAG | 
                   BNO085_INT_QUAT | BNO085_INT_ERROR | BNO085_INT_CALIB;
    
    ret = dev->transport.write(dev->dev, BNO085_REG_INT_ENABLE, &int_enable, 1);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to enable interrupts: %d\n", ret);
        return ret;
    }
    
    dev->buffer_enabled = true;
    
    return 0;
}

int bno085_buffer_predisable(struct iio_dev *indio_dev)
{
    struct bno085_device *dev = iio_priv(indio_dev);
    int ret;
    
    /* Disable data ready interrupt */
    u8 int_enable = 0;
    
    ret = dev->transport.write(dev->dev, BNO085_REG_INT_ENABLE, &int_enable, 1);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to disable interrupts: %d\n", ret);
        return ret;
    }
    
    dev->buffer_enabled = false;
    
    return 0;
}

/* Debugfs operations */
int bno085_debugfs_reg_access(struct iio_dev *indio_dev,
                            unsigned reg, unsigned writeval,
                            unsigned *readval)
{
    struct bno085_device *dev = iio_priv(indio_dev);
    u8 val;
    int ret;
    
    if (reg > 0xFF)
        return -EINVAL;
    
    if (readval) {
        ret = dev->transport.read(dev->dev, reg, &val, 1);
        if (ret < 0)
            return ret;
        *readval = val;
    } else {
        val = writeval;
        ret = dev->transport.write(dev->dev, reg, &val, 1);
        if (ret < 0)
            return ret;
    }
    
    return 0;
}

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
    
    ret = bno085_set_feature(dev, BNO085_FEATURE_MAGNETOMETER, true);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to enable magnetometer: %d\n", ret);
        return ret;
    }
    
    ret = bno085_set_feature(dev, BNO085_FEATURE_ROTATION_VECTOR, true);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to enable rotation vector: %d\n", ret);
        return ret;
    }
    
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
    bno085_dev->trig = devm_iio_trigger_alloc(dev, "%s-trigger", indio_dev->name);
    if (!bno085_dev->trig) {
        dev_err(dev, "Failed to allocate trigger\n");
        return -ENOMEM;
    }
    
    bno085_dev->trig->dev.parent = dev;
    iio_trigger_set_drvdata(bno085_dev->trig, indio_dev);
    
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

/* Device operations */
int bno085_set_mode(struct bno085_device *dev, enum bno085_operation_mode mode)
{
    u8 mode_val = mode;
    int ret;
    
    if (dev->mode == mode)
        return 0;
    
    ret = dev->transport.write(dev->dev, BNO085_REG_COMMAND, &mode_val, 1);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to set operation mode: %d\n", ret);
        return ret;
    }
    
    /* Wait for mode change to take effect */
    msleep(50);
    
    dev->mode = mode;
    
    return 0;
}

int bno085_set_feature(struct bno085_device *dev, enum bno085_sensor_feature feature, bool enable)
{
    u8 feat_ctrl;
    int ret;
    
    ret = dev->transport.read(dev->dev, BNO085_REG_FEAT_CTRL, &feat_ctrl, 1);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to read feature control: %d\n", ret);
        return ret;
    }
    
    if (enable)
        feat_ctrl |= feature;
    else
        feat_ctrl &= ~feature;
    
    ret = dev->transport.write(dev->dev, BNO085_REG_FEAT_CTRL, &feat_ctrl, 1);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to write feature control: %d\n", ret);
        return ret;
    }
    
    if (enable)
        dev->enabled_features |= feature;
    else
        dev->enabled_features &= ~feature;
    
    return 0;
}

int bno085_set_sampling_frequency(struct bno085_device *dev, u32 frequency)
{
    u8 freq_bytes[4];
    int ret;
    
    if (frequency < 1 || frequency > 1000)
        return -EINVAL;
    
    /* Convert frequency to little-endian bytes */
    freq_bytes[0] = frequency & 0xFF;
    freq_bytes[1] = (frequency >> 8) & 0xFF;
    freq_bytes[2] = (frequency >> 16) & 0xFF;
    freq_bytes[3] = (frequency >> 24) & 0xFF;
    
    ret = dev->transport.write(dev->dev, BNO085_REG_COMMAND, freq_bytes, 4);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to set sampling frequency: %d\n", ret);
        return ret;
    }
    
    dev->sampling_frequency = frequency;
    
    return 0;
}

int bno085_read_data(struct bno085_device *dev)
{
    int ret;
    
    /* Read accelerometer data */
    if (dev->enabled_features & BNO085_FEATURE_ACCELEROMETER) {
        ret = dev->transport.read(dev->dev, BNO085_REG_ACCEL_X, (u8 *)dev->accel_data, 6);
        if (ret < 0) {
            dev_err(dev->dev, "Failed to read accelerometer data: %d\n", ret);
            return ret;
        }
    }
    
    /* Read gyroscope data */
    if (dev->enabled_features & BNO085_FEATURE_GYROSCOPE) {
        ret = dev->transport.read(dev->dev, BNO085_REG_GYRO_X, (u8 *)dev->gyro_data, 6);
        if (ret < 0) {
            dev_err(dev->dev, "Failed to read gyroscope data: %d\n", ret);
            return ret;
        }
    }
    
    /* Read magnetometer data */
    if (dev->enabled_features & BNO085_FEATURE_MAGNETOMETER) {
        ret = dev->transport.read(dev->dev, BNO085_REG_MAG_X, (u8 *)dev->mag_data, 6);
        if (ret < 0) {
            dev_err(dev->dev, "Failed to read magnetometer data: %d\n", ret);
            return ret;
        }
    }
    
    /* Read quaternion data */
    if (dev->enabled_features & BNO085_FEATURE_ROTATION_VECTOR) {
        ret = dev->transport.read(dev->dev, BNO085_REG_QUAT_W, (u8 *)dev->quaternion_data, 8);
        if (ret < 0) {
            dev_err(dev->dev, "Failed to read quaternion data: %d\n", ret);
            return ret;
        }
    }
    
    /* Read temperature data */
    ret = dev->transport.read(dev->dev, BNO085_REG_TEMP, (u8 *)&dev->temperature_data, 2);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to read temperature data: %d\n", ret);
        return ret;
    }
    
    /* Update timestamp */
    dev->last_sample_time = ktime_get();
    
    return 0;
}

int bno085_reset(struct bno085_device *dev)
{
    u8 reset_cmd = BNO085_RESET_COMMAND;
    int ret;
    u8 status;
    
    /* Send reset command */
    ret = dev->transport.write(dev->dev, BNO085_REG_RESET, &reset_cmd, 1);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to send reset command: %d\n", ret);
        return ret;
    }
    
    /* Wait for reset to complete */
    msleep(100);
    
    /* Check reset status */
    ret = dev->transport.read(dev->dev, BNO085_REG_STATUS, &status, 1);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to read status after reset: %d\n", ret);
        return ret;
    }
    
    if (!(status & BNO085_STATUS_RESET_DONE)) {
        dev_err(dev->dev, "Reset failed to complete\n");
        return -EIO;
    }
    
    /* Reset device state */
    dev->state = BNO085_STATE_INITIALIZING;
    dev->mode = BNO085_MODE_CONFIG;
    dev->enabled_features = 0;
    dev->sampling_frequency = 0;
    dev->calibrated = false;
    
    return 0;
}

int bno085_update_calibration(struct bno085_device *dev)
{
    int ret;
    u8 calib_status;
    
    ret = dev->transport.read(dev->dev, BNO085_REG_CALIB_STATUS, &calib_status, 1);
    if (ret < 0) {
        dev_err(dev->dev, "Failed to read calibration status: %d\n", ret);
        return ret;
    }
    
    /* Check if all sensors are calibrated */
    if ((calib_status & 0x3F) == 0x3F) {
        dev->calibrated = true;
        
        /* Read calibration data */
        ret = dev->transport.read(dev->dev, BNO085_REG_COMMAND, dev->accel_calib, 6);
        if (ret < 0) {
            dev_err(dev->dev, "Failed to read accelerometer calibration: %d\n", ret);
            return ret;
        }
        
        ret = dev->transport.read(dev->dev, BNO085_REG_COMMAND, dev->gyro_calib, 6);
        if (ret < 0) {
            dev_err(dev->dev, "Failed to read gyroscope calibration: %d\n", ret);
            return ret;
        }
        
        ret = dev->transport.read(dev->dev, BNO085_REG_COMMAND, dev->mag_calib, 6);
        if (ret < 0) {
            dev_err(dev->dev, "Failed to read magnetometer calibration: %d\n", ret);
            return ret;
        }
    }
    
    return 0;
}

/* Power management operations */
#ifdef CONFIG_PM
int bno085_suspend(struct device *dev)
{
    struct iio_dev *indio_dev = dev_get_drvdata(dev);
    struct bno085_device *bno085_dev = iio_priv(indio_dev);
    
    if (bno085_dev->buffer_enabled)
        bno085_buffer_predisable(indio_dev);
    
    /* Put device in low power mode */
    bno085_set_mode(bno085_dev, BNO085_MODE_CONFIG);
    
    return 0;
}

int bno085_resume(struct device *dev)
{
    struct iio_dev *indio_dev = dev_get_drvdata(dev);
    struct bno085_device *bno085_dev = iio_priv(indio_dev);
    
    /* Restore previous mode */
    bno085_set_mode(bno085_dev, bno085_dev->mode);
    
    if (bno085_dev->buffer_enabled)
        bno085_buffer_postenable(indio_dev);
    
    return 0;
}

const struct dev_pm_ops bno085_pm_ops = {
    SET_SYSTEM_SLEEP_PM_OPS(bno085_suspend, bno085_resume)
};
EXPORT_SYMBOL_GPL(bno085_pm_ops);
#endif

MODULE_AUTHOR("VR Headset Project");
MODULE_DESCRIPTION("BNO085 IMU driver");
MODULE_LICENSE("GPL v2");
