/*
 * Orange Pi CM5 VR Headset I2S Controller Driver
 *
 * Copyright (c) 2025 VR Headset Project
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License version 2 as
 * published by the Free Software Foundation.
 */

#include <linux/module.h>
#include <linux/platform_device.h>
#include <linux/of.h>
#include <linux/of_device.h>
#include <linux/clk.h>
#include <linux/pm_runtime.h>
#include <linux/regmap.h>
#include <linux/reset.h>
#include <linux/dmaengine.h>
#include <sound/soc.h>
#include <sound/pcm_params.h>

#include "orangepi_vr_i2s.h"

/* I2S controller registers */
#define ROCKCHIP_I2S_TXCR         0x0000
#define ROCKCHIP_I2S_RXCR         0x0004
#define ROCKCHIP_I2S_CKR          0x0008
#define ROCKCHIP_I2S_TXFIFOLR     0x000c
#define ROCKCHIP_I2S_DMACR        0x0010
#define ROCKCHIP_I2S_INTCR        0x0014
#define ROCKCHIP_I2S_INTSR        0x0018
#define ROCKCHIP_I2S_XFER         0x001c
#define ROCKCHIP_I2S_CLR          0x0020
#define ROCKCHIP_I2S_TXDR         0x0024
#define ROCKCHIP_I2S_RXDR         0x0028
#define ROCKCHIP_I2S_RXFIFOLR     0x002c
#define ROCKCHIP_I2S_TDM_TXCR     0x0030
#define ROCKCHIP_I2S_TDM_RXCR     0x0034
#define ROCKCHIP_I2S_CLKDIV       0x0038

/* TXCR/RXCR bits */
#define I2S_BIT_WIDTH_MASK        (0x1f << 0)
#define I2S_BIT_WIDTH(x)          ((x-1) << 0)
#define I2S_CHANNEL_WIDTH_MASK    (0x1f << 5)
#define I2S_CHANNEL_WIDTH(x)      ((x-1) << 5)
#define I2S_MONO_MASK             (0x1 << 10)
#define I2S_MONO                  (0x1 << 10)
#define I2S_STEREO                (0x0 << 10)
#define I2S_MONO_LEFT             (0x0 << 11)
#define I2S_MONO_RIGHT            (0x1 << 11)
#define I2S_TXCR_TFS_MASK         (0x3 << 12)
#define I2S_TXCR_TFS_I2S          (0x0 << 12)
#define I2S_TXCR_TFS_PCM          (0x1 << 12)
#define I2S_TXCR_TFS_TDM1         (0x2 << 12)
#define I2S_TXCR_TFS_TDM2         (0x3 << 12)
#define I2S_RXCR_TFS_MASK         (0x3 << 12)
#define I2S_RXCR_TFS_I2S          (0x0 << 12)
#define I2S_RXCR_TFS_PCM          (0x1 << 12)
#define I2S_RXCR_TFS_TDM1         (0x2 << 12)
#define I2S_RXCR_TFS_TDM2         (0x3 << 12)
#define I2S_VDW_MASK              (0x1f << 0)
#define I2S_VDW(x)                ((x-1) << 0)
#define I2S_TDM_FSYNC_WIDTH_MASK  (0x3ff << 16)
#define I2S_TDM_FSYNC_WIDTH(x)    ((x-1) << 16)
#define I2S_TDM_SLOTS_MASK        (0x1f << 5)
#define I2S_TDM_SLOTS(x)          ((x-1) << 5)

