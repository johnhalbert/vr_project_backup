// SPDX-License-Identifier: GPL-2.0
/*
 * OV9281 Camera Driver
 *
 * Copyright (C) 2025 VR Headset Project
 */

#include <linux/module.h>
#include <linux/init.h>
#include <linux/delay.h>
#include <linux/clk.h>
#include <linux/gpio/consumer.h>
#include <linux/i2c.h>
#include <linux/regulator/consumer.h>
#include <linux/of.h>
#include <linux/of_device.h>
#include <linux/of_graph.h>
#include <linux/pm_runtime.h>
#include <linux/videodev2.h>
#include <linux/v4l2-mediabus.h>
#include <media/v4l2-device.h>
#include <media/v4l2-subdev.h>
#include <media/v4l2-ctrls.h>
#include <media/v4l2-fwnode.h>
#include <media/v4l2-event.h>
#include <linux/debugfs.h>
#include <linux/dma-mapping.h>

#include "ov9281_core.h"

/* Register access functions */
static int ov9281_read_reg(struct ov9281_device *dev, u16 reg, u8 *val)
{
    struct i2c_client *client = dev->client;
    struct i2c_msg msg[2];
    u8 buf[2];
    int ret;

    buf[0] = reg >> 8;
    buf[1] = reg & 0xff;

    /* Write register address */
    msg[0].addr = client->addr;
    msg[0].flags = 0;
    msg[0].len = 2;
    msg[0].buf = buf;

    /* Read data */
    msg[1].addr = client->addr;
    msg[1].flags = I2C_M_RD;
    msg[1].len = 1;
    msg[1].buf = val;

    ret = i2c_transfer(client->adapter, msg, 2);
    if (ret != 2) {
        dev_err(&client->dev, "Failed to read reg 0x%04x: %d\n", reg, ret);
        return ret < 0 ? ret : -EIO;
    }

    return 0;
}

static int ov9281_write_reg(struct ov9281_device *dev, u16 reg, u8 val)
{
    struct i2c_client *client = dev->client;
    struct i2c_msg msg;
    u8 buf[3];
    int ret;

    buf[0] = reg >> 8;
    buf[1] = reg & 0xff;
    buf[2] = val;

    msg.addr = client->addr;
    msg.flags = 0;
    msg.len = 3;
    msg.buf = buf;

    ret = i2c_transfer(client->adapter, &msg, 1);
    if (ret != 1) {
        dev_err(&client->dev, "Failed to write reg 0x%04x: %d\n", reg, ret);
        return ret < 0 ? ret : -EIO;
    }

    return 0;
}

static int ov9281_read_reg16(struct ov9281_device *dev, u16 reg, u16 *val)
{
    u8 hi, lo;
    int ret;

    ret = ov9281_read_reg(dev, reg, &hi);
    if (ret)
        return ret;

    ret = ov9281_read_reg(dev, reg + 1, &lo);
    if (ret)
        return ret;

    *val = (hi << 8) | lo;
    return 0;
}

static int ov9281_write_reg16(struct ov9281_device *dev, u16 reg, u16 val)
{
    int ret;

    ret = ov9281_write_reg(dev, reg, val >> 8);
    if (ret)
        return ret;

    return ov9281_write_reg(dev, reg + 1, val & 0xff);
}

/* Register array for different modes */
struct ov9281_reg {
    u16 addr;
    u8 val;
};

/* Register settings for different frame rates */
static const struct ov9281_reg ov9281_30fps_regs[] = {
    {OV9281_REG_TIMING_HTS_H, 0x0A},
    {OV9281_REG_TIMING_HTS_L, 0x00},
    {OV9281_REG_TIMING_VTS_H, 0x04},
    {OV9281_REG_TIMING_VTS_L, 0x65},
    {OV9281_REG_CLK_CTRL, 0x10},
    {0xFFFF, 0xFF}, /* End of array */
};

static const struct ov9281_reg ov9281_60fps_regs[] = {
    {OV9281_REG_TIMING_HTS_H, 0x05},
    {OV9281_REG_TIMING_HTS_L, 0x00},
    {OV9281_REG_TIMING_VTS_H, 0x04},
    {OV9281_REG_TIMING_VTS_L, 0x65},
    {OV9281_REG_CLK_CTRL, 0x10},
    {0xFFFF, 0xFF}, /* End of array */
};

static const struct ov9281_reg ov9281_90fps_regs[] = {
    {OV9281_REG_TIMING_HTS_H, 0x03},
    {OV9281_REG_TIMING_HTS_L, 0x55},
    {OV9281_REG_TIMING_VTS_H, 0x04},
    {OV9281_REG_TIMING_VTS_L, 0x65},
    {OV9281_REG_CLK_CTRL, 0x0C},
    {0xFFFF, 0xFF}, /* End of array */
};

static const struct ov9281_reg ov9281_120fps_regs[] = {
    {OV9281_REG_TIMING_HTS_H, 0x02},
    {OV9281_REG_TIMING_HTS_L, 0x80},
    {OV9281_REG_TIMING_VTS_H, 0x04},
    {OV9281_REG_TIMING_VTS_L, 0x65},
    {OV9281_REG_CLK_CTRL, 0x0A},
    {0xFFFF, 0xFF}, /* End of array */
};

static const struct ov9281_reg ov9281_150fps_regs[] = {
    {OV9281_REG_TIMING_HTS_H, 0x02},
    {OV9281_REG_TIMING_HTS_L, 0x00},
    {OV9281_REG_TIMING_VTS_H, 0x04},
    {OV9281_REG_TIMING_VTS_L, 0x65},
    {OV9281_REG_CLK_CTRL, 0x08},
    {0xFFFF, 0xFF}, /* End of array */
};

