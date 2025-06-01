/*
 * Orange Pi CM5 VR Headset Power Management Library Implementation
 *
 * Copyright (C) 2025 VR Headset Project
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <sys/ioctl.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <pthread.h>
#include <signal.h>

#include "libvrpower.h"
#include "orangepi_vr_power.h" /* Kernel driver header */

#define DEVICE_PATH "/dev/orangepi-vr-power"
#define MAX_CALLBACKS 10

/* Global variables */
static int g_fd = -1;
static pthread_mutex_t g_lock = PTHREAD_MUTEX_INITIALIZER;
static int g_initialized = 0;

/* Callback arrays */
static vr_power_profile_callback_t g_profile_callbacks[MAX_CALLBACKS];
static vr_battery_callback_t g_battery_callbacks[MAX_CALLBACKS];
static vr_thermal_callback_t g_thermal_callbacks[MAX_CALLBACKS];

/* Current state */
static vr_power_profile_t g_current_profile;
static vr_battery_status_info_t g_battery_status;
static vr_thermal_status_info_t g_thermal_status;

/* Initialize the library */
int vr_power_init(void)
{
    int ret = 0;
    
    pthread_mutex_lock(&g_lock);
    
    if (g_initialized) {
        pthread_mutex_unlock(&g_lock);
        return 0;
    }
    
    /* Open the device */
    g_fd = open(DEVICE_PATH, O_RDWR);
    if (g_fd < 0) {
        ret = -errno;
        pthread_mutex_unlock(&g_lock);
        return ret;
    }
    
    /* Initialize callback arrays */
    memset(g_profile_callbacks, 0, sizeof(g_profile_callbacks));
    memset(g_battery_callbacks, 0, sizeof(g_battery_callbacks));
    memset(g_thermal_callbacks, 0, sizeof(g_thermal_callbacks));
    
    /* Get initial state */
    struct vr_power_profile profile;
    if (ioctl(g_fd, VR_POWER_IOCTL_GET_PROFILE, &profile) < 0) {
        ret = -errno;
        close(g_fd);
        pthread_mutex_unlock(&g_lock);
        return ret;
    }
    g_current_profile = profile.type;
    
    struct vr_battery_status battery;
    if (ioctl(g_fd, VR_POWER_IOCTL_GET_BATTERY, &battery) < 0) {
        ret = -errno;
        close(g_fd);
        pthread_mutex_unlock(&g_lock);
        return ret;
    }
    
    /* Convert kernel structures to library structures */
    g_battery_status.status = battery.status;
    g_battery_status.charger_type = battery.charger_type;
    g_battery_status.capacity = battery.capacity;
    g_battery_status.voltage = battery.voltage;
    g_battery_status.current = battery.current;
    g_battery_status.temperature = battery.temperature;
    g_battery_status.time_to_empty = battery.time_to_empty;
    g_battery_status.time_to_full = battery.time_to_full;
    
    /* TODO: Get thermal status when kernel API is available */
    
    g_initialized = 1;
    pthread_mutex_unlock(&g_lock);
    
    return 0;
}

/* Clean up the library */
void vr_power_cleanup(void)
{
    pthread_mutex_lock(&g_lock);
    
    if (!g_initialized) {
        pthread_mutex_unlock(&g_lock);
        return;
    }
    
    if (g_fd >= 0) {
        close(g_fd);
        g_fd = -1;
    }
    
    g_initialized = 0;
    pthread_mutex_unlock(&g_lock);
}

