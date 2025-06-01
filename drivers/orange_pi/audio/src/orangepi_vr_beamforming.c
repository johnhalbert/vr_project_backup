/*
 * Orange Pi CM5 VR Headset Beamforming Module
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
#include "orangepi_vr_beamforming.h"

/* Beamforming registers */
#define BEAM_CTRL              0x00
#define BEAM_CONFIG            0x01
#define BEAM_STATUS            0x02
#define BEAM_DIRECTION         0x03
#define BEAM_PATTERN           0x04
#define BEAM_NOISE             0x05
#define BEAM_GAIN              0x06

/* BEAM_CTRL bits */
#define BEAM_CTRL_ENABLE       (1 << 0)
#define BEAM_CTRL_ADAPTIVE     (1 << 1)
#define BEAM_CTRL_FIXED        (1 << 2)
#define BEAM_CTRL_TRACKING     (1 << 3)
#define BEAM_CTRL_NOISE        (1 << 4)
#define BEAM_CTRL_LOWLAT       (1 << 5)

/* VR-specific configuration */
#define VR_BEAM_DEFAULT_GAIN   80  /* 0-100 scale */
#define VR_BEAM_MAX_GAIN       100
#define VR_BEAM_MIN_GAIN       0
#define VR_BEAM_DEFAULT_DIR    0   /* 0-359 degrees */

struct orangepi_vr_beamforming {
    struct device *dev;
    struct regmap *regmap;
    
    bool vr_low_latency_mode;
    
    unsigned int gain;
    bool enabled;
    bool adaptive_mode;
    bool tracking_mode;
    bool noise_reduction;
    
    /* Beam direction in degrees (0-359) */
    unsigned int direction;
    
    /* ALSA controls */
    struct snd_kcontrol_new *controls;
    int num_controls;
};

/* ALSA mixer controls */
static const DECLARE_TLV_DB_SCALE(beam_gain_tlv, -9000, 100, 0);

static int orangepi_vr_beam_get_gain(struct snd_kcontrol *kcontrol,
                                   struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_beamforming *beam = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = beam->gain;

    return 0;
}

static int orangepi_vr_beam_set_gain(struct snd_kcontrol *kcontrol,
                                   struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_beamforming *beam = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    if (val > VR_BEAM_MAX_GAIN)
        return -EINVAL;

    beam->gain = val;
    
    /* Update hardware gain */
    regmap_write(beam->regmap, BEAM_GAIN, beam->gain);

    return 0;
}

static int orangepi_vr_beam_get_enable(struct snd_kcontrol *kcontrol,
                                     struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_beamforming *beam = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = beam->enabled ? 1 : 0;

    return 0;
}

static int orangepi_vr_beam_set_enable(struct snd_kcontrol *kcontrol,
                                     struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_beamforming *beam = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    beam->enabled = val ? true : false;
    
    /* Update beamforming state */
    regmap_update_bits(beam->regmap, BEAM_CTRL, BEAM_CTRL_ENABLE,
                      beam->enabled ? BEAM_CTRL_ENABLE : 0);

    return 0;
}

static int orangepi_vr_beam_get_direction(struct snd_kcontrol *kcontrol,
                                        struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_beamforming *beam = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = beam->direction;

    return 0;
}

static int orangepi_vr_beam_set_direction(struct snd_kcontrol *kcontrol,
                                        struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_beamforming *beam = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    if (val > 359)
        return -EINVAL;

    beam->direction = val;
    
    /* Update beam direction */
    regmap_write(beam->regmap, BEAM_DIRECTION, beam->direction);

    return 0;
}

static int orangepi_vr_beam_get_adaptive(struct snd_kcontrol *kcontrol,
                                       struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_beamforming *beam = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = beam->adaptive_mode ? 1 : 0;

    return 0;
}

static int orangepi_vr_beam_set_adaptive(struct snd_kcontrol *kcontrol,
                                       struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_beamforming *beam = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    beam->adaptive_mode = val ? true : false;
    
    /* Update adaptive mode */
    regmap_update_bits(beam->regmap, BEAM_CTRL, BEAM_CTRL_ADAPTIVE,
                      beam->adaptive_mode ? BEAM_CTRL_ADAPTIVE : 0);

    return 0;
}

static int orangepi_vr_beam_get_tracking(struct snd_kcontrol *kcontrol,
                                       struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_beamforming *beam = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = beam->tracking_mode ? 1 : 0;

    return 0;
}