static const struct ov9281_reg ov9281_180fps_regs[] = {
    {OV9281_REG_TIMING_HTS_H, 0x01},
    {OV9281_REG_TIMING_HTS_L, 0xAA},
    {OV9281_REG_TIMING_VTS_H, 0x04},
    {OV9281_REG_TIMING_VTS_L, 0x65},
    {OV9281_REG_CLK_CTRL, 0x06},
    {0xFFFF, 0xFF}, /* End of array */
};

/* Register settings for different sync modes */
static const struct ov9281_reg ov9281_master_regs[] = {
    {OV9281_REG_SYNC_MODE, 0x00},
    {0xFFFF, 0xFF}, /* End of array */
};

static const struct ov9281_reg ov9281_slave_regs[] = {
    {OV9281_REG_SYNC_MODE, 0x01},
    {0xFFFF, 0xFF}, /* End of array */
};

static const struct ov9281_reg ov9281_external_regs[] = {
    {OV9281_REG_SYNC_MODE, 0x02},
    {0xFFFF, 0xFF}, /* End of array */
};

/* Register settings for VR mode */
static const struct ov9281_reg ov9281_vr_mode_regs[] = {
    {OV9281_REG_EXPOSURE_CTRL, 0x01},
    {OV9281_REG_MIPI_CTRL_00, 0x24},
    {OV9281_REG_MIPI_CTRL_01, 0x0F},
    {OV9281_REG_MIPI_CTRL_05, 0x10},
    {0xFFFF, 0xFF}, /* End of array */
};

/* Register settings for low latency mode */
static const struct ov9281_reg ov9281_low_latency_regs[] = {
    {OV9281_REG_FRAME_CTRL, 0x00},
    {OV9281_REG_FORMAT_CTRL, 0x80},
    {0xFFFF, 0xFF}, /* End of array */
};

/* Register settings for initialization */
static const struct ov9281_reg ov9281_init_regs[] = {
    {OV9281_REG_MODE_SELECT, 0x00},
    {OV9281_REG_SC_CMMN_PAD_OEN0, 0xFF},
    {OV9281_REG_SC_CMMN_PAD_OEN1, 0xFF},
    {OV9281_REG_SC_CMMN_PAD_OEN2, 0xE3},
    {OV9281_REG_SC_CMMN_PAD_OUT0, 0x00},
    {OV9281_REG_SC_CMMN_PAD_OUT1, 0x00},
    {OV9281_REG_SC_CMMN_PAD_OUT2, 0x00},
    {OV9281_REG_SC_CMMN_PAD_SEL0, 0x00},
    {OV9281_REG_SC_CMMN_PAD_SEL1, 0x00},
    {OV9281_REG_SC_CMMN_PAD_SEL2, 0x00},
    {OV9281_REG_AEC_MANUAL, 0x01},
    {OV9281_REG_TIMING_X_INC, 0x11},
    {OV9281_REG_TIMING_Y_INC, 0x11},
    {OV9281_REG_HVOFFS_H, 0x00},
    {OV9281_REG_HVOFFS_L, 0x00},
    {OV9281_REG_VFLIP, 0x00},
    {OV9281_REG_HFLIP, 0x00},
    {OV9281_REG_FORMAT1, 0x00},
    {OV9281_REG_FORMAT2, 0x00},
    {OV9281_REG_ISP_CTRL, 0x00},
    {OV9281_REG_ISP_CTRL2, 0x00},
    {0xFFFF, 0xFF}, /* End of array */
};

/* Apply register settings */
static int ov9281_write_reg_array(struct ov9281_device *dev, const struct ov9281_reg *regs)
{
    int ret = 0;
    
    while (regs->addr != 0xFFFF) {
        ret = ov9281_write_reg(dev, regs->addr, regs->val);
        if (ret)
            return ret;
        regs++;
    }
    
    return 0;
}

/* Core initialization function */
int ov9281_core_init(struct ov9281_device *dev)
{
    struct i2c_client *client = dev->client;
    u16 chip_id;
    int ret;
    
    /* Check device ID */
    ret = ov9281_read_reg16(dev, OV9281_REG_CHIP_ID_HIGH, &chip_id);
    if (ret) {
        dev_err(&client->dev, "Failed to read chip ID: %d\n", ret);
        return ret;
    }
    
    if (chip_id != OV9281_CHIP_ID) {
        dev_err(&client->dev, "Unexpected chip ID: 0x%04x (expected 0x%04x)\n",
                chip_id, OV9281_CHIP_ID);
        return -ENODEV;
    }
    
    /* Reset device */
    ret = ov9281_reset(dev);
    if (ret) {
        dev_err(&client->dev, "Failed to reset device: %d\n", ret);
        return ret;
    }
    
    /* Apply initialization settings */
    ret = ov9281_write_reg_array(dev, ov9281_init_regs);
    if (ret) {
        dev_err(&client->dev, "Failed to apply init settings: %d\n", ret);
        return ret;
    }
    
    /* Configure default frame rate */
    ret = ov9281_set_frame_rate(dev, OV9281_60_FPS);
    if (ret) {
        dev_err(&client->dev, "Failed to set default frame rate: %d\n", ret);
        return ret;
    }
    
    /* Configure default sync mode */
    ret = ov9281_set_mode(dev, OV9281_SYNC_MODE_MASTER);
    if (ret) {
        dev_err(&client->dev, "Failed to set default sync mode: %d\n", ret);
        return ret;
    }
    
    /* Set default exposure and gain */
    ret = ov9281_set_exposure(dev, OV9281_EXPOSURE_DEFAULT);
    if (ret) {
        dev_err(&client->dev, "Failed to set default exposure: %d\n", ret);
        return ret;
    }
    
    ret = ov9281_set_gain(dev, OV9281_GAIN_DEFAULT);
    if (ret) {
        dev_err(&client->dev, "Failed to set default gain: %d\n", ret);
        return ret;
    }
    
    /* Update device state */
    dev->state = OV9281_STATE_INITIALIZED;
    
    dev_info(&client->dev, "OV9281 camera initialized\n");
    
    return 0;
}