/* CKR bits */
#define I2S_CKR_TRCM_MASK         (0x3 << 28)
#define I2S_CKR_TRCM(x)           ((x) << 28)
#define I2S_CKR_MSS_MASK          (0x1 << 27)
#define I2S_CKR_MSS_MASTER        (0x0 << 27)
#define I2S_CKR_MSS_SLAVE         (0x1 << 27)
#define I2S_CKR_CKP_MASK          (0x1 << 26)
#define I2S_CKR_CKP_NORMAL        (0x0 << 26)
#define I2S_CKR_CKP_INVERTED      (0x1 << 26)
#define I2S_CKR_RLP_MASK          (0x1 << 25)
#define I2S_CKR_RLP_NORMAL        (0x0 << 25)
#define I2S_CKR_RLP_INVERTED      (0x1 << 25)
#define I2S_CKR_TLP_MASK          (0x1 << 24)
#define I2S_CKR_TLP_NORMAL        (0x0 << 24)
#define I2S_CKR_TLP_INVERTED      (0x1 << 24)
#define I2S_CKR_MDIV_MASK         (0xff << 16)
#define I2S_CKR_MDIV(x)           ((x) << 16)
#define I2S_CKR_RSD_MASK          (0xff << 8)
#define I2S_CKR_RSD(x)            ((x) << 8)
#define I2S_CKR_TSD_MASK          (0xff << 0)
#define I2S_CKR_TSD(x)            ((x) << 0)

/* DMACR bits */
#define I2S_DMACR_RDE_MASK        (0x1 << 24)
#define I2S_DMACR_RDE_ENABLE      (0x1 << 24)
#define I2S_DMACR_RDE_DISABLE     (0x0 << 24)
#define I2S_DMACR_RDL_MASK        (0x1f << 16)
#define I2S_DMACR_RDL(x)          ((x) << 16)
#define I2S_DMACR_TDE_MASK        (0x1 << 8)
#define I2S_DMACR_TDE_ENABLE      (0x1 << 8)
#define I2S_DMACR_TDE_DISABLE     (0x0 << 8)
#define I2S_DMACR_TDL_MASK        (0x1f << 0)
#define I2S_DMACR_TDL(x)          ((x) << 0)

/* XFER bits */
#define I2S_XFER_RXS_MASK         (0x1 << 1)
#define I2S_XFER_RXS_START        (0x1 << 1)
#define I2S_XFER_RXS_STOP         (0x0 << 1)
#define I2S_XFER_TXS_MASK         (0x1 << 0)
#define I2S_XFER_TXS_START        (0x1 << 0)
#define I2S_XFER_TXS_STOP         (0x0 << 0)

/* CLR bits */
#define I2S_CLR_RXC               (0x1 << 1)
#define I2S_CLR_TXC               (0x1 << 0)

/* VR-specific configuration */
#define VR_LOW_LATENCY_FIFO_DEPTH 8
#define VR_DEFAULT_FIFO_DEPTH     32
#define VR_DEFAULT_SAMPLE_RATE    48000
#define VR_DEFAULT_CHANNELS       2
#define VR_DEFAULT_FORMAT         SNDRV_PCM_FORMAT_S16_LE

struct orangepi_vr_i2s_dev {
    struct device *dev;
    struct regmap *regmap;
    struct clk *hclk;
    struct clk *mclk;
    struct reset_control *reset;
    
    struct snd_dmaengine_dai_dma_data capture_dma_data;
    struct snd_dmaengine_dai_dma_data playback_dma_data;
    
    bool vr_low_latency_mode;
    bool vr_beamforming_enabled;
    bool vr_spatial_audio_enabled;
    
    int playback_channels;
    int capture_channels;
    
    unsigned int mclk_rate;
    unsigned int bclk_ratio;
    unsigned int fmt;
    
    bool is_master;
    bool is_running;
};

static int orangepi_vr_i2s_runtime_suspend(struct device *dev)
{
    struct orangepi_vr_i2s_dev *i2s = dev_get_drvdata(dev);

    regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_XFER,
                      I2S_XFER_TXS_MASK | I2S_XFER_RXS_MASK,
                      I2S_XFER_TXS_STOP | I2S_XFER_RXS_STOP);

    clk_disable_unprepare(i2s->mclk);
    clk_disable_unprepare(i2s->hclk);

    return 0;
}

