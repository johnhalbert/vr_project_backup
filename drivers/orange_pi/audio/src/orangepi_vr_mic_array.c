/*
 * Orange Pi CM5 VR Headset Microphone Array Driver
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
#include <sound/soc.h>
#include <sound/pcm_params.h>
#include <sound/tlv.h>

#include "orangepi_vr_i2s.h"
#include "orangepi_vr_mic_array.h"

/* Microphone array registers */
#define MIC_CTRL               0x00
#define MIC_GAIN               0x01
#define MIC_STATUS             0x02
#define MIC_POWER              0x03
#define MIC_BEAM               0x04
#define MIC_NOISE              0x05
#define MIC_AEC                0x06
#define MIC_CONFIG             0x07

/* MIC_CTRL bits */
#define MIC_CTRL_ENABLE        (1 << 0)
#define MIC_CTRL_MUTE          (1 << 1)
#define MIC_CTRL_BEAM          (1 << 2)
#define MIC_CTRL_NOISE         (1 << 3)
#define MIC_CTRL_AEC           (1 << 4)
#define MIC_CTRL_LOWLAT        (1 << 5)
#define MIC_CTRL_SYNC          (1 << 6)

/* MIC_POWER bits */
#define MIC_POWER_ON           (1 << 0)
#define MIC_POWER_STANDBY      (1 << 1)
#define MIC_POWER_DOWN         (1 << 2)

/* VR-specific configuration */
#define VR_MIC_DEFAULT_GAIN    80  /* 0-100 scale */
#define VR_MIC_MAX_GAIN        100
#define VR_MIC_MIN_GAIN        0
#define VR_MIC_ARRAY_SIZE      4   /* 4-mic array */

struct orangepi_vr_mic_array {
    struct device *dev;
    struct regmap *regmap;
    struct orangepi_vr_i2s_dev *i2s;
    
    bool vr_beamforming_enabled;
    bool vr_low_latency_mode;
    
    unsigned int gain;
    bool muted;
    bool enabled;
    bool beamforming_enabled;
    bool noise_suppression_enabled;
    bool aec_enabled;
    
    /* Microphone positions in degrees */
    int mic_positions[VR_MIC_ARRAY_SIZE];
    
    /* ALSA controls */
    struct snd_kcontrol_new *controls;
    int num_controls;
};

/* ALSA mixer controls */
static const DECLARE_TLV_DB_SCALE(mic_gain_tlv, -9000, 100, 0);

static int orangepi_vr_mic_get_gain(struct snd_kcontrol *kcontrol,
                                  struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = mic->gain;

    return 0;
}

static int orangepi_vr_mic_set_gain(struct snd_kcontrol *kcontrol,
                                  struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    if (val > VR_MIC_MAX_GAIN)
        return -EINVAL;

    mic->gain = val;
    
    /* Update hardware gain */
    regmap_write(mic->regmap, MIC_GAIN, mic->gain);

    return 0;
}

static int orangepi_vr_mic_get_mute(struct snd_kcontrol *kcontrol,
                                  struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = mic->muted ? 1 : 0;

    return 0;
}

static int orangepi_vr_mic_set_mute(struct snd_kcontrol *kcontrol,
                                  struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    mic->muted = val ? true : false;
    
    /* Update hardware mute state */
    regmap_update_bits(mic->regmap, MIC_CTRL, MIC_CTRL_MUTE,
                      mic->muted ? MIC_CTRL_MUTE : 0);

    return 0;
}

static int orangepi_vr_mic_get_beamforming(struct snd_kcontrol *kcontrol,
                                         struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = mic->beamforming_enabled ? 1 : 0;

    return 0;
}

static int orangepi_vr_mic_set_beamforming(struct snd_kcontrol *kcontrol,
                                         struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    mic->beamforming_enabled = val ? true : false;
    
    /* Update beamforming state */
    regmap_update_bits(mic->regmap, MIC_CTRL, MIC_CTRL_BEAM,
                      mic->beamforming_enabled ? MIC_CTRL_BEAM : 0);

    return 0;
}