/* Device operations */
int ov9281_set_mode(struct ov9281_device *dev, enum ov9281_sync_mode mode)
{
    int ret;
    
    if (dev->sync_mode == mode)
        return 0;
    
    switch (mode) {
    case OV9281_SYNC_MODE_MASTER:
        ret = ov9281_write_reg_array(dev, ov9281_master_regs);
        if (ret)
            return ret;
        dev->is_master = true;
        break;
    case OV9281_SYNC_MODE_SLAVE:
        ret = ov9281_write_reg_array(dev, ov9281_slave_regs);
        if (ret)
            return ret;
        dev->is_master = false;
        break;
    case OV9281_SYNC_MODE_EXTERNAL:
        ret = ov9281_write_reg_array(dev, ov9281_external_regs);
        if (ret)
            return ret;
        dev->is_master = false;
        break;
    default:
        return -EINVAL;
    }
    
    dev->sync_mode = mode;
    
    return 0;
}

int ov9281_set_frame_rate(struct ov9281_device *dev, enum ov9281_frame_rate rate)
{
    int ret;
    
    if (dev->frame_rate == rate)
        return 0;
    
    switch (rate) {
    case OV9281_30_FPS:
        ret = ov9281_write_reg_array(dev, ov9281_30fps_regs);
        if (ret)
            return ret;
        dev->hts = 0x0A00;
        dev->vts = 0x0465;
        dev->high_framerate = false;
        break;
    case OV9281_60_FPS:
        ret = ov9281_write_reg_array(dev, ov9281_60fps_regs);
        if (ret)
            return ret;
        dev->hts = 0x0500;
        dev->vts = 0x0465;
        dev->high_framerate = false;
        break;
    case OV9281_90_FPS:
        ret = ov9281_write_reg_array(dev, ov9281_90fps_regs);
        if (ret)
            return ret;
        dev->hts = 0x0355;
        dev->vts = 0x0465;
        dev->high_framerate = true;
        break;
    case OV9281_120_FPS:
        ret = ov9281_write_reg_array(dev, ov9281_120fps_regs);
        if (ret)
            return ret;
        dev->hts = 0x0280;
        dev->vts = 0x0465;
        dev->high_framerate = true;
        break;
    case OV9281_150_FPS:
        ret = ov9281_write_reg_array(dev, ov9281_150fps_regs);
        if (ret)
            return ret;
        dev->hts = 0x0200;
        dev->vts = 0x0465;
        dev->high_framerate = true;
        break;
    case OV9281_180_FPS:
        ret = ov9281_write_reg_array(dev, ov9281_180fps_regs);
        if (ret)
            return ret;
        dev->hts = 0x01AA;
        dev->vts = 0x0465;
        dev->high_framerate = true;
        break;
    default:
        return -EINVAL;
    }
    
    dev->frame_rate = rate;
    
    return 0;
}

int ov9281_set_test_pattern(struct ov9281_device *dev, enum ov9281_test_pattern pattern)
{
    int ret;
    u8 val;
    
    ret = ov9281_read_reg(dev, OV9281_REG_ISP_CTRL, &val);
    if (ret)
        return ret;
    
    switch (pattern) {
    case OV9281_TEST_PATTERN_DISABLED:
        val &= ~0x80;
        val &= ~0x03;
        break;
    case OV9281_TEST_PATTERN_SOLID_COLOR:
        val |= 0x80;
        val &= ~0x03;
        break;
    case OV9281_TEST_PATTERN_COLOR_BARS:
        val |= 0x80;
        val &= ~0x03;
        val |= 0x01;
        break;
    case OV9281_TEST_PATTERN_GRADIENT_H:
        val |= 0x80;
        val &= ~0x03;
        val |= 0x02;
        break;
    case OV9281_TEST_PATTERN_GRADIENT_V:
        val |= 0x80;
        val |= 0x03;
        break;
    default:
        return -EINVAL;
    }
    
    return ov9281_write_reg(dev, OV9281_REG_ISP_CTRL, val);
}

int ov9281_set_exposure(struct ov9281_device *dev, u32 exposure)
{
    int ret;
    
    if (exposure > OV9281_EXPOSURE_MAX)
        exposure = OV9281_EXPOSURE_MAX;
    else if (exposure < OV9281_EXPOSURE_MIN)
        exposure = OV9281_EXPOSURE_MIN;
    
    ret = ov9281_write_reg(dev, OV9281_REG_AEC_EXPO_H, (exposure >> 16) & 0x0F);
    if (ret)
        return ret;
    
    ret = ov9281_write_reg(dev, OV9281_REG_AEC_EXPO_M, (exposure >> 8) & 0xFF);
    if (ret)
        return ret;
    
    return ov9281_write_reg(dev, OV9281_REG_AEC_EXPO_L, exposure & 0xFF);
}

int ov9281_set_gain(struct ov9281_device *dev, u32 gain)
{
    int ret;
    
    if (gain > OV9281_GAIN_MAX)
        gain = OV9281_GAIN_MAX;
    else if (gain < OV9281_GAIN_MIN)
        gain = OV9281_GAIN_MIN;
    
    ret = ov9281_write_reg(dev, OV9281_REG_AEC_AGC_ADJ_H, (gain >> 8) & 0x0F);
    if (ret)
        return ret;
    
    return ov9281_write_reg(dev, OV9281_REG_AEC_AGC_ADJ_L, gain & 0xFF);
}

