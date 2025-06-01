/*
 * Orange Pi CM5 VR Headset Headphone Output Driver
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
#include "orangepi_vr_headphone.h"

/* Headphone amplifier registers */
#define HP_CTRL                0x00
#define HP_VOL                 0x01
#define HP_STATUS              0x02
#define HP_POWER               0x03
#define HP_FILTER              0x04
#define HP_SPATIAL             0x05
#define HP_EQ                  0x06

/* HP_CTRL bits */
#define HP_CTRL_ENABLE         (1 << 0)
#define HP_CTRL_MUTE           (1 << 1)
#define HP_CTRL_DEEMPH         (1 << 2)
#define HP_CTRL_SPATIAL        (1 << 3)
#define HP_CTRL_EQ             (1 << 4)
#define HP_CTRL_LOWLAT         (1 << 5)

/* HP_POWER bits */
#define HP_POWER_ON            (1 << 0)
#define HP_POWER_STANDBY       (1 << 1)
#define HP_POWER_DOWN          (1 << 2)

/* VR-specific configuration */
#define VR_HP_DEFAULT_VOLUME   80  /* 0-100 scale */
#define VR_HP_MAX_VOLUME       100
#define VR_HP_MIN_VOLUME       0

struct orangepi_vr_headphone {
    struct device *dev;
    struct regmap *regmap;
    struct orangepi_vr_i2s_dev *i2s;
    
    bool vr_spatial_audio_enabled;
    bool vr_low_latency_mode;
    
    unsigned int volume;
    bool muted;
    bool enabled;
    bool eq_enabled;
    bool spatial_enabled;
    
    /* ALSA controls */
    struct snd_kcontrol_new *controls;
    int num_controls;
};

/* ALSA mixer controls */
static const DECLARE_TLV_DB_SCALE(hp_volume_tlv, -9000, 100, 0);

static int orangepi_vr_headphone_get_volume(struct snd_kcontrol *kcontrol,
                                          struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_headphone *hp = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = hp->volume;
    ucontrol->value.integer.value[1] = hp->volume;

    return 0;
}

static int orangepi_vr_headphone_set_volume(struct snd_kcontrol *kcontrol,
                                          struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_headphone *hp = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    if (val > VR_HP_MAX_VOLUME)
        return -EINVAL;

    hp->volume = val;
    
    /* Update hardware volume */
    regmap_write(hp->regmap, HP_VOL, hp->volume);

    return 0;
}

static int orangepi_vr_headphone_get_mute(struct snd_kcontrol *kcontrol,
                                        struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_headphone *hp = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = hp->muted ? 1 : 0;

    return 0;
}

static int orangepi_vr_headphone_set_mute(struct snd_kcontrol *kcontrol,
                                        struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_headphone *hp = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    hp->muted = val ? true : false;
    
    /* Update hardware mute state */
    regmap_update_bits(hp->regmap, HP_CTRL, HP_CTRL_MUTE,
                      hp->muted ? HP_CTRL_MUTE : 0);

    return 0;
}

static int orangepi_vr_headphone_get_spatial(struct snd_kcontrol *kcontrol,
                                           struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_headphone *hp = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = hp->spatial_enabled ? 1 : 0;

    return 0;
}

static int orangepi_vr_headphone_set_spatial(struct snd_kcontrol *kcontrol,
                                           struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_headphone *hp = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    hp->spatial_enabled = val ? true : false;
    
    /* Update spatial audio state */
    regmap_update_bits(hp->regmap, HP_CTRL, HP_CTRL_SPATIAL,
                      hp->spatial_enabled ? HP_CTRL_SPATIAL : 0);

    return 0;
}

static int orangepi_vr_headphone_get_eq(struct snd_kcontrol *kcontrol,
                                      struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_headphone *hp = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = hp->eq_enabled ? 1 : 0;

    return 0;
}

static int orangepi_vr_headphone_set_eq(struct snd_kcontrol *kcontrol,
                                      struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_headphone *hp = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    hp->eq_enabled = val ? true : false;
    
    /* Update EQ state */
    regmap_update_bits(hp->regmap, HP_CTRL, HP_CTRL_EQ,
                      hp->eq_enabled ? HP_CTRL_EQ : 0);

    return 0;
}

