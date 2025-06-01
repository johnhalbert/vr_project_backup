/*
 * Orange Pi CM5 VR Headset Power Management Driver
 *
 * Copyright (C) 2025 VR Headset Project
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 */

#ifndef _ORANGEPI_VR_POWER_H_
#define _ORANGEPI_VR_POWER_H_

#include <linux/device.h>
#include <linux/power_supply.h>
#include <linux/thermal.h>
#include <linux/regulator/consumer.h>
#include <linux/platform_device.h>

/* Power profile types */
enum vr_power_profile_type {
    VR_POWER_PROFILE_HIGH_PERFORMANCE,
    VR_POWER_PROFILE_BALANCED,
    VR_POWER_PROFILE_POWER_SAVE,
    VR_POWER_PROFILE_CUSTOM
};

/* Power profile structure */
struct vr_power_profile {
    enum vr_power_profile_type type;
    
    /* CPU settings */
    unsigned int cpu_freq_min;
    unsigned int cpu_freq_max;
    char cpu_governor[32];
    
    /* GPU settings */
    unsigned int gpu_freq_min;
    unsigned int gpu_freq_max;
    
    /* NPU settings */
    unsigned int npu_freq_min;
    unsigned int npu_freq_max;
    
    /* Display settings */
    unsigned int display_brightness;
    unsigned int display_refresh_rate;
    
    /* Misc settings */
    bool wifi_power_save;
    unsigned int sensor_rate;
};

/* Thermal zones */
enum vr_thermal_zone {
    VR_THERMAL_ZONE_CPU,
    VR_THERMAL_ZONE_GPU,
    VR_THERMAL_ZONE_NPU,
    VR_THERMAL_ZONE_BATTERY,
    VR_THERMAL_ZONE_AMBIENT,
    VR_THERMAL_ZONE_COUNT
};

/* Thermal status */
enum vr_thermal_status {
    VR_THERMAL_STATUS_NORMAL,
    VR_THERMAL_STATUS_WARNING,
    VR_THERMAL_STATUS_CRITICAL,
    VR_THERMAL_STATUS_EMERGENCY
};

/* Thermal configuration */
struct vr_thermal_config {
    int trip_points[VR_THERMAL_ZONE_COUNT][3];  /* Warning, Critical, Emergency */
    int hysteresis[VR_THERMAL_ZONE_COUNT];      /* Hysteresis for trip points */
};

/* Thermal status structure */
struct vr_thermal_status {
    enum vr_thermal_status status[VR_THERMAL_ZONE_COUNT];
    int temperature[VR_THERMAL_ZONE_COUNT];     /* 0.1°C */
};

/* Battery status types */
enum vr_battery_status_type {
    VR_BATTERY_STATUS_CHARGING,
    VR_BATTERY_STATUS_DISCHARGING,
    VR_BATTERY_STATUS_FULL,
    VR_BATTERY_STATUS_UNKNOWN
};

/* Charger types */
enum vr_charger_type {
    VR_CHARGER_TYPE_NONE,
    VR_CHARGER_TYPE_USB,
    VR_CHARGER_TYPE_AC,
    VR_CHARGER_TYPE_WIRELESS
};

/* Battery status structure */
struct vr_battery_status {
    enum vr_battery_status_type status;
    enum vr_charger_type charger_type;
    
    unsigned int capacity;        /* 0-100% */
    unsigned int voltage;         /* mV */
    int current;                  /* mA (positive = charging, negative = discharging) */
    int temperature;              /* 0.1°C */
    
    unsigned int time_to_empty;   /* minutes */
    unsigned int time_to_full;    /* minutes */
};

/* Main driver data structure */
struct vr_power_data {
    struct device *dev;
    struct power_supply *battery_psy;
    struct power_supply *charger_psy;
    struct thermal_zone_device *tz_devices[VR_THERMAL_ZONE_COUNT];
    
    /* Regulators */
    struct regulator *vdd_cpu;
    struct regulator *vdd_gpu;
    struct regulator *vdd_npu;
    
    /* Power state */
    struct vr_power_profile current_profile;
    struct vr_battery_status battery_status;
    struct vr_thermal_status thermal_status;
    
    /* Work queue for power management */
    struct workqueue_struct *pm_wq;
    struct delayed_work battery_work;
    struct delayed_work thermal_work;
    
    /* Mutex for protecting data */
    struct mutex lock;
};

/* IOCTL commands */
#define VR_POWER_IOC_MAGIC 'V'
#define VR_POWER_IOCTL_SET_PROFILE    _IOW(VR_POWER_IOC_MAGIC, 1, struct vr_power_profile)
#define VR_POWER_IOCTL_GET_PROFILE    _IOR(VR_POWER_IOC_MAGIC, 2, struct vr_power_profile)
#define VR_POWER_IOCTL_SET_THERMAL    _IOW(VR_POWER_IOC_MAGIC, 3, struct vr_thermal_config)
#define VR_POWER_IOCTL_GET_THERMAL    _IOR(VR_POWER_IOC_MAGIC, 4, struct vr_thermal_config)
#define VR_POWER_IOCTL_SET_BATTERY    _IOW(VR_POWER_IOC_MAGIC, 5, struct vr_battery_status)
#define VR_POWER_IOCTL_GET_BATTERY    _IOR(VR_POWER_IOC_MAGIC, 6, struct vr_battery_status)

/* Function prototypes */
/* Battery management */
int vr_power_init_battery(struct vr_power_data *data);
void vr_power_exit_battery(struct vr_power_data *data);
int vr_power_update_battery_status(struct vr_power_data *data);

/* Thermal management */
int vr_power_init_thermal(struct vr_power_data *data);
void vr_power_exit_thermal(struct vr_power_data *data);
int vr_power_update_thermal_status(struct vr_power_data *data);
int vr_power_handle_thermal_event(struct vr_power_data *data, enum vr_thermal_zone zone);

/* Power profile management */
int vr_power_init_profile(struct vr_power_data *data);
void vr_power_exit_profile(struct vr_power_data *data);
int vr_power_set_profile(struct vr_power_data *data, struct vr_power_profile *profile);
int vr_power_get_profile(struct vr_power_data *data, struct vr_power_profile *profile);

/* DVFS management */
int vr_power_init_dvfs(struct vr_power_data *data);
void vr_power_exit_dvfs(struct vr_power_data *data);
int vr_power_set_cpu_freq(struct vr_power_data *data, unsigned int min, unsigned int max);
int vr_power_set_gpu_freq(struct vr_power_data *data, unsigned int min, unsigned int max);
int vr_power_set_npu_freq(struct vr_power_data *data, unsigned int min, unsigned int max);

/* Sysfs interface */
int vr_power_init_sysfs(struct vr_power_data *data);
void vr_power_exit_sysfs(struct vr_power_data *data);

#endif /* _ORANGEPI_VR_POWER_H_ */