int ov9281_set_flip(struct ov9281_device *dev, bool hflip, bool vflip)
{
    int ret;
    u8 val_h, val_v;
    
    ret = ov9281_read_reg(dev, OV9281_REG_HFLIP, &val_h);
    if (ret)
        return ret;
    
    ret = ov9281_read_reg(dev, OV9281_REG_VFLIP, &val_v);
    if (ret)
        return ret;
    
    if (hflip)
        val_h |= OV9281_FLIP_ENABLE;
    else
        val_h &= ~OV9281_FLIP_ENABLE;
    
    if (vflip)
        val_v |= OV9281_FLIP_ENABLE;
    else
        val_v &= ~OV9281_FLIP_ENABLE;
    
    ret = ov9281_write_reg(dev, OV9281_REG_HFLIP, val_h);
    if (ret)
        return ret;
    
    return ov9281_write_reg(dev, OV9281_REG_VFLIP, val_v);
}

int ov9281_start_streaming(struct ov9281_device *dev)
{
    int ret;
    
    if (dev->state == OV9281_STATE_STREAMING)
        return 0;
    
    /* Apply VR mode settings if enabled */
    if (dev->vr_mode) {
        ret = ov9281_write_reg_array(dev, ov9281_vr_mode_regs);
        if (ret)
            return ret;
    }
    
    /* Apply low latency settings if enabled */
    if (dev->low_latency) {
        ret = ov9281_write_reg_array(dev, ov9281_low_latency_regs);
        if (ret)
            return ret;
    }
    
    /* Start streaming */
    ret = ov9281_write_reg(dev, OV9281_REG_STREAM_CTRL, OV9281_MODE_STREAMING);
    if (ret)
        return ret;
    
    /* Synchronize sensors if master */
    if (dev->is_master && dev->num_slaves > 0) {
        ret = ov9281_sync_sensors(dev);
        if (ret)
            return ret;
    }
    
    dev->state = OV9281_STATE_STREAMING;
    
    return 0;
}

int ov9281_stop_streaming(struct ov9281_device *dev)
{
    int ret;
    
    if (dev->state != OV9281_STATE_STREAMING)
        return 0;
    
    ret = ov9281_write_reg(dev, OV9281_REG_STREAM_CTRL, OV9281_MODE_SW_STANDBY);
    if (ret)
        return ret;
    
    dev->state = OV9281_STATE_INITIALIZED;
    
    return 0;
}

int ov9281_reset(struct ov9281_device *dev)
{
    int ret;
    
    /* Software reset */
    ret = ov9281_write_reg(dev, OV9281_REG_MODE_SELECT, OV9281_RESET_VALUE);
    if (ret)
        return ret;
    
    /* Wait for reset to complete */
    msleep(OV9281_RESET_DELAY_MS);
    
    /* Reset device state */
    dev->state = OV9281_STATE_INITIALIZING;
    dev->sync_mode = OV9281_SYNC_MODE_MASTER;
    dev->frame_rate = OV9281_60_FPS;
    dev->is_master = true;
    dev->vr_mode = false;
    dev->low_latency = false;
    dev->high_framerate = false;
    
    return 0;
}

int ov9281_enable_zero_copy(struct ov9281_device *dev, bool enable)
{
    struct i2c_client *client = dev->client;
    
    if (dev->zero_copy_enabled == enable)
        return 0;
    
    if (enable) {
        /* Allocate DMA buffer */
        dev->dma_size = OV9281_MAX_WIDTH * OV9281_MAX_HEIGHT * 2; /* 10-bit per pixel */
        dev->dma_buffer = dma_alloc_coherent(&client->dev, dev->dma_size,
                                           &dev->dma_addr, GFP_KERNEL);
        if (!dev->dma_buffer) {
            dev_err(&client->dev, "Failed to allocate DMA buffer\n");
            return -ENOMEM;
        }
        
        dev->zero_copy_enabled = true;
        dev_info(&client->dev, "Zero-copy mode enabled\n");
    } else {
        /* Free DMA buffer */
        if (dev->dma_buffer) {
            dma_free_coherent(&client->dev, dev->dma_size,
                            dev->dma_buffer, dev->dma_addr);
            dev->dma_buffer = NULL;
            dev->dma_addr = 0;
            dev->dma_size = 0;
        }
        
        dev->zero_copy_enabled = false;
        dev_info(&client->dev, "Zero-copy mode disabled\n");
    }
    
    return 0;
}

int ov9281_sync_sensors(struct ov9281_device *dev)
{
    int i, ret;
    
    if (!dev->is_master || dev->num_slaves == 0)
        return 0;
    
    /* Ensure all slaves are in slave mode */
    for (i = 0; i < dev->num_slaves; i++) {
        if (!dev->slaves[i])
            continue;
        
        ret = ov9281_set_mode(dev->slaves[i], OV9281_SYNC_MODE_SLAVE);
        if (ret)
            return ret;
    }
    
    /* Trigger synchronization pulse */
    if (dev->sync_gpio >= 0) {
        gpio_set_value(dev->sync_gpio, 1);
        udelay(10);
        gpio_set_value(dev->sync_gpio, 0);
    }
    
    return 0;
}

