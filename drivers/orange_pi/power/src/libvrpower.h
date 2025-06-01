/*
 * Orange Pi CM5 VR Headset Power Management Library
 *
 * Copyright (C) 2025 VR Headset Project
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 */

#ifndef _LIBVRPOWER_H_
#define _LIBVRPOWER_H_

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>

/* Power profile types */
typedef enum {
    VR_POWER_PROFILE_HIGH_PERFORMANCE = 0,
    VR_POWER_PROFILE_BALANCED = 1,
    VR_POWER_PROFILE_POWER_SAVE = 2,
    VR_POWER_PROFILE_CUSTOM = 3
} vr_power_profile_t;

/* Battery status types */
typedef enum {
    VR_BATTERY_STATUS_CHARGING = 0,
    VR_BATTERY_STATUS_DISCHARGING = 1,
    VR_BATTERY_STATUS_FULL = 2,
    VR_BATTERY_STATUS_UNKNOWN = 3
} vr_battery_status_type_t;

/* Charger types */
typedef enum {
    VR_CHARGER_TYPE_NONE = 0,
    VR_CHARGER_TYPE_USB = 1,
    VR_CHARGER_TYPE_AC = 2,
    VR_CHARGER_TYPE_WIRELESS = 3
} vr_charger_type_t;

/* Thermal zones */
typedef enum {
    VR_THERMAL_ZONE_CPU = 0,
    VR_THERMAL_ZONE_GPU = 1,
    VR_THERMAL_ZONE_NPU = 2,
    VR_THERMAL_ZONE_BATTERY = 3,
    VR_THERMAL_ZONE_AMBIENT = 4,
    VR_THERMAL_ZONE_COUNT = 5
} vr_thermal_zone_t;

/* Thermal status */
typedef enum {
    VR_THERMAL_STATUS_NORMAL = 0,
    VR_THERMAL_STATUS_WARNING = 1,
    VR_THERMAL_STATUS_CRITICAL = 2,
    VR_THERMAL_STATUS_EMERGENCY = 3
} vr_thermal_status_t;

/* Power profile structure */
typedef struct {
    vr_power_profile_t type;
    
    /* CPU settings */
    uint32_t cpu_freq_min;
    uint32_t cpu_freq_max;
    char cpu_governor[32];
    
    /* GPU settings */
    uint32_t gpu_freq_min;
    uint32_t gpu_freq_max;
    
    /* NPU settings */
    uint32_t npu_freq_min;
    uint32_t npu_freq_max;
    
    /* Display settings */
    uint32_t display_brightness;
    uint32_t display_refresh_rate;
    
    /* Misc settings */
    bool wifi_power_save;
    uint32_t sensor_rate;
} vr_power_profile_info_t;

/* Battery status structure */
typedef struct {
    vr_battery_status_type_t status;
    vr_charger_type_t charger_type;
    
    uint32_t capacity;        /* 0-100% */
    uint32_t voltage;         /* mV */
    int32_t current;          /* mA (positive = charging, negative = discharging) */
    int32_t temperature;      /* 0.1°C */
    
    uint32_t time_to_empty;   /* minutes */
    uint32_t time_to_full;    /* minutes */
} vr_battery_status_info_t;

/* Thermal status structure */
typedef struct {
    vr_thermal_status_t status[VR_THERMAL_ZONE_COUNT];
    int32_t temperature[VR_THERMAL_ZONE_COUNT];     /* 0.1°C */
} vr_thermal_status_info_t;

/* Callback function types */
typedef void (*vr_power_profile_callback_t)(vr_power_profile_t profile);
typedef void (*vr_battery_callback_t)(const vr_battery_status_info_t *status);
typedef void (*vr_thermal_callback_t)(const vr_thermal_status_info_t *status);

/*
 * Library initialization and cleanup
 */

/**
 * Initialize the VR power management library.
 *
 * @return 0 on success, negative error code on failure
 */
int vr_power_init(void);

/**
 * Clean up the VR power management library.
 */
void vr_power_cleanup(void);

/*
 * Power profile API
 */

/**
 * Set the power profile.
 *
 * @param profile The power profile to set
 * @return 0 on success, negative error code on failure
 */
int vr_power_set_profile(vr_power_profile_t profile);

/**
 * Get the current power profile.
 *
 * @return The current power profile
 */
vr_power_profile_t vr_power_get_profile(void);

/**
 * Get detailed information about the current power profile.
 *
 * @param info Pointer to a vr_power_profile_info_t structure to fill
 * @return 0 on success, negative error code on failure
 */
int vr_power_get_profile_info(vr_power_profile_info_t *info);

/**
 * Register a callback function to be called when the power profile changes.
 *
 * @param callback The callback function
 * @return 0 on success, negative error code on failure
 */
int vr_power_register_profile_callback(vr_power_profile_callback_t callback);

/**
 * Unregister a previously registered power profile callback function.
 *
 * @param callback The callback function to unregister
 * @return 0 on success, negative error code on failure
 */
int vr_power_unregister_profile_callback(vr_power_profile_callback_t callback);

/*
 * Battery API
 */

/**
 * Get the current battery status.
 *
 * @param status Pointer to a vr_battery_status_info_t structure to fill
 * @return 0 on success, negative error code on failure
 */
int vr_power_get_battery_status(vr_battery_status_info_t *status);

/**
 * Register a callback function to be called when the battery status changes.
 *
 * @param callback The callback function
 * @return 0 on success, negative error code on failure
 */
int vr_power_register_battery_callback(vr_battery_callback_t callback);

/**
 * Unregister a previously registered battery callback function.
 *
 * @param callback The callback function to unregister
 * @return 0 on success, negative error code on failure
 */
int vr_power_unregister_battery_callback(vr_battery_callback_t callback);

/*
 * Thermal API
 */

/**
 * Get the current thermal status.
 *
 * @param status Pointer to a vr_thermal_status_info_t structure to fill
 * @return 0 on success, negative error code on failure
 */
int vr_power_get_thermal_status(vr_thermal_status_info_t *status);

/**
 * Register a callback function to be called when the thermal status changes.
 *
 * @param callback The callback function
 * @return 0 on success, negative error code on failure
 */
int vr_power_register_thermal_callback(vr_thermal_callback_t callback);

/**
 * Unregister a previously registered thermal callback function.
 *
 * @param callback The callback function to unregister
 * @return 0 on success, negative error code on failure
 */
int vr_power_unregister_thermal_callback(vr_thermal_callback_t callback);

#ifdef __cplusplus
}
#endif

#endif /* _LIBVRPOWER_H_ */