static int orangepi_vr_i2s_runtime_resume(struct device *dev)
{
    struct orangepi_vr_i2s_dev *i2s = dev_get_drvdata(dev);
    int ret;

    ret = clk_prepare_enable(i2s->hclk);
    if (ret) {
        dev_err(i2s->dev, "Failed to enable hclk: %d\n", ret);
        return ret;
    }

    ret = clk_prepare_enable(i2s->mclk);
    if (ret) {
        dev_err(i2s->dev, "Failed to enable mclk: %d\n", ret);
        clk_disable_unprepare(i2s->hclk);
        return ret;
    }

    return 0;
}

static int orangepi_vr_i2s_hw_params(struct snd_pcm_substream *substream,
                                   struct snd_pcm_hw_params *params,
                                   struct snd_soc_dai *dai)
{
    struct orangepi_vr_i2s_dev *i2s = snd_soc_dai_get_drvdata(dai);
    unsigned int val = 0;
    unsigned int mclk_rate, bclk_rate, div_bclk, div_lrck;
    int ret;

    /* Get sample size and configure appropriate registers */
    switch (params_format(params)) {
    case SNDRV_PCM_FORMAT_S16_LE:
        val |= I2S_VDW(16);
        break;
    case SNDRV_PCM_FORMAT_S20_3LE:
        val |= I2S_VDW(20);
        break;
    case SNDRV_PCM_FORMAT_S24_LE:
        val |= I2S_VDW(24);
        break;
    case SNDRV_PCM_FORMAT_S32_LE:
        val |= I2S_VDW(32);
        break;
    default:
        dev_err(i2s->dev, "Unsupported data format: %d\n", params_format(params));
        return -EINVAL;
    }

    /* Configure channel width */
    val |= I2S_CHANNEL_WIDTH(32);

    /* Configure mono/stereo mode */
    if (params_channels(params) == 1)
        val |= I2S_MONO;
    else
        val |= I2S_STEREO;

    /* Configure I2S format */
    val |= I2S_TXCR_TFS_I2S;

    /* Apply configuration based on stream direction */
    if (substream->stream == SNDRV_PCM_STREAM_PLAYBACK) {
        regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_TXCR,
                          I2S_BIT_WIDTH_MASK | I2S_CHANNEL_WIDTH_MASK |
                          I2S_MONO_MASK | I2S_TXCR_TFS_MASK,
                          val);
    } else {
        regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_RXCR,
                          I2S_BIT_WIDTH_MASK | I2S_CHANNEL_WIDTH_MASK |
                          I2S_MONO_MASK | I2S_RXCR_TFS_MASK,
                          val);
    }

    /* Configure clocks */
    mclk_rate = clk_get_rate(i2s->mclk);
    bclk_rate = params_rate(params) * i2s->bclk_ratio;
    div_bclk = mclk_rate / bclk_rate;
    div_lrck = bclk_rate / params_rate(params);

    regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_CKR,
                      I2S_CKR_MDIV_MASK | I2S_CKR_TSD_MASK | I2S_CKR_RSD_MASK,
                      I2S_CKR_MDIV(div_bclk) |
                      I2S_CKR_TSD(div_lrck) |
                      I2S_CKR_RSD(div_lrck));

    /* Configure DMA settings */
    if (i2s->vr_low_latency_mode) {
        /* Low latency mode: smaller FIFO threshold */
        if (substream->stream == SNDRV_PCM_STREAM_PLAYBACK) {
            regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_DMACR,
                              I2S_DMACR_TDL_MASK,
                              I2S_DMACR_TDL(VR_LOW_LATENCY_FIFO_DEPTH));
        } else {
            regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_DMACR,
                              I2S_DMACR_RDL_MASK,
                              I2S_DMACR_RDL(VR_LOW_LATENCY_FIFO_DEPTH));
        }
    } else {
        /* Normal mode: standard FIFO threshold */
        if (substream->stream == SNDRV_PCM_STREAM_PLAYBACK) {
            regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_DMACR,
                              I2S_DMACR_TDL_MASK,
                              I2S_DMACR_TDL(VR_DEFAULT_FIFO_DEPTH));
        } else {
            regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_DMACR,
                              I2S_DMACR_RDL_MASK,
                              I2S_DMACR_RDL(VR_DEFAULT_FIFO_DEPTH));
        }
    }

    return 0;
}