/* V4L2 subdev operations */
static int ov9281_s_power(struct v4l2_subdev *sd, int on)
{
    struct ov9281_device *dev = container_of(sd, struct ov9281_device, sd);
    struct i2c_client *client = dev->client;
    int ret = 0;
    
    mutex_lock(&dev->lock);
    
    if (on) {
        /* Enable power supplies */
        if (dev->avdd) {
            ret = regulator_enable(dev->avdd);
            if (ret) {
                dev_err(&client->dev, "Failed to enable AVDD: %d\n", ret);
                goto unlock;
            }
        }
        
        if (dev->dovdd) {
            ret = regulator_enable(dev->dovdd);
            if (ret) {
                dev_err(&client->dev, "Failed to enable DOVDD: %d\n", ret);
                goto disable_avdd;
            }
        }
        
        if (dev->dvdd) {
            ret = regulator_enable(dev->dvdd);
            if (ret) {
                dev_err(&client->dev, "Failed to enable DVDD: %d\n", ret);
                goto disable_dovdd;
            }
        }
        
        /* Enable clock */
        if (dev->xvclk) {
            ret = clk_prepare_enable(dev->xvclk);
            if (ret) {
                dev_err(&client->dev, "Failed to enable XVCLK: %d\n", ret);
                goto disable_dvdd;
            }
        }
        
        /* De-assert reset */
        if (dev->reset_gpio >= 0) {
            gpio_set_value(dev->reset_gpio, 1);
            msleep(10);
        }
        
        /* De-assert power down */
        if (dev->pwdn_gpio >= 0) {
            gpio_set_value(dev->pwdn_gpio, 0);
            msleep(10);
        }
        
        /* Initialize device */
        ret = ov9281_core_init(dev);
        if (ret) {
            dev_err(&client->dev, "Failed to initialize device: %d\n", ret);
            goto disable_clk;
        }
    } else {
        /* Stop streaming */
        ov9281_stop_streaming(dev);
        
        /* Assert power down */
        if (dev->pwdn_gpio >= 0)
            gpio_set_value(dev->pwdn_gpio, 1);
        
        /* Assert reset */
        if (dev->reset_gpio >= 0)
            gpio_set_value(dev->reset_gpio, 0);
        
        /* Disable clock */
        if (dev->xvclk)
            clk_disable_unprepare(dev->xvclk);
        
        /* Disable power supplies */
        if (dev->dvdd)
            regulator_disable(dev->dvdd);
        
        if (dev->dovdd)
            regulator_disable(dev->dovdd);
        
        if (dev->avdd)
            regulator_disable(dev->avdd);
        
        dev->state = OV9281_STATE_DISABLED;
    }
    
    goto unlock;
    
disable_clk:
    if (dev->xvclk)
        clk_disable_unprepare(dev->xvclk);
disable_dvdd:
    if (dev->dvdd)
        regulator_disable(dev->dvdd);
disable_dovdd:
    if (dev->dovdd)
        regulator_disable(dev->dovdd);
disable_avdd:
    if (dev->avdd)
        regulator_disable(dev->avdd);
unlock:
    mutex_unlock(&dev->lock);
    return ret;
}

static int ov9281_g_frame_interval(struct v4l2_subdev *sd,
                                 struct v4l2_subdev_frame_interval *fi)
{
    struct ov9281_device *dev = container_of(sd, struct ov9281_device, sd);
    
    mutex_lock(&dev->lock);
    
    switch (dev->frame_rate) {
    case OV9281_30_FPS:
        fi->interval.numerator = 1;
        fi->interval.denominator = 30;
        break;
    case OV9281_60_FPS:
        fi->interval.numerator = 1;
        fi->interval.denominator = 60;
        break;
    case OV9281_90_FPS:
        fi->interval.numerator = 1;
        fi->interval.denominator = 90;
        break;
    case OV9281_120_FPS:
        fi->interval.numerator = 1;
        fi->interval.denominator = 120;
        break;
    case OV9281_150_FPS:
        fi->interval.numerator = 1;
        fi->interval.denominator = 150;
        break;
    case OV9281_180_FPS:
        fi->interval.numerator = 1;
        fi->interval.denominator = 180;
        break;
    default:
        fi->interval.numerator = 1;
        fi->interval.denominator = 60;
        break;
    }
    
    mutex_unlock(&dev->lock);
    
    return 0;
}

static int ov9281_s_frame_interval(struct v4l2_subdev *sd,
                                 struct v4l2_subdev_frame_interval *fi)
{
    struct ov9281_device *dev = container_of(sd, struct ov9281_device, sd);
    int ret = 0;
    
    mutex_lock(&dev->lock);
    
    if (fi->interval.numerator == 0 || fi->interval.denominator == 0) {
        /* Default to 60 FPS */
        ret = ov9281_set_frame_rate(dev, OV9281_60_FPS);
    } else {
        int fps = fi->interval.denominator / fi->interval.numerator;
        
        if (fps <= 30)
            ret = ov9281_set_frame_rate(dev, OV9281_30_FPS);
        else if (fps <= 60)
            ret = ov9281_set_frame_rate(dev, OV9281_60_FPS);
        else if (fps <= 90)
            ret = ov9281_set_frame_rate(dev, OV9281_90_FPS);
        else if (fps <= 120)
            ret = ov9281_set_frame_rate(dev, OV9281_120_FPS);
        else if (fps <= 150)
            ret = ov9281_set_frame_rate(dev, OV9281_150_FPS);
        else
            ret = ov9281_set_frame_rate(dev, OV9281_180_FPS);
    }
    
    mutex_unlock(&dev->lock);
    
    return ret;
}

static int ov9281_enum_mbus_code(struct v4l2_subdev *sd,
                               struct v4l2_subdev_pad_config *cfg,
                               struct v4l2_subdev_mbus_code_enum *code)
{
    if (code->index > 0)
        return -EINVAL;
    
    code->code = MEDIA_BUS_FMT_Y10_1X10;
    
    return 0;
}

static int ov9281_enum_frame_size(struct v4l2_subdev *sd,
                                struct v4l2_subdev_pad_config *cfg,
                                struct v4l2_subdev_frame_size_enum *fse)
{
    if (fse->index > 0)
        return -EINVAL;
    
    if (fse->code != MEDIA_BUS_FMT_Y10_1X10)
        return -EINVAL;
    
    fse->min_width = OV9281_MIN_WIDTH;
    fse->max_width = OV9281_MAX_WIDTH;
    fse->min_height = OV9281_MIN_HEIGHT;
    fse->max_height = OV9281_MAX_HEIGHT;
    
