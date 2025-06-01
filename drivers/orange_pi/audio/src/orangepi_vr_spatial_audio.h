/*
 * Orange Pi CM5 VR Headset Spatial Audio Module Header
 *
 * Copyright (c) 2025 VR Headset Project
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License version 2 as
 * published by the Free Software Foundation.
 */

#ifndef __ORANGEPI_VR_SPATIAL_AUDIO_H__
#define __ORANGEPI_VR_SPATIAL_AUDIO_H__

#include <linux/device.h>

/* Initialize spatial audio module */
int orangepi_vr_spatial_audio_init(struct device *dev);

#endif /* __ORANGEPI_VR_SPATIAL_AUDIO_H__ */