static int orangepi_vr_i2s_trigger(struct snd_pcm_substream *substream,
                                 int cmd, struct snd_soc_dai *dai)
{
    struct orangepi_vr_i2s_dev *i2s = snd_soc_dai_get_drvdata(dai);
    int ret = 0;

    switch (cmd) {
    case SNDRV_PCM_TRIGGER_START:
    case SNDRV_PCM_TRIGGER_RESUME:
    case SNDRV_PCM_TRIGGER_PAUSE_RELEASE:
        if (substream->stream == SNDRV_PCM_STREAM_PLAYBACK) {
            regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_DMACR,
                              I2S_DMACR_TDE_MASK, I2S_DMACR_TDE_ENABLE);
            regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_XFER,
                              I2S_XFER_TXS_MASK, I2S_XFER_TXS_START);
        } else {
            regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_DMACR,
                              I2S_DMACR_RDE_MASK, I2S_DMACR_RDE_ENABLE);
            regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_XFER,
                              I2S_XFER_RXS_MASK, I2S_XFER_RXS_START);
        }
        i2s->is_running = true;
        break;
    case SNDRV_PCM_TRIGGER_STOP:
    case SNDRV_PCM_TRIGGER_SUSPEND:
    case SNDRV_PCM_TRIGGER_PAUSE_PUSH:
        if (substream->stream == SNDRV_PCM_STREAM_PLAYBACK) {
            regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_DMACR,
                              I2S_DMACR_TDE_MASK, I2S_DMACR_TDE_DISABLE);
            regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_XFER,
                              I2S_XFER_TXS_MASK, I2S_XFER_TXS_STOP);
        } else {
            regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_DMACR,
                              I2S_DMACR_RDE_MASK, I2S_DMACR_RDE_DISABLE);
            regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_XFER,
                              I2S_XFER_RXS_MASK, I2S_XFER_RXS_STOP);
        }
        i2s->is_running = false;
        break;
    default:
        ret = -EINVAL;
        break;
    }

    return ret;
}

static int orangepi_vr_i2s_set_fmt(struct snd_soc_dai *dai, unsigned int fmt)
{
    struct orangepi_vr_i2s_dev *i2s = snd_soc_dai_get_drvdata(dai);
    unsigned int mask = 0, val = 0;

    mask = I2S_CKR_MSS_MASK | I2S_CKR_CKP_MASK | I2S_CKR_TLP_MASK | I2S_CKR_RLP_MASK;

    switch (fmt & SND_SOC_DAIFMT_MASTER_MASK) {
    case SND_SOC_DAIFMT_CBS_CFS:
        /* I2S controller is master */
        val |= I2S_CKR_MSS_MASTER;
        i2s->is_master = true;
        break;
    case SND_SOC_DAIFMT_CBM_CFM:
        /* I2S controller is slave */
        val |= I2S_CKR_MSS_SLAVE;
        i2s->is_master = false;
        break;
    default:
        dev_err(i2s->dev, "Unsupported master/slave mode: %d\n",
                fmt & SND_SOC_DAIFMT_MASTER_MASK);
        return -EINVAL;
    }

    switch (fmt & SND_SOC_DAIFMT_INV_MASK) {
    case SND_SOC_DAIFMT_NB_NF:
        /* Normal BCLK, Normal LRCK */
        val |= I2S_CKR_CKP_NORMAL | I2S_CKR_TLP_NORMAL | I2S_CKR_RLP_NORMAL;
        break;
    case SND_SOC_DAIFMT_NB_IF:
        /* Normal BCLK, Inverted LRCK */
        val |= I2S_CKR_CKP_NORMAL | I2S_CKR_TLP_INVERTED | I2S_CKR_RLP_INVERTED;
        break;
    case SND_SOC_DAIFMT_IB_NF:
        /* Inverted BCLK, Normal LRCK */
        val |= I2S_CKR_CKP_INVERTED | I2S_CKR_TLP_NORMAL | I2S_CKR_RLP_NORMAL;
        break;
    case SND_SOC_DAIFMT_IB_IF:
        /* Inverted BCLK, Inverted LRCK */
        val |= I2S_CKR_CKP_INVERTED | I2S_CKR_TLP_INVERTED | I2S_CKR_RLP_INVERTED;
        break;
    default:
        dev_err(i2s->dev, "Unsupported clock inversion: %d\n",
                fmt & SND_SOC_DAIFMT_INV_MASK);
        return -EINVAL;
    }

    regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_CKR, mask, val);

    return 0;
}