    return 0;
}

static int ov9281_get_fmt(struct v4l2_subdev *sd,
                        struct v4l2_subdev_pad_config *cfg,
                        struct v4l2_subdev_format *format)
{
    struct ov9281_device *dev = container_of(sd, struct ov9281_device, sd);
    
    mutex_lock(&dev->lock);
    
    format->format = dev->fmt;
    
    mutex_unlock(&dev->lock);
    
    return 0;
}

static int ov9281_set_fmt(struct v4l2_subdev *sd,
                        struct v4l2_subdev_pad_config *cfg,
                        struct v4l2_subdev_format *format)
{
    struct ov9281_device *dev = container_of(sd, struct ov9281_device, sd);
    struct v4l2_mbus_framefmt *fmt = &format->format;
    
    mutex_lock(&dev->lock);
    
    /* Only Y10 format is supported */
    if (fmt->code != MEDIA_BUS_FMT_Y10_1X10)
        fmt->code = MEDIA_BUS_FMT_Y10_1X10;
    
    /* Clamp width and height */
    if (fmt->width > OV9281_MAX_WIDTH)
        fmt->width = OV9281_MAX_WIDTH;
    else if (fmt->width < OV9281_MIN_WIDTH)
        fmt->width = OV9281_MIN_WIDTH;
    
    if (fmt->height > OV9281_MAX_HEIGHT)
        fmt->height = OV9281_MAX_HEIGHT;
    else if (fmt->height < OV9281_MIN_HEIGHT)
        fmt->height = OV9281_MIN_HEIGHT;
    
    /* Set field and colorspace */
    fmt->field = V4L2_FIELD_NONE;
    fmt->colorspace = V4L2_COLORSPACE_RAW;
    
    /* Update format */
    dev->fmt = *fmt;
    
    if (format->which == V4L2_SUBDEV_FORMAT_TRY)
        cfg->try_fmt = *fmt;
    
    mutex_unlock(&dev->lock);
    
    return 0;
}

static int ov9281_s_stream(struct v4l2_subdev *sd, int enable)
{
    struct ov9281_device *dev = container_of(sd, struct ov9281_device, sd);
    int ret;
    
    mutex_lock(&dev->lock);
    
    if (enable)
        ret = ov9281_start_streaming(dev);
    else
        ret = ov9281_stop_streaming(dev);
    
    mutex_unlock(&dev->lock);
    
    return ret;
}

/* V4L2 control operations */
static int ov9281_s_ctrl(struct v4l2_ctrl *ctrl)
{
    struct ov9281_device *dev = container_of(ctrl->handler, struct ov9281_device, ctrl_handler);
    int ret = 0;
    
    mutex_lock(&dev->lock);
    
    switch (ctrl->id) {
    case V4L2_CID_EXPOSURE:
        ret = ov9281_set_exposure(dev, ctrl->val);
        break;
    case V4L2_CID_GAIN:
        ret = ov9281_set_gain(dev, ctrl->val);
        break;
    case V4L2_CID_HFLIP:
        ret = ov9281_set_flip(dev, ctrl->val, dev->vflip->val);
        break;
    case V4L2_CID_VFLIP:
        ret = ov9281_set_flip(dev, dev->hflip->val, ctrl->val);
        break;
    case V4L2_CID_TEST_PATTERN:
        ret = ov9281_set_test_pattern(dev, ctrl->val);
        break;
    case V4L2_CID_PIXEL_RATE:
        /* Read-only control */
        break;
    case V4L2_CID_LINK_FREQ:
        /* Read-only control */
        break;
    default:
        if (ctrl->id == ov9281_ctrl_sync_mode.id) {
            ret = ov9281_set_mode(dev, ctrl->val);
        } else if (ctrl->id == ov9281_ctrl_frame_rate.id) {
            ret = ov9281_set_frame_rate(dev, ctrl->val);
        } else if (ctrl->id == ov9281_ctrl_vr_mode.id) {
            dev->vr_mode = ctrl->val;
        } else if (ctrl->id == ov9281_ctrl_low_latency.id) {
            dev->low_latency = ctrl->val;
        } else {
            ret = -EINVAL;
        }
        break;
    }
    
    mutex_unlock(&dev->lock);
    
    return ret;
}

/* V4L2 subdev operations */
const struct v4l2_subdev_core_ops ov9281_core_ops = {
    .s_power = ov9281_s_power,
};

const struct v4l2_subdev_video_ops ov9281_video_ops = {
    .g_frame_interval = ov9281_g_frame_interval,
    .s_frame_interval = ov9281_s_frame_interval,
    .s_stream = ov9281_s_stream,
};

const struct v4l2_subdev_pad_ops ov9281_pad_ops = {
    .enum_mbus_code = ov9281_enum_mbus_code,
    .enum_frame_size = ov9281_enum_frame_size,
    .get_fmt = ov9281_get_fmt,
    .set_fmt = ov9281_set_fmt,
};

const struct v4l2_subdev_ops ov9281_subdev_ops = {
    .core = &ov9281_core_ops,
    .video = &ov9281_video_ops,
    .pad = &ov9281_pad_ops,
};

/* V4L2 control operations */
const struct v4l2_ctrl_ops ov9281_ctrl_ops = {
    .s_ctrl = ov9281_s_ctrl,
};

/* Custom control configurations */
const struct v4l2_ctrl_config ov9281_ctrl_sync_mode = {
    .ops = &ov9281_ctrl_ops,
    .id = V4L2_CID_PRIVATE_BASE,
    .name = "Sync Mode",
    .type = V4L2_CTRL_TYPE_INTEGER,
    .min = OV9281_SYNC_MODE_MASTER,
    .max = OV9281_SYNC_MODE_EXTERNAL,
    .step = 1,
    .def = OV9281_SYNC_MODE_MASTER,
};