static int orangepi_vr_beam_set_tracking(struct snd_kcontrol *kcontrol,
                                       struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_beamforming *beam = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    beam->tracking_mode = val ? true : false;
    
    /* Update tracking mode */
    regmap_update_bits(beam->regmap, BEAM_CTRL, BEAM_CTRL_TRACKING,
                      beam->tracking_mode ? BEAM_CTRL_TRACKING : 0);

    return 0;
}

static const struct snd_kcontrol_new orangepi_vr_beamforming_controls[] = {
    SOC_SINGLE_EXT_TLV("Beamforming Gain", BEAM_GAIN,
                      0, VR_BEAM_MAX_GAIN, 0,
                      orangepi_vr_beam_get_gain,
                      orangepi_vr_beam_set_gain,
                      beam_gain_tlv),
    SOC_SINGLE_BOOL_EXT("Beamforming Enable", 0,
                       orangepi_vr_beam_get_enable,
                       orangepi_vr_beam_set_enable),
    SOC_SINGLE_EXT("Beam Direction", BEAM_DIRECTION,
                  0, 359, 0,
                  orangepi_vr_beam_get_direction,
                  orangepi_vr_beam_set_direction),
    SOC_SINGLE_BOOL_EXT("Adaptive Beamforming", 0,
                       orangepi_vr_beam_get_adaptive,
                       orangepi_vr_beam_set_adaptive),
    SOC_SINGLE_BOOL_EXT("Voice Tracking", 0,
                       orangepi_vr_beam_get_tracking,
                       orangepi_vr_beam_set_tracking),
};

static const struct regmap_config orangepi_vr_beamforming_regmap_config = {
    .reg_bits = 8,
    .val_bits = 8,
    .max_register = BEAM_GAIN,
    .cache_type = REGCACHE_RBTREE,
};

int orangepi_vr_beamforming_init(struct device *dev)
{
    struct orangepi_vr_beamforming *beam;
    struct device_node *node = dev->of_node;
    void __iomem *regs;
    int ret;

    beam = devm_kzalloc(dev, sizeof(*beam), GFP_KERNEL);
    if (!beam)
        return -ENOMEM;

    beam->dev = dev;

    /* Get memory-mapped registers */
    regs = devm_platform_ioremap_resource(to_platform_device(dev), 3);
    if (IS_ERR(regs))
        return PTR_ERR(regs);

    /* Create regmap */
    beam->regmap = devm_regmap_init_mmio(dev, regs, &orangepi_vr_beamforming_regmap_config);
    if (IS_ERR(beam->regmap)) {
        dev_err(dev, "Failed to initialize regmap: %ld\n", PTR_ERR(beam->regmap));
        return PTR_ERR(beam->regmap);
    }

    /* Parse VR-specific properties */
    beam->vr_low_latency_mode = of_property_read_bool(node, "vr,low-latency-mode");

    /* Set default values */
    beam->gain = VR_BEAM_DEFAULT_GAIN;
    beam->direction = VR_BEAM_DEFAULT_DIR;
    beam->enabled = true;
    beam->adaptive_mode = true;
    beam->tracking_mode = true;
    beam->noise_reduction = true;

    /* Initialize beamforming hardware */
    regmap_write(beam->regmap, BEAM_GAIN, beam->gain);
    regmap_write(beam->regmap, BEAM_DIRECTION, beam->direction);
    
    regmap_write(beam->regmap, BEAM_CTRL, 
                BEAM_CTRL_ENABLE | 
                BEAM_CTRL_ADAPTIVE | 
                BEAM_CTRL_TRACKING | 
                BEAM_CTRL_NOISE);
    
    /* Configure beamforming pattern */
    regmap_write(beam->regmap, BEAM_PATTERN, 0x80); /* Default pattern configuration */
    
    /* Configure noise reduction */
    regmap_write(beam->regmap, BEAM_NOISE, 0x80); /* Default noise reduction configuration */

    /* Configure low latency mode if enabled */
    if (beam->vr_low_latency_mode) {
        regmap_update_bits(beam->regmap, BEAM_CTRL, BEAM_CTRL_LOWLAT, BEAM_CTRL_LOWLAT);
    }

    dev_info(dev, "Orange Pi CM5 VR Beamforming module initialized\n");
    if (beam->vr_low_latency_mode)
        dev_info(dev, "VR low-latency mode enabled\n");

    return 0;
}
EXPORT_SYMBOL_GPL(orangepi_vr_beamforming_init);

MODULE_DESCRIPTION("Orange Pi CM5 VR Headset Beamforming Module");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
