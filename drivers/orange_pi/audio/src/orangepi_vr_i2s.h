/*
 * Orange Pi CM5 VR Headset I2S Controller Driver Header
 *
 * Copyright (c) 2025 VR Headset Project
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License version 2 as
 * published by the Free Software Foundation.
 */

#ifndef __ORANGEPI_VR_I2S_H__
#define __ORANGEPI_VR_I2S_H__

#include <linux/clk.h>
#include <linux/device.h>
#include <linux/regmap.h>
#include <sound/soc.h>

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

#endif /* __ORANGEPI_VR_I2S_H__ */