const struct v4l2_ctrl_config ov9281_ctrl_frame_rate = {
    .ops = &ov9281_ctrl_ops,
    .id = V4L2_CID_PRIVATE_BASE + 1,
    .name = "Frame Rate Mode",
    .type = V4L2_CTRL_TYPE_INTEGER,
    .min = OV9281_30_FPS,
    .max = OV9281_180_FPS,
    .step = 1,
    .def = OV9281_60_FPS,
};

const struct v4l2_ctrl_config ov9281_ctrl_vr_mode = {
    .ops = &ov9281_ctrl_ops,
    .id = V4L2_CID_PRIVATE_BASE + 2,
    .name = "VR Mode",
    .type = V4L2_CTRL_TYPE_BOOLEAN,
    .min = 0,
    .max = 1,
    .step = 1,
    .def = 0,
};

const struct v4l2_ctrl_config ov9281_ctrl_low_latency = {
    .ops = &ov9281_ctrl_ops,
    .id = V4L2_CID_PRIVATE_BASE + 3,
    .name = "Low Latency Mode",
    .type = V4L2_CTRL_TYPE_BOOLEAN,
    .min = 0,
    .max = 1,
    .step = 1,
    .def = 0,
};

/* Core probe function */
int ov9281_core_probe(struct i2c_client *client, const struct i2c_device_id *id)
{
    struct device *dev = &client->dev;
    struct device_node *node = dev->of_node;
    struct ov9281_device *ov9281_dev;
    struct v4l2_subdev *sd;
    struct media_pad *pad;
    struct v4l2_ctrl_handler *handler;
    int ret;
    
    /* Allocate device structure */
    ov9281_dev = devm_kzalloc(dev, sizeof(*ov9281_dev), GFP_KERNEL);
    if (!ov9281_dev)
        return -ENOMEM;
    
    ov9281_dev->client = client;
    
    /* Initialize mutex */
    mutex_init(&ov9281_dev->lock);
    
    /* Get resources */
    ov9281_dev->xvclk = devm_clk_get(dev, "xvclk");
    if (IS_ERR(ov9281_dev->xvclk)) {
        dev_err(dev, "Failed to get xvclk: %ld\n", PTR_ERR(ov9281_dev->xvclk));
        return PTR_ERR(ov9281_dev->xvclk);
    }
    
    ov9281_dev->xvclk_freq = clk_get_rate(ov9281_dev->xvclk);
    if (ov9281_dev->xvclk_freq != OV9281_XVCLK_FREQ) {
        dev_warn(dev, "xvclk frequency %u Hz differs from expected %u Hz\n",
                ov9281_dev->xvclk_freq, OV9281_XVCLK_FREQ);
    }
    
    ov9281_dev->reset_gpio = of_get_named_gpio(node, "reset-gpios", 0);
    if (ov9281_dev->reset_gpio < 0)
        dev_warn(dev, "No reset GPIO specified\n");
    
    ov9281_dev->pwdn_gpio = of_get_named_gpio(node, "powerdown-gpios", 0);
    if (ov9281_dev->pwdn_gpio < 0)
        dev_warn(dev, "No powerdown GPIO specified\n");
    
    ov9281_dev->sync_gpio = of_get_named_gpio(node, "sync-gpios", 0);
    if (ov9281_dev->sync_gpio < 0)
        dev_warn(dev, "No sync GPIO specified\n");
    
    /* Get regulators */
    ov9281_dev->avdd = devm_regulator_get(dev, "avdd");
    if (IS_ERR(ov9281_dev->avdd)) {
        dev_warn(dev, "Failed to get avdd regulator: %ld\n",
                PTR_ERR(ov9281_dev->avdd));
        ov9281_dev->avdd = NULL;
    }
    
    ov9281_dev->dovdd = devm_regulator_get(dev, "dovdd");
    if (IS_ERR(ov9281_dev->dovdd)) {
        dev_warn(dev, "Failed to get dovdd regulator: %ld\n",
                PTR_ERR(ov9281_dev->dovdd));
        ov9281_dev->dovdd = NULL;
    }
    
    ov9281_dev->dvdd = devm_regulator_get(dev, "dvdd");
    if (IS_ERR(ov9281_dev->dvdd)) {
        dev_warn(dev, "Failed to get dvdd regulator: %ld\n",
                PTR_ERR(ov9281_dev->dvdd));
        ov9281_dev->dvdd = NULL;
    }
    
    /* Initialize format */
    ov9281_dev->fmt.width = OV9281_DEFAULT_WIDTH;
    ov9281_dev->fmt.height = OV9281_DEFAULT_HEIGHT;
    ov9281_dev->fmt.code = OV9281_DEFAULT_MBUS_CODE;
    ov9281_dev->fmt.field = V4L2_FIELD_NONE;
    ov9281_dev->fmt.colorspace = V4L2_COLORSPACE_RAW;
    
    /* Initialize V4L2 subdev */
    sd = &ov9281_dev->sd;
    v4l2_i2c_subdev_init(sd, client, &ov9281_subdev_ops);
    
    /* Initialize media pad */
    pad = &ov9281_dev->pad;
    pad->flags = MEDIA_PAD_FL_SOURCE;
    ret = media_entity_pads_init(&sd->entity, 1, pad);
    if (ret) {
        dev_err(dev, "Failed to initialize media entity: %d\n", ret);
        goto err_mutex_destroy;
    }
    
    /* Initialize controls */
    handler = &ov9281_dev->ctrl_handler;
    ret = v4l2_ctrl_handler_init(handler, 10);
    if (ret) {
        dev_err(dev, "Failed to initialize control handler: %d\n", ret);
        goto err_media_entity_cleanup;
    }
    
    /* Standard controls */
    ov9281_dev->exposure = v4l2_ctrl_new_std(handler, &ov9281_ctrl_ops,
                                          V4L2_CID_EXPOSURE, OV9281_EXPOSURE_MIN,
                                          OV9281_EXPOSURE_MAX, OV9281_EXPOSURE_STEP,
                                          OV9281_EXPOSURE_DEFAULT);
    
    ov9281_dev->gain = v4l2_ctrl_new_std(handler, &ov9281_ctrl_ops,
                                      V4L2_CID_GAIN, OV9281_GAIN_MIN,
                                      OV9281_GAIN_MAX, OV9281_GAIN_STEP,
                                      OV9281_GAIN_DEFAULT);
    
    ov9281_dev->hflip = v4l2_ctrl_new_std(handler, &ov9281_ctrl_ops,
                                       V4L2_CID_HFLIP, 0, 1, 1, 0);
    
    ov9281_dev->vflip = v4l2_ctrl_new_std(handler, &ov9281_ctrl_ops,
                                       V4L2_CID_VFLIP, 0, 1, 1, 0);
    
    ov9281_dev->test_pattern = v4l2_ctrl_new_std_menu_items(handler, &ov9281_ctrl_ops,
                                                        V4L2_CID_TEST_PATTERN,
                                                        OV9281_TEST_PATTERN_GRADIENT_V,
                                                        0, 0,
                                                        ov9281_test_pattern_menu);
    
    /* Read-only controls */
    ov9281_dev->pixel_rate = v4l2_ctrl_new_std(handler, NULL,
                                            V4L2_CID_PIXEL_RATE,
                                            OV9281_PIXEL_RATE, OV9281_PIXEL_RATE,
                                            1, OV9281_PIXEL_RATE);
    
    ov9281_dev->link_freq = v4l2_ctrl_new_int_menu(handler, NULL,
                                                V4L2_CID_LINK_FREQ, 0, 0,
                                                &OV9281_DEFAULT_LINK_FREQ);
    
    /* Custom controls */
    v4l2_ctrl_new_custom(handler, &ov9281_ctrl_sync_mode, NULL);
    v4l2_ctrl_new_custom(handler, &ov9281_ctrl_frame_rate, NULL);
    v4l2_ctrl_new_custom(handler, &ov9281_ctrl_vr_mode, NULL);
    v4l2_ctrl_new_custom(handler, &ov9281_ctrl_low_latency, NULL);
    
    if (handler->error) {
        ret = handler->error;
        dev_err(dev, "Failed to initialize controls: %d\n", ret);
        goto err_free_handler;
    }
    
    sd->ctrl_handler = handler;
    
    /* Set default state */
    ov9281_dev->state = OV9281_STATE_DISABLED;
    ov9281_dev->sync_mode = OV9281_SYNC_MODE_MASTER;
    ov9281_dev->frame_rate = OV9281_60_FPS;
    ov9281_dev->is_master = true;
    ov9281_dev->vr_mode = false;
    ov9281_dev->low_latency = false;
    ov9281_dev->high_framerate = false;
    ov9281_dev->zero_copy_enabled = false;
    
    /* Register V4L2 subdev */
    ret = v4l2_async_register_subdev(sd);
    if (ret) {
        dev_err(dev, "Failed to register V4L2 subdev: %d\n", ret);
        goto err_free_handler;
    }
    
    /* Set up debugfs */
    ov9281_dev->debugfs_root = debugfs_create_dir("ov9281", NULL);
    
    /* Enable runtime PM */
    pm_runtime_enable(dev);
    
    dev_info(dev, "OV9281 camera driver probed\n");
    
    return 0;
    
err_free_handler:
    v4l2_ctrl_handler_free(handler);
err_media_entity_cleanup:
    media_entity_cleanup(&sd->entity);
err_mutex_destroy:
    mutex_destroy(&ov9281_dev->lock);
    return ret;
}