/* Set power profile */
int vr_power_set_profile(vr_power_profile_t profile)
{
    int ret = 0;
    struct vr_power_profile kernel_profile;
    int i;
    
    pthread_mutex_lock(&g_lock);
    
    if (!g_initialized) {
        pthread_mutex_unlock(&g_lock);
        return -EINVAL;
    }
    
    /* Initialize profile based on type */
    memset(&kernel_profile, 0, sizeof(kernel_profile));
    kernel_profile.type = profile;
    
    switch (profile) {
    case VR_POWER_PROFILE_HIGH_PERFORMANCE:
        kernel_profile.cpu_freq_min = 1800000;
        kernel_profile.cpu_freq_max = 2400000;
        strcpy(kernel_profile.cpu_governor, "performance");
        kernel_profile.gpu_freq_min = 800000;
        kernel_profile.gpu_freq_max = 1000000;
        kernel_profile.npu_freq_min = 800000;
        kernel_profile.npu_freq_max = 1000000;
        kernel_profile.display_brightness = 255;
        kernel_profile.display_refresh_rate = 90;
        kernel_profile.wifi_power_save = 0;
        kernel_profile.sensor_rate = 1000;
        break;
    case VR_POWER_PROFILE_BALANCED:
        kernel_profile.cpu_freq_min = 1200000;
        kernel_profile.cpu_freq_max = 2000000;
        strcpy(kernel_profile.cpu_governor, "schedutil");
        kernel_profile.gpu_freq_min = 600000;
        kernel_profile.gpu_freq_max = 800000;
        kernel_profile.npu_freq_min = 600000;
        kernel_profile.npu_freq_max = 800000;
        kernel_profile.display_brightness = 200;
        kernel_profile.display_refresh_rate = 90;
        kernel_profile.wifi_power_save = 0;
        kernel_profile.sensor_rate = 500;
        break;
    case VR_POWER_PROFILE_POWER_SAVE:
        kernel_profile.cpu_freq_min = 600000;
        kernel_profile.cpu_freq_max = 1500000;
        strcpy(kernel_profile.cpu_governor, "powersave");
        kernel_profile.gpu_freq_min = 400000;
        kernel_profile.gpu_freq_max = 600000;
        kernel_profile.npu_freq_min = 400000;
        kernel_profile.npu_freq_max = 600000;
        kernel_profile.display_brightness = 150;
        kernel_profile.display_refresh_rate = 60;
        kernel_profile.wifi_power_save = 1;
        kernel_profile.sensor_rate = 200;
        break;
    default:
        pthread_mutex_unlock(&g_lock);
        return -EINVAL;
    }
    
    /* Set the profile */
    if (ioctl(g_fd, VR_POWER_IOCTL_SET_PROFILE, &kernel_profile) < 0) {
        ret = -errno;
        pthread_mutex_unlock(&g_lock);
        return ret;
    }
    
    /* Update current profile */
    g_current_profile = profile;
    
    /* Call callbacks */
    for (i = 0; i < MAX_CALLBACKS; i++) {
        if (g_profile_callbacks[i]) {
            g_profile_callbacks[i](profile);
        }
    }
    
    pthread_mutex_unlock(&g_lock);
    
    return 0;
}

/* Get current power profile */
vr_power_profile_t vr_power_get_profile(void)
{
    vr_power_profile_t profile;
    
    pthread_mutex_lock(&g_lock);
    
    if (!g_initialized) {
        pthread_mutex_unlock(&g_lock);
        return VR_POWER_PROFILE_BALANCED; /* Default */
    }
    
    profile = g_current_profile;
    
    pthread_mutex_unlock(&g_lock);
    
    return profile;
}

/* Get detailed information about the current power profile */
int vr_power_get_profile_info(vr_power_profile_info_t *info)
{
    int ret = 0;
    struct vr_power_profile kernel_profile;
    
    if (!info) {
        return -EINVAL;
    }
    
    pthread_mutex_lock(&g_lock);
    
    if (!g_initialized) {
        pthread_mutex_unlock(&g_lock);
        return -EINVAL;
    }
    
    /* Get profile from kernel */
    if (ioctl(g_fd, VR_POWER_IOCTL_GET_PROFILE, &kernel_profile) < 0) {
        ret = -errno;
        pthread_mutex_unlock(&g_lock);
        return ret;
    }
    
    /* Convert kernel structure to library structure */
    info->type = kernel_profile.type;
    info->cpu_freq_min = kernel_profile.cpu_freq_min;
    info->cpu_freq_max = kernel_profile.cpu_freq_max;
    strcpy(info->cpu_governor, kernel_profile.cpu_governor);
    info->gpu_freq_min = kernel_profile.gpu_freq_min;
    info->gpu_freq_max = kernel_profile.gpu_freq_max;
    info->npu_freq_min = kernel_profile.npu_freq_min;
    info->npu_freq_max = kernel_profile.npu_freq_max;
    info->display_brightness = kernel_profile.display_brightness;
    info->display_refresh_rate = kernel_profile.display_refresh_rate;
    info->wifi_power_save = kernel_profile.wifi_power_save;
    info->sensor_rate = kernel_profile.sensor_rate;
    
    pthread_mutex_unlock(&g_lock);
    
    return 0;
}

/* Register a callback function to be called when the power profile changes */
int vr_power_register_profile_callback(vr_power_profile_callback_t callback)
{
    int i;
    
    if (!callback) {
        return -EINVAL;
    }
    
    pthread_mutex_lock(&g_lock);
    
    if (!g_initialized) {
        pthread_mutex_unlock(&g_lock);
        return -EINVAL;
    }
    
    /* Find an empty slot */
    for (i = 0; i < MAX_CALLBACKS; i++) {
        if (!g_profile_callbacks[i]) {
            g_profile_callbacks[i] = callback;
            pthread_mutex_unlock(&g_lock);
            return 0;
        }
    }
    
    pthread_mutex_unlock(&g_lock);
    
    return -ENOSPC;
}

/* Unregister a previously registered power profile callback function */
int vr_power_unregister_profile_callback(vr_power_profile_callback_t callback)
{
    int i;
    
    if (!callback) {
        return -EINVAL;
    }
    
    pthread_mutex_lock(&g_lock);
    
    if (!g_initialized) {
        pthread_mutex_unlock(&g_lock);
        return -EINVAL;
    }
    
    /* Find the callback */
    for (i = 0; i < MAX_CALLBACKS; i++) {
        if (g_profile_callbacks[i] == callback) {
            g_profile_callbacks[i] = NULL;
            pthread_mutex_unlock(&g_lock);
            return 0;
        }
    }
    
    pthread_mutex_unlock(&g_lock);
    
    return -ENOENT;
}