static const struct snd_kcontrol_new orangepi_vr_headphone_controls[] = {
    SOC_DOUBLE_R_TLV("Headphone Playback Volume", HP_VOL, HP_VOL,
                    0, VR_HP_MAX_VOLUME, 0, hp_volume_tlv),
    SOC_SINGLE_BOOL_EXT("Headphone Playback Switch", 0,
                       orangepi_vr_headphone_get_mute,
                       orangepi_vr_headphone_set_mute),
    SOC_SINGLE_BOOL_EXT("Spatial Audio Enable", 0,
                       orangepi_vr_headphone_get_spatial,
                       orangepi_vr_headphone_set_spatial),
    SOC_SINGLE_BOOL_EXT("Equalizer Enable", 0,
                       orangepi_vr_headphone_get_eq,
                       orangepi_vr_headphone_set_eq),
};

static int orangepi_vr_headphone_hw_params(struct snd_pcm_substream *substream,
                                         struct snd_pcm_hw_params *params,
                                         struct snd_soc_dai *dai)
{
    struct snd_soc_component *component = dai->component;
    struct orangepi_vr_headphone *hp = snd_soc_component_get_drvdata(component);
    unsigned int val = 0;

    /* Only configure for playback */
    if (substream->stream != SNDRV_PCM_STREAM_PLAYBACK)
        return 0;

    /* Configure low latency mode if enabled */
    if (hp->vr_low_latency_mode) {
        regmap_update_bits(hp->regmap, HP_CTRL, HP_CTRL_LOWLAT, HP_CTRL_LOWLAT);
    } else {
        regmap_update_bits(hp->regmap, HP_CTRL, HP_CTRL_LOWLAT, 0);
    }

    return 0;
}

static int orangepi_vr_headphone_set_fmt(struct snd_soc_dai *dai, unsigned int fmt)
{
    /* No format configuration needed for headphone output */
    return 0;
}

static int orangepi_vr_headphone_digital_mute(struct snd_soc_dai *dai, int mute, int direction)
{
    struct snd_soc_component *component = dai->component;
    struct orangepi_vr_headphone *hp = snd_soc_component_get_drvdata(component);

    if (direction == SNDRV_PCM_STREAM_PLAYBACK) {
        hp->muted = mute ? true : false;
        regmap_update_bits(hp->regmap, HP_CTRL, HP_CTRL_MUTE,
                          hp->muted ? HP_CTRL_MUTE : 0);
    }

    return 0;
}

static int orangepi_vr_headphone_startup(struct snd_pcm_substream *substream,
                                       struct snd_soc_dai *dai)
{
    struct snd_soc_component *component = dai->component;
    struct orangepi_vr_headphone *hp = snd_soc_component_get_drvdata(component);

    if (substream->stream == SNDRV_PCM_STREAM_PLAYBACK) {
        /* Power up headphone amplifier */
        regmap_write(hp->regmap, HP_POWER, HP_POWER_ON);
        
        /* Enable headphone output */
        hp->enabled = true;
        regmap_update_bits(hp->regmap, HP_CTRL, HP_CTRL_ENABLE, HP_CTRL_ENABLE);
    }

    return 0;
}

static void orangepi_vr_headphone_shutdown(struct snd_pcm_substream *substream,
                                         struct snd_soc_dai *dai)
{
    struct snd_soc_component *component = dai->component;
    struct orangepi_vr_headphone *hp = snd_soc_component_get_drvdata(component);

    if (substream->stream == SNDRV_PCM_STREAM_PLAYBACK) {
        /* Disable headphone output */
        hp->enabled = false;
        regmap_update_bits(hp->regmap, HP_CTRL, HP_CTRL_ENABLE, 0);
        
        /* Power down headphone amplifier */
        regmap_write(hp->regmap, HP_POWER, HP_POWER_DOWN);
    }
}

static const struct snd_soc_dai_ops orangepi_vr_headphone_dai_ops = {
    .hw_params = orangepi_vr_headphone_hw_params,
    .set_fmt = orangepi_vr_headphone_set_fmt,
    .digital_mute = orangepi_vr_headphone_digital_mute,
    .startup = orangepi_vr_headphone_startup,
    .shutdown = orangepi_vr_headphone_shutdown,
};

static struct snd_soc_dai_driver orangepi_vr_headphone_dai = {
    .name = "orangepi-vr-headphone",
    .playback = {
        .stream_name = "Headphone Playback",
        .channels_min = 1,
        .channels_max = 2,
        .rates = SNDRV_PCM_RATE_8000_192000,
        .formats = SNDRV_PCM_FMTBIT_S16_LE |
                  SNDRV_PCM_FMTBIT_S20_3LE |
                  SNDRV_PCM_FMTBIT_S24_LE |
                  SNDRV_PCM_FMTBIT_S32_LE,
    },
    .ops = &orangepi_vr_headphone_dai_ops,
};