static int orangepi_vr_i2s_set_sysclk(struct snd_soc_dai *dai, int clk_id,
                                    unsigned int freq, int dir)
{
    struct orangepi_vr_i2s_dev *i2s = snd_soc_dai_get_drvdata(dai);
    int ret = 0;

    if (!freq) {
        dev_err(i2s->dev, "Invalid frequency: %u\n", freq);
        return -EINVAL;
    }

    i2s->mclk_rate = freq;
    ret = clk_set_rate(i2s->mclk, freq);
    if (ret)
        dev_err(i2s->dev, "Failed to set mclk rate to %u: %d\n", freq, ret);

    return ret;
}

static int orangepi_vr_i2s_dai_probe(struct snd_soc_dai *dai)
{
    struct orangepi_vr_i2s_dev *i2s = snd_soc_dai_get_drvdata(dai);

    /* Set default formats and capabilities */
    snd_soc_dai_init_dma_data(dai, &i2s->playback_dma_data, &i2s->capture_dma_data);

    return 0;
}

static const struct snd_soc_dai_ops orangepi_vr_i2s_dai_ops = {
    .hw_params = orangepi_vr_i2s_hw_params,
    .trigger = orangepi_vr_i2s_trigger,
    .set_fmt = orangepi_vr_i2s_set_fmt,
    .set_sysclk = orangepi_vr_i2s_set_sysclk,
};

static const struct snd_soc_component_driver orangepi_vr_i2s_component = {
    .name = "orangepi-vr-i2s",
};

static struct snd_soc_dai_driver orangepi_vr_i2s_dai = {
    .probe = orangepi_vr_i2s_dai_probe,
    .playback = {
        .stream_name = "Playback",
        .channels_min = 1,
        .channels_max = 8,
        .rates = SNDRV_PCM_RATE_8000_192000,
        .formats = SNDRV_PCM_FMTBIT_S16_LE |
                  SNDRV_PCM_FMTBIT_S20_3LE |
                  SNDRV_PCM_FMTBIT_S24_LE |
                  SNDRV_PCM_FMTBIT_S32_LE,
    },
    .capture = {
        .stream_name = "Capture",
        .channels_min = 1,
        .channels_max = 8,
        .rates = SNDRV_PCM_RATE_8000_192000,
        .formats = SNDRV_PCM_FMTBIT_S16_LE |
                  SNDRV_PCM_FMTBIT_S20_3LE |
                  SNDRV_PCM_FMTBIT_S24_LE |
                  SNDRV_PCM_FMTBIT_S32_LE,
    },
    .ops = &orangepi_vr_i2s_dai_ops,
    .symmetric_rates = 1,
};

static const struct regmap_config orangepi_vr_i2s_regmap_config = {
    .reg_bits = 32,
    .reg_stride = 4,
    .val_bits = 32,
    .max_register = ROCKCHIP_I2S_CLKDIV,
    .cache_type = REGCACHE_FLAT,
};