/* Get the current battery status */
int vr_power_get_battery_status(vr_battery_status_info_t *status)
{
    int ret = 0;
    struct vr_battery_status kernel_status;
    
    if (!status) {
        return -EINVAL;
    }
    
    pthread_mutex_lock(&g_lock);
    
    if (!g_initialized) {
        pthread_mutex_unlock(&g_lock);
        return -EINVAL;
    }
    
    /* Get battery status from kernel */
    if (ioctl(g_fd, VR_POWER_IOCTL_GET_BATTERY, &kernel_status) < 0) {
        ret = -errno;
        pthread_mutex_unlock(&g_lock);
        return ret;
    }
    
    /* Convert kernel structure to library structure */
    status->status = kernel_status.status;
    status->charger_type = kernel_status.charger_type;
    status->capacity = kernel_status.capacity;
    status->voltage = kernel_status.voltage;
    status->current = kernel_status.current;
    status->temperature = kernel_status.temperature;
    status->time_to_empty = kernel_status.time_to_empty;
    status->time_to_full = kernel_status.time_to_full;
    
    pthread_mutex_unlock(&g_lock);
    
    return 0;
}

/* Register a callback function to be called when the battery status changes */
int vr_power_register_battery_callback(vr_battery_callback_t callback)
{
    int i;
    
    if (!callback) {
        return -EINVAL;
    }
    
    pthread_mutex_lock(&g_lock);
    
    if (!g_initialized) {
        pthread_mutex_unlock(&g_lock);
        return -EINVAL;
    }
    
    /* Find an empty slot */
    for (i = 0; i < MAX_CALLBACKS; i++) {
        if (!g_battery_callbacks[i]) {
            g_battery_callbacks[i] = callback;
            pthread_mutex_unlock(&g_lock);
            return 0;
        }
    }
    
    pthread_mutex_unlock(&g_lock);
    
    return -ENOSPC;
}

/* Unregister a previously registered battery callback function */
int vr_power_unregister_battery_callback(vr_battery_callback_t callback)
{
    int i;
    
    if (!callback) {
        return -EINVAL;
    }
    
    pthread_mutex_lock(&g_lock);
    
    if (!g_initialized) {
        pthread_mutex_unlock(&g_lock);
        return -EINVAL;
    }
    
    /* Find the callback */
    for (i = 0; i < MAX_CALLBACKS; i++) {
        if (g_battery_callbacks[i] == callback) {
            g_battery_callbacks[i] = NULL;
            pthread_mutex_unlock(&g_lock);
            return 0;
        }
    }
    
    pthread_mutex_unlock(&g_lock);
    
    return -ENOENT;
}

/* Get the current thermal status */
int vr_power_get_thermal_status(vr_thermal_status_info_t *status)
{
    if (!status) {
        return -EINVAL;
    }
    
    pthread_mutex_lock(&g_lock);
    
    if (!g_initialized) {
        pthread_mutex_unlock(&g_lock);
        return -EINVAL;
    }
    
    /* TODO: Implement when kernel API is available */
    /* For now, just return the cached status */
    *status = g_thermal_status;
    
    pthread_mutex_unlock(&g_lock);
    
    return 0;
}

/* Register a callback function to be called when the thermal status changes */
int vr_power_register_thermal_callback(vr_thermal_callback_t callback)
{
    int i;
    
    if (!callback) {
        return -EINVAL;
    }
    
    pthread_mutex_lock(&g_lock);
    
    if (!g_initialized) {
        pthread_mutex_unlock(&g_lock);
        return -EINVAL;
    }
    
    /* Find an empty slot */
    for (i = 0; i < MAX_CALLBACKS; i++) {
        if (!g_thermal_callbacks[i]) {
            g_thermal_callbacks[i] = callback;
            pthread_mutex_unlock(&g_lock);
            return 0;
        }
    }
    
    pthread_mutex_unlock(&g_lock);
    
    return -ENOSPC;
}

/* Unregister a previously registered thermal callback function */
int vr_power_unregister_thermal_callback(vr_thermal_callback_t callback)
{
    int i;
    
    if (!callback) {
        return -EINVAL;
    }
    
    pthread_mutex_lock(&g_lock);
    
    if (!g_initialized) {
        pthread_mutex_unlock(&g_lock);
        return -EINVAL;
    }
    
    /* Find the callback */
    for (i = 0; i < MAX_CALLBACKS; i++) {
        if (g_thermal_callbacks[i] == callback) {
            g_thermal_callbacks[i] = NULL;
            pthread_mutex_unlock(&g_lock);
            return 0;
        }
    }
    
    pthread_mutex_unlock(&g_lock);
    
    return -ENOENT;
}