static int orangepi_vr_headphone_probe(struct snd_soc_component *component)
{
    struct orangepi_vr_headphone *hp = snd_soc_component_get_drvdata(component);
    int ret;

    /* Initialize headphone amplifier */
    regmap_write(hp->regmap, HP_POWER, HP_POWER_STANDBY);
    regmap_write(hp->regmap, HP_VOL, hp->volume);
    regmap_write(hp->regmap, HP_CTRL, 0);

    /* Configure spatial audio if enabled */
    if (hp->vr_spatial_audio_enabled) {
        hp->spatial_enabled = true;
        regmap_update_bits(hp->regmap, HP_CTRL, HP_CTRL_SPATIAL, HP_CTRL_SPATIAL);
        
        /* Configure spatial audio parameters */
        regmap_write(hp->regmap, HP_SPATIAL, 0x80); /* Default spatial configuration */
    }

    /* Configure low latency mode if enabled */
    if (hp->vr_low_latency_mode) {
        regmap_update_bits(hp->regmap, HP_CTRL, HP_CTRL_LOWLAT, HP_CTRL_LOWLAT);
    }

    return 0;
}

static void orangepi_vr_headphone_remove(struct snd_soc_component *component)
{
    struct orangepi_vr_headphone *hp = snd_soc_component_get_drvdata(component);

    /* Power down headphone amplifier */
    regmap_write(hp->regmap, HP_POWER, HP_POWER_DOWN);
}

static const struct snd_soc_component_driver orangepi_vr_headphone_component = {
    .probe = orangepi_vr_headphone_probe,
    .remove = orangepi_vr_headphone_remove,
    .controls = orangepi_vr_headphone_controls,
    .num_controls = ARRAY_SIZE(orangepi_vr_headphone_controls),
};

static const struct regmap_config orangepi_vr_headphone_regmap_config = {
    .reg_bits = 8,
    .val_bits = 8,
    .max_register = HP_EQ,
    .cache_type = REGCACHE_RBTREE,
};

int orangepi_vr_headphone_init(struct device *dev, struct orangepi_vr_i2s_dev *i2s)
{
    struct orangepi_vr_headphone *hp;
    struct device_node *node = dev->of_node;
    void __iomem *regs;
    int ret;

    hp = devm_kzalloc(dev, sizeof(*hp), GFP_KERNEL);
    if (!hp)
        return -ENOMEM;

    hp->dev = dev;
    hp->i2s = i2s;

    /* Get memory-mapped registers */
    regs = devm_platform_ioremap_resource(to_platform_device(dev), 1);
    if (IS_ERR(regs))
        return PTR_ERR(regs);

    /* Create regmap */
    hp->regmap = devm_regmap_init_mmio(dev, regs, &orangepi_vr_headphone_regmap_config);
    if (IS_ERR(hp->regmap)) {
        dev_err(dev, "Failed to initialize regmap: %ld\n", PTR_ERR(hp->regmap));
        return PTR_ERR(hp->regmap);
    }

    /* Parse VR-specific properties */
    hp->vr_spatial_audio_enabled = of_property_read_bool(node, "vr,spatial-audio-enabled");
    hp->vr_low_latency_mode = of_property_read_bool(node, "vr,low-latency-mode");

    /* Set default values */
    hp->volume = VR_HP_DEFAULT_VOLUME;
    hp->muted = false;
    hp->enabled = false;
    hp->eq_enabled = false;

    /* Register component */
    dev_set_drvdata(dev, hp);
    ret = devm_snd_soc_register_component(dev, &orangepi_vr_headphone_component,
                                         &orangepi_vr_headphone_dai, 1);
    if (ret) {
        dev_err(dev, "Failed to register component: %d\n", ret);
        return ret;
    }

    dev_info(dev, "Orange Pi CM5 VR Headphone driver initialized\n");
    if (hp->vr_spatial_audio_enabled)
        dev_info(dev, "VR spatial audio enabled\n");
    if (hp->vr_low_latency_mode)
        dev_info(dev, "VR low-latency mode enabled\n");

    return 0;
}
EXPORT_SYMBOL_GPL(orangepi_vr_headphone_init);

MODULE_DESCRIPTION("Orange Pi CM5 VR Headset Headphone Output Driver");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