static int orangepi_vr_i2s_probe(struct platform_device *pdev)
{
    struct device_node *node = pdev->dev.of_node;
    struct orangepi_vr_i2s_dev *i2s;
    struct resource *res;
    void __iomem *regs;
    int ret;

    i2s = devm_kzalloc(&pdev->dev, sizeof(*i2s), GFP_KERNEL);
    if (!i2s)
        return -ENOMEM;

    i2s->dev = &pdev->dev;

    /* Get memory resource */
    res = platform_get_resource(pdev, IORESOURCE_MEM, 0);
    regs = devm_ioremap_resource(&pdev->dev, res);
    if (IS_ERR(regs))
        return PTR_ERR(regs);

    /* Create regmap */
    i2s->regmap = devm_regmap_init_mmio(&pdev->dev, regs,
                                       &orangepi_vr_i2s_regmap_config);
    if (IS_ERR(i2s->regmap)) {
        dev_err(&pdev->dev, "Failed to initialize regmap: %ld\n",
                PTR_ERR(i2s->regmap));
        return PTR_ERR(i2s->regmap);
    }

    /* Get clocks */
    i2s->hclk = devm_clk_get(&pdev->dev, "i2s_hclk");
    if (IS_ERR(i2s->hclk)) {
        dev_err(&pdev->dev, "Failed to get hclk: %ld\n", PTR_ERR(i2s->hclk));
        return PTR_ERR(i2s->hclk);
    }

    i2s->mclk = devm_clk_get(&pdev->dev, "i2s_clk");
    if (IS_ERR(i2s->mclk)) {
        dev_err(&pdev->dev, "Failed to get mclk: %ld\n", PTR_ERR(i2s->mclk));
        return PTR_ERR(i2s->mclk);
    }

    /* Get reset control */
    i2s->reset = devm_reset_control_get(&pdev->dev, "reset");
    if (IS_ERR(i2s->reset)) {
        if (PTR_ERR(i2s->reset) != -EPROBE_DEFER)
            dev_err(&pdev->dev, "Failed to get reset control: %ld\n",
                    PTR_ERR(i2s->reset));
        return PTR_ERR(i2s->reset);
    }

    /* Parse VR-specific properties */
    i2s->vr_low_latency_mode = of_property_read_bool(node, "vr,low-latency-mode");
    i2s->vr_beamforming_enabled = of_property_read_bool(node, "vr,beamforming-enabled");
    i2s->vr_spatial_audio_enabled = of_property_read_bool(node, "vr,spatial-audio-enabled");

    /* Get channel configuration */
    of_property_read_u32(node, "rockchip,playback-channels", &i2s->playback_channels);
    of_property_read_u32(node, "rockchip,capture-channels", &i2s->capture_channels);

    /* Set default values if not specified */
    if (!i2s->playback_channels)
        i2s->playback_channels = VR_DEFAULT_CHANNELS;
    if (!i2s->capture_channels)
        i2s->capture_channels = 4; /* Default to 4-mic array */

    /* Update dai driver with channel configuration */
    orangepi_vr_i2s_dai.playback.channels_max = i2s->playback_channels;
    orangepi_vr_i2s_dai.capture.channels_max = i2s->capture_channels;

    /* Configure DMA parameters */
    i2s->playback_dma_data.addr = res->start + ROCKCHIP_I2S_TXDR;
    i2s->playback_dma_data.addr_width = DMA_SLAVE_BUSWIDTH_4_BYTES;
    i2s->playback_dma_data.maxburst = 8;

    i2s->capture_dma_data.addr = res->start + ROCKCHIP_I2S_RXDR;
    i2s->capture_dma_data.addr_width = DMA_SLAVE_BUSWIDTH_4_BYTES;
    i2s->capture_dma_data.maxburst = 8;

    /* Set default BCLK ratio */
    i2s->bclk_ratio = 64;
    of_property_read_u32(node, "rockchip,bclk-fs", &i2s->bclk_ratio);

    /* Initialize I2S controller */
    platform_set_drvdata(pdev, i2s);
    pm_runtime_enable(&pdev->dev);

    /* Reset I2S controller */
    ret = reset_control_assert(i2s->reset);
    if (ret) {
        dev_err(&pdev->dev, "Failed to assert reset: %d\n", ret);
        goto err_pm_disable;
    }

    ret = reset_control_deassert(i2s->reset);
    if (ret) {
        dev_err(&pdev->dev, "Failed to deassert reset: %d\n", ret);
        goto err_pm_disable;
    }

    /* Register DAI */
    ret = devm_snd_soc_register_component(&pdev->dev, &orangepi_vr_i2s_component,
                                         &orangepi_vr_i2s_dai, 1);
    if (ret) {
        dev_err(&pdev->dev, "Failed to register DAI: %d\n", ret);
        goto err_pm_disable;
    }

    /* Register PCM */
    ret = devm_snd_dmaengine_pcm_register(&pdev->dev, NULL, 0);
    if (ret) {
        dev_err(&pdev->dev, "Failed to register PCM: %d\n", ret);
        goto err_pm_disable;
    }

    dev_info(&pdev->dev, "Orange Pi CM5 VR I2S controller initialized\n");
    if (i2s->vr_low_latency_mode)
        dev_info(&pdev->dev, "VR low-latency mode enabled\n");
    if (i2s->vr_beamforming_enabled)
        dev_info(&pdev->dev, "VR beamforming enabled\n");
    if (i2s->vr_spatial_audio_enabled)
        dev_info(&pdev->dev, "VR spatial audio enabled\n");

    return 0;

err_pm_disable:
    pm_runtime_disable(&pdev->dev);
    return ret;
}