/* Core remove function */
int ov9281_core_remove(struct i2c_client *client)
{
    struct v4l2_subdev *sd = i2c_get_clientdata(client);
    struct ov9281_device *dev = container_of(sd, struct ov9281_device, sd);
    
    /* Disable runtime PM */
    pm_runtime_disable(&client->dev);
    
    /* Unregister V4L2 subdev */
    v4l2_async_unregister_subdev(sd);
    
    /* Clean up media entity */
    media_entity_cleanup(&sd->entity);
    
    /* Free control handler */
    v4l2_ctrl_handler_free(&dev->ctrl_handler);
    
    /* Free DMA buffer if allocated */
    if (dev->dma_buffer) {
        dma_free_coherent(&client->dev, dev->dma_size,
                        dev->dma_buffer, dev->dma_addr);
    }
    
    /* Remove debugfs */
    debugfs_remove_recursive(dev->debugfs_root);
    
    /* Destroy mutex */
    mutex_destroy(&dev->lock);
    
    return 0;
}

/* Module initialization */
static const struct i2c_device_id ov9281_id[] = {
    { "ov9281", 0 },
    { }
};
MODULE_DEVICE_TABLE(i2c, ov9281_id);

static const struct of_device_id ov9281_of_match[] = {
    { .compatible = "ovti,ov9281" },
    { }
};
MODULE_DEVICE_TABLE(of, ov9281_of_match);

static struct i2c_driver ov9281_i2c_driver = {
    .driver = {
        .name = "ov9281",
        .of_match_table = ov9281_of_match,
    },
    .probe = ov9281_core_probe,
    .remove = ov9281_core_remove,
    .id_table = ov9281_id,
};

module_i2c_driver(ov9281_i2c_driver);

MODULE_AUTHOR("VR Headset Project");
MODULE_DESCRIPTION("OV9281 Camera Driver");
MODULE_LICENSE("GPL v2");
