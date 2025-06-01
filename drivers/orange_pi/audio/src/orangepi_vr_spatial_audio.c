/*
 * Orange Pi CM5 VR Headset Spatial Audio Module
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
#include "orangepi_vr_spatial_audio.h"

/* Spatial audio registers */
#define SPATIAL_CTRL           0x00
#define SPATIAL_CONFIG         0x01
#define SPATIAL_STATUS         0x02
#define SPATIAL_HRTF           0x03
#define SPATIAL_ROOM           0x04
#define SPATIAL_POSITION       0x05
#define SPATIAL_EQ             0x06

/* SPATIAL_CTRL bits */
#define SPATIAL_CTRL_ENABLE    (1 << 0)
#define SPATIAL_CTRL_HRTF      (1 << 1)
#define SPATIAL_CTRL_ROOM      (1 << 2)
#define SPATIAL_CTRL_POSITION  (1 << 3)
#define SPATIAL_CTRL_EQ        (1 << 4)
#define SPATIAL_CTRL_LOWLAT    (1 << 5)

/* VR-specific configuration */
#define VR_SPATIAL_DEFAULT_ROOM_SIZE  50  /* 0-100 scale */
#define VR_SPATIAL_MAX_ROOM_SIZE      100
#define VR_SPATIAL_MIN_ROOM_SIZE      0

struct orangepi_vr_spatial_audio {
    struct device *dev;
    struct regmap *regmap;
    
    bool vr_low_latency_mode;
    
    bool enabled;
    bool hrtf_enabled;
    bool room_acoustics_enabled;
    bool position_tracking_enabled;
    bool eq_enabled;
    
    unsigned int room_size;
    
    /* 3D position (x, y, z) in normalized coordinates (-1.0 to 1.0) */
    int position_x;
    int position_y;
    int position_z;
    
    /* ALSA controls */
    struct snd_kcontrol_new *controls;
    int num_controls;
};

/* ALSA mixer controls */
static int orangepi_vr_spatial_get_enable(struct snd_kcontrol *kcontrol,
                                        struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_spatial_audio *spatial = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = spatial->enabled ? 1 : 0;

    return 0;
}

static int orangepi_vr_spatial_set_enable(struct snd_kcontrol *kcontrol,
                                        struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_spatial_audio *spatial = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    spatial->enabled = val ? true : false;
    
    /* Update spatial audio state */
    regmap_update_bits(spatial->regmap, SPATIAL_CTRL, SPATIAL_CTRL_ENABLE,
                      spatial->enabled ? SPATIAL_CTRL_ENABLE : 0);

    return 0;
}

static int orangepi_vr_spatial_get_hrtf(struct snd_kcontrol *kcontrol,
                                      struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_spatial_audio *spatial = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = spatial->hrtf_enabled ? 1 : 0;

    return 0;
}

static int orangepi_vr_spatial_set_hrtf(struct snd_kcontrol *kcontrol,
                                      struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_spatial_audio *spatial = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    spatial->hrtf_enabled = val ? true : false;
    
    /* Update HRTF state */
    regmap_update_bits(spatial->regmap, SPATIAL_CTRL, SPATIAL_CTRL_HRTF,
                      spatial->hrtf_enabled ? SPATIAL_CTRL_HRTF : 0);

    return 0;
}

static int orangepi_vr_spatial_get_room(struct snd_kcontrol *kcontrol,
                                      struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_spatial_audio *spatial = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = spatial->room_acoustics_enabled ? 1 : 0;

    return 0;
}

static int orangepi_vr_spatial_set_room(struct snd_kcontrol *kcontrol,
                                      struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_spatial_audio *spatial = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    spatial->room_acoustics_enabled = val ? true : false;
    
    /* Update room acoustics state */
    regmap_update_bits(spatial->regmap, SPATIAL_CTRL, SPATIAL_CTRL_ROOM,
                      spatial->room_acoustics_enabled ? SPATIAL_CTRL_ROOM : 0);

    return 0;
}

static int orangepi_vr_spatial_get_room_size(struct snd_kcontrol *kcontrol,
                                           struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_spatial_audio *spatial = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = spatial->room_size;

    return 0;
}

static int orangepi_vr_spatial_set_room_size(struct snd_kcontrol *kcontrol,
                                           struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_spatial_audio *spatial = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    if (val > VR_SPATIAL_MAX_ROOM_SIZE)
        return -EINVAL;

    spatial->room_size = val;
    
    /* Update room size */
    regmap_write(spatial->regmap, SPATIAL_ROOM, spatial->room_size);

    return 0;
}

static int orangepi_vr_spatial_get_position(struct snd_kcontrol *kcontrol,
                                          struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_spatial_audio *spatial = snd_soc_component_get_drvdata(component);

    ucontrol->value.integer.value[0] = spatial->position_tracking_enabled ? 1 : 0;

    return 0;
}

static int orangepi_vr_spatial_set_position(struct snd_kcontrol *kcontrol,
                                          struct snd_ctl_elem_value *ucontrol)
{
    struct snd_soc_component *component = snd_soc_kcontrol_component(kcontrol);
    struct orangepi_vr_spatial_audio *spatial = snd_soc_component_get_drvdata(component);
    unsigned int val;