static int orangepi_vr_mic_get_noise_suppression(struct snd_kcontrol *kcontrol,
                                               struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = mic->noise_suppression_enabled ? 1 : 0;

    return 0;
}

static int orangepi_vr_mic_set_noise_suppression(struct snd_kcontrol *kcontrol,
                                               struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    mic->noise_suppression_enabled = val ? true : false;
    
    /* Update noise suppression state */
    regmap_update_bits(mic->regmap, MIC_CTRL, MIC_CTRL_NOISE,
                      mic->noise_suppression_enabled ? MIC_CTRL_NOISE : 0);

    return 0;
}

static int orangepi_vr_mic_get_aec(struct snd_kcontrol *kcontrol,
                                 struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = mic->aec_enabled ? 1 : 0;

    return 0;
}

static int orangepi_vr_mic_set_aec(struct snd_kcontrol *kcontrol,
                                 struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    mic->aec_enabled = val ? true : false;
    
    /* Update AEC state */
    regmap_update_bits(mic->regmap, MIC_CTRL, MIC_CTRL_AEC,
                      mic->aec_enabled ? MIC_CTRL_AEC : 0);

    return 0;
}

static const struct snd_kcontrol_new orangepi_vr_mic_array_controls[] = {
    SOC_SINGLE_EXT_TLV("Microphone Capture Volume", MIC_GAIN,
                      0, VR_MIC_MAX_GAIN, 0,
                      orangepi_vr_mic_get_gain,
                      orangepi_vr_mic_set_gain,
                      mic_gain_tlv),
    SOC_SINGLE_BOOL_EXT("Microphone Capture Switch", 0,
                       orangepi_vr_mic_get_mute,
                       orangepi_vr_mic_set_mute),
    SOC_SINGLE_BOOL_EXT("Beamforming Enable", 0,
                       orangepi_vr_mic_get_beamforming,
                       orangepi_vr_mic_set_beamforming),
    SOC_SINGLE_BOOL_EXT("Noise Suppression Enable", 0,
                       orangepi_vr_mic_get_noise_suppression,
                       orangepi_vr_mic_set_noise_suppression),
    SOC_SINGLE_BOOL_EXT("Acoustic Echo Cancellation Enable", 0,
                       orangepi_vr_mic_get_aec,
                       orangepi_vr_mic_set_aec),
};

static int orangepi_vr_mic_array_hw_params(struct snd_pcm_substream *substream,
                                         struct snd_pcm_hw_params *params,
                                         struct snd_soc_dai *dai)
{
    struct snd_soc_component *component = dai->component;
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);
    unsigned int val = 0;

    /* Only configure for capture */
    if (substream->stream != SNDRV_PCM_STREAM_CAPTURE)
        return 0;

    /* Configure low latency mode if enabled */
    if (mic->vr_low_latency_mode) {
        regmap_update_bits(mic->regmap, MIC_CTRL, MIC_CTRL_LOWLAT, MIC_CTRL_LOWLAT);
    } else {
        regmap_update_bits(mic->regmap, MIC_CTRL, MIC_CTRL_LOWLAT, 0);
    }

    /* Configure synchronized capture for all microphones */
    regmap_update_bits(mic->regmap, MIC_CTRL, MIC_CTRL_SYNC, MIC_CTRL_SYNC);

    return 0;
}

static int orangepi_vr_mic_array_set_fmt(struct snd_soc_dai *dai, unsigned int fmt)
{
    /* No format configuration needed for microphone array */
    return 0;
}

static int orangepi_vr_mic_array_digital_mute(struct snd_soc_dai *dai, int mute, int direction)
{
    struct snd_soc_component *component = dai->component;
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);

    if (direction == SNDRV_PCM_STREAM_CAPTURE) {
        mic->muted = mute ? true : false;
        regmap_update_bits(mic->regmap, MIC_CTRL, MIC_CTRL_MUTE,
                          mic->muted ? MIC_CTRL_MUTE : 0);
    }

    return 0;
}