static int orangepi_vr_i2s_remove(struct platform_device *pdev)
{
    pm_runtime_disable(&pdev->dev);
    return 0;
}

#ifdef CONFIG_PM_SLEEP
static int orangepi_vr_i2s_suspend(struct device *dev)
{
    struct orangepi_vr_i2s_dev *i2s = dev_get_drvdata(dev);

    regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_XFER,
                      I2S_XFER_TXS_MASK | I2S_XFER_RXS_MASK,
                      I2S_XFER_TXS_STOP | I2S_XFER_RXS_STOP);

    return 0;
}

static int orangepi_vr_i2s_resume(struct device *dev)
{
    struct orangepi_vr_i2s_dev *i2s = dev_get_drvdata(dev);

    if (i2s->is_running) {
        regmap_update_bits(i2s->regmap, ROCKCHIP_I2S_XFER,
                          I2S_XFER_TXS_MASK | I2S_XFER_RXS_MASK,
                          I2S_XFER_TXS_START | I2S_XFER_RXS_START);
    }

    return 0;
}
#endif

static const struct dev_pm_ops orangepi_vr_i2s_pm_ops = {
    SET_RUNTIME_PM_OPS(orangepi_vr_i2s_runtime_suspend, orangepi_vr_i2s_runtime_resume, NULL)
    SET_SYSTEM_SLEEP_PM_OPS(orangepi_vr_i2s_suspend, orangepi_vr_i2s_resume)
};

static const struct of_device_id orangepi_vr_i2s_match[] = {
    { .compatible = "orangepi,vr-i2s", },
    {},
};
MODULE_DEVICE_TABLE(of, orangepi_vr_i2s_match);

static struct platform_driver orangepi_vr_i2s_driver = {
    .probe = orangepi_vr_i2s_probe,
    .remove = orangepi_vr_i2s_remove,
    .driver = {
        .name = "orangepi-vr-i2s",
        .of_match_table = orangepi_vr_i2s_match,
        .pm = &orangepi_vr_i2s_pm_ops,
    },
};
module_platform_driver(orangepi_vr_i2s_driver);

MODULE_DESCRIPTION("Orange Pi CM5 VR Headset I2S Controller Driver");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