    val = ucontrol->value.integer.value[0];
    spatial->position_tracking_enabled = val ? true : false;
    
    /* Update position tracking state */
    regmap_update_bits(spatial->regmap, SPATIAL_CTRL, SPATIAL_CTRL_POSITION,
                      spatial->position_tracking_enabled ? SPATIAL_CTRL_POSITION : 0);

    return 0;
}

static const struct snd_kcontrol_new orangepi_vr_spatial_audio_controls[] = {
    SOC_SINGLE_BOOL_EXT("Spatial Audio Enable", 0,
                       orangepi_vr_spatial_get_enable,
                       orangepi_vr_spatial_set_enable),
    SOC_SINGLE_BOOL_EXT("HRTF Enable", 0,
                       orangepi_vr_spatial_get_hrtf,
                       orangepi_vr_spatial_set_hrtf),
    SOC_SINGLE_BOOL_EXT("Room Acoustics Enable", 0,
                       orangepi_vr_spatial_get_room,
                       orangepi_vr_spatial_set_room),
    SOC_SINGLE_EXT("Room Size", SPATIAL_ROOM,
                  0, VR_SPATIAL_MAX_ROOM_SIZE, 0,
                  orangepi_vr_spatial_get_room_size,
                  orangepi_vr_spatial_set_room_size),
    SOC_SINGLE_BOOL_EXT("Position Tracking Enable", 0,
                       orangepi_vr_spatial_get_position,
                       orangepi_vr_spatial_set_position),
};

static const struct regmap_config orangepi_vr_spatial_audio_regmap_config = {
    .reg_bits = 8,
    .val_bits = 8,
    .max_register = SPATIAL_EQ,
    .cache_type = REGCACHE_RBTREE,
};

int orangepi_vr_spatial_audio_init(struct device *dev)
{
    struct orangepi_vr_spatial_audio *spatial;
    struct device_node *node = dev->of_node;
    void __iomem *regs;
    int ret;

    spatial = devm_kzalloc(dev, sizeof(*spatial), GFP_KERNEL);
    if (!spatial)
        return -ENOMEM;

    spatial->dev = dev;

    /* Get memory-mapped registers */
    regs = devm_platform_ioremap_resource(to_platform_device(dev), 4);
    if (IS_ERR(regs))
        return PTR_ERR(regs);

    /* Create regmap */
    spatial->regmap = devm_regmap_init_mmio(dev, regs, &orangepi_vr_spatial_audio_regmap_config);
    if (IS_ERR(spatial->regmap)) {
        dev_err(dev, "Failed to initialize regmap: %ld\n", PTR_ERR(spatial->regmap));
        return PTR_ERR(spatial->regmap);
    }

    /* Parse VR-specific properties */
    spatial->vr_low_latency_mode = of_property_read_bool(node, "vr,low-latency-mode");

    /* Set default values */
    spatial->room_size = VR_SPATIAL_DEFAULT_ROOM_SIZE;
    spatial->enabled = true;
    spatial->hrtf_enabled = true;
    spatial->room_acoustics_enabled = true;
    spatial->position_tracking_enabled = true;
    spatial->eq_enabled = true;
    
    spatial->position_x = 0;
    spatial->position_y = 0;
    spatial->position_z = 0;

    /* Initialize spatial audio hardware */
    regmap_write(spatial->regmap, SPATIAL_ROOM, spatial->room_size);
    regmap_write(spatial->regmap, SPATIAL_POSITION, 0x80); /* Center position */
    
    regmap_write(spatial->regmap, SPATIAL_CTRL, 
                SPATIAL_CTRL_ENABLE | 
                SPATIAL_CTRL_HRTF | 
                SPATIAL_CTRL_ROOM | 
                SPATIAL_CTRL_POSITION | 
                SPATIAL_CTRL_EQ);
    
    /* Configure HRTF */
    regmap_write(spatial->regmap, SPATIAL_HRTF, 0x80); /* Default HRTF configuration */
    
    /* Configure room acoustics */
    regmap_write(spatial->regmap, SPATIAL_ROOM, spatial->room_size);
    
    /* Configure EQ */
    regmap_write(spatial->regmap, SPATIAL_EQ, 0x80); /* Default EQ configuration */

    /* Configure low latency mode if enabled */
    if (spatial->vr_low_latency_mode) {
        regmap_update_bits(spatial->regmap, SPATIAL_CTRL, SPATIAL_CTRL_LOWLAT, SPATIAL_CTRL_LOWLAT);
    }

    dev_info(dev, "Orange Pi CM5 VR Spatial Audio module initialized\n");
    if (spatial->vr_low_latency_mode)
        dev_info(dev, "VR low-latency mode enabled\n");

    return 0;
}
EXPORT_SYMBOL_GPL(orangepi_vr_spatial_audio_init);

MODULE_DESCRIPTION("Orange Pi CM5 VR Headset Spatial Audio Module");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