static int orangepi_vr_mic_array_startup(struct snd_pcm_substream *substream,
                                       struct snd_soc_dai *dai)
{
    struct snd_soc_component *component = dai->component;
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);

    if (substream->stream == SNDRV_PCM_STREAM_CAPTURE) {
        /* Power up microphone array */
        regmap_write(mic->regmap, MIC_POWER, MIC_POWER_ON);
        
        /* Enable microphone array */
        mic->enabled = true;
        regmap_update_bits(mic->regmap, MIC_CTRL, MIC_CTRL_ENABLE, MIC_CTRL_ENABLE);
    }

    return 0;
}

static void orangepi_vr_mic_array_shutdown(struct snd_pcm_substream *substream,
                                         struct snd_soc_dai *dai)
{
    struct snd_soc_component *component = dai->component;
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);

    if (substream->stream == SNDRV_PCM_STREAM_CAPTURE) {
        /* Disable microphone array */
        mic->enabled = false;
        regmap_update_bits(mic->regmap, MIC_CTRL, MIC_CTRL_ENABLE, 0);
        
        /* Power down microphone array */
        regmap_write(mic->regmap, MIC_POWER, MIC_POWER_DOWN);
    }
}

static const struct snd_soc_dai_ops orangepi_vr_mic_array_dai_ops = {
    .hw_params = orangepi_vr_mic_array_hw_params,
    .set_fmt = orangepi_vr_mic_array_set_fmt,
    .digital_mute = orangepi_vr_mic_array_digital_mute,
    .startup = orangepi_vr_mic_array_startup,
    .shutdown = orangepi_vr_mic_array_shutdown,
};

static struct snd_soc_dai_driver orangepi_vr_mic_array_dai = {
    .name = "orangepi-vr-mic-array",
    .capture = {
        .stream_name = "Microphone Capture",
        .channels_min = 1,
        .channels_max = VR_MIC_ARRAY_SIZE,
        .rates = SNDRV_PCM_RATE_8000_192000,
        .formats = SNDRV_PCM_FMTBIT_S16_LE |
                  SNDRV_PCM_FMTBIT_S20_3LE |
                  SNDRV_PCM_FMTBIT_S24_LE |
                  SNDRV_PCM_FMTBIT_S32_LE,
    },
    .ops = &orangepi_vr_mic_array_dai_ops,
};

static int orangepi_vr_mic_array_probe(struct snd_soc_component *component)
{
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);
    int ret;

    /* Initialize microphone array */
    regmap_write(mic->regmap, MIC_POWER, MIC_POWER_STANDBY);
    regmap_write(mic->regmap, MIC_GAIN, mic->gain);
    regmap_write(mic->regmap, MIC_CTRL, 0);

    /* Configure beamforming if enabled */
    if (mic->vr_beamforming_enabled) {
        mic->beamforming_enabled = true;
        regmap_update_bits(mic->regmap, MIC_CTRL, MIC_CTRL_BEAM, MIC_CTRL_BEAM);
        
        /* Configure beamforming parameters */
        regmap_write(mic->regmap, MIC_BEAM, 0x80); /* Default beamforming configuration */
        
        /* Configure microphone positions */
        regmap_write(mic->regmap, MIC_CONFIG, 
                    (mic->mic_positions[0] & 0xFF) |
                    ((mic->mic_positions[1] & 0xFF) << 8) |
                    ((mic->mic_positions[2] & 0xFF) << 16) |
                    ((mic->mic_positions[3] & 0xFF) << 24));
    }

    /* Configure noise suppression */
    mic->noise_suppression_enabled = true;
    regmap_update_bits(mic->regmap, MIC_CTRL, MIC_CTRL_NOISE, MIC_CTRL_NOISE);
    regmap_write(mic->regmap, MIC_NOISE, 0x80); /* Default noise suppression configuration */

    /* Configure acoustic echo cancellation */
    mic->aec_enabled = true;
    regmap_update_bits(mic->regmap, MIC_CTRL, MIC_CTRL_AEC, MIC_CTRL_AEC);
    regmap_write(mic->regmap, MIC_AEC, 0x80); /* Default AEC configuration */

    /* Configure low latency mode if enabled */
    if (mic->vr_low_latency_mode) {
        regmap_update_bits(mic->regmap, MIC_CTRL, MIC_CTRL_LOWLAT, MIC_CTRL_LOWLAT);
    }

    return 0;
}

