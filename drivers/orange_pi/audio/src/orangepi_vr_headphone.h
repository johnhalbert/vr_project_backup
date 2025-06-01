/*
 * Orange Pi CM5 VR Headset Headphone Output Driver Header
 *
 * Copyright (c) 2025 VR Headset Project
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License version 2 as
 * published by the Free Software Foundation.
 */

#ifndef __ORANGEPI_VR_HEADPHONE_H__
#define __ORANGEPI_VR_HEADPHONE_H__

#include <linux/device.h>
#include "orangepi_vr_i2s.h"

/* Initialize headphone driver */
int orangepi_vr_headphone_init(struct device *dev, struct orangepi_vr_i2s_dev *i2s);

#endif /* __ORANGEPI_VR_HEADPHONE_H__ */