static void orangepi_vr_mic_array_remove(struct snd_soc_component *component)
{
    struct orangepi_vr_mic_array *mic = snd_soc_component_get_drvdata(component);

    /* Power down microphone array */
    regmap_write(mic->regmap, MIC_POWER, MIC_POWER_DOWN);
}

static const struct snd_soc_component_driver orangepi_vr_mic_array_component = {
    .probe = orangepi_vr_mic_array_probe,
    .remove = orangepi_vr_mic_array_remove,
    .controls = orangepi_vr_mic_array_controls,
    .num_controls = ARRAY_SIZE(orangepi_vr_mic_array_controls),
};

static const struct regmap_config orangepi_vr_mic_array_regmap_config = {
    .reg_bits = 8,
    .val_bits = 8,
    .max_register = MIC_CONFIG,
    .cache_type = REGCACHE_RBTREE,
};

int orangepi_vr_mic_array_init(struct device *dev, struct orangepi_vr_i2s_dev *i2s)
{
    struct orangepi_vr_mic_array *mic;
    struct device_node *node = dev->of_node;
    void __iomem *regs;
    int ret, i;
    u32 positions[VR_MIC_ARRAY_SIZE];

    mic = devm_kzalloc(dev, sizeof(*mic), GFP_KERNEL);
    if (!mic)
        return -ENOMEM;

    mic->dev = dev;
    mic->i2s = i2s;

    /* Get memory-mapped registers */
    regs = devm_platform_ioremap_resource(to_platform_device(dev), 2);
    if (IS_ERR(regs))
        return PTR_ERR(regs);

    /* Create regmap */
    mic->regmap = devm_regmap_init_mmio(dev, regs, &orangepi_vr_mic_array_regmap_config);
    if (IS_ERR(mic->regmap)) {
        dev_err(dev, "Failed to initialize regmap: %ld\n", PTR_ERR(mic->regmap));
        return PTR_ERR(mic->regmap);
    }

    /* Parse VR-specific properties */
    mic->vr_beamforming_enabled = of_property_read_bool(node, "vr,beamforming-enabled");
    mic->vr_low_latency_mode = of_property_read_bool(node, "vr,low-latency-mode");

    /* Get microphone positions */
    ret = of_property_read_u32_array(node, "vr,mic-positions", positions, VR_MIC_ARRAY_SIZE);
    if (ret) {
        /* Default positions if not specified (0, 90, 180, 270 degrees) */
        mic->mic_positions[0] = 0;
        mic->mic_positions[1] = 90;
        mic->mic_positions[2] = 180;
        mic->mic_positions[3] = 270;
    } else {
        for (i = 0; i < VR_MIC_ARRAY_SIZE; i++)
            mic->mic_positions[i] = positions[i];
    }

    /* Set default values */
    mic->gain = VR_MIC_DEFAULT_GAIN;
    mic->muted = false;
    mic->enabled = false;

    /* Register component */
    dev_set_drvdata(dev, mic);
    ret = devm_snd_soc_register_component(dev, &orangepi_vr_mic_array_component,
                                         &orangepi_vr_mic_array_dai, 1);
    if (ret) {
        dev_err(dev, "Failed to register component: %d\n", ret);
        return ret;
    }

    dev_info(dev, "Orange Pi CM5 VR Microphone Array driver initialized\n");
    if (mic->vr_beamforming_enabled)
        dev_info(dev, "VR beamforming enabled\n");
    if (mic->vr_low_latency_mode)
        dev_info(dev, "VR low-latency mode enabled\n");

    return 0;
}
EXPORT_SYMBOL_GPL(orangepi_vr_mic_array_init);

MODULE_DESCRIPTION("Orange Pi CM5 VR Headset Microphone Array Driver");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
