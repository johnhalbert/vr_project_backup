/*
 * Orange Pi CM5 VR Headset Power Management Userspace Service
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
#include <getopt.h>

/* Include the same header as the kernel driver */
#include "orangepi_vr_power.h"

#define DEVICE_PATH "/dev/orangepi-vr-power"
#define SERVICE_NAME "vr-power-mgr"
#define SERVICE_VERSION "1.0.0"

/* Global variables */
static int g_fd = -1;
static int g_running = 1;
static pthread_mutex_t g_lock = PTHREAD_MUTEX_INITIALIZER;
static struct vr_power_profile g_current_profile;
static struct vr_battery_status g_battery_status;
static struct vr_thermal_status g_thermal_status;

/* Forward declarations */
static void *battery_monitor_thread(void *arg);
static void *thermal_monitor_thread(void *arg);
static void signal_handler(int sig);

/* Initialize the power management service */
static int init_service(void)
{
    /* Open the device */
    g_fd = open(DEVICE_PATH, O_RDWR);
    if (g_fd < 0) {
        fprintf(stderr, "Failed to open %s: %s\n", DEVICE_PATH, strerror(errno));
        return -1;
    }

    /* Get current profile */
    if (ioctl(g_fd, VR_POWER_IOCTL_GET_PROFILE, &g_current_profile) < 0) {
        fprintf(stderr, "Failed to get current profile: %s\n", strerror(errno));
        close(g_fd);
        return -1;
    }

    /* Get battery status */
    if (ioctl(g_fd, VR_POWER_IOCTL_GET_BATTERY, &g_battery_status) < 0) {
        fprintf(stderr, "Failed to get battery status: %s\n", strerror(errno));
        close(g_fd);
        return -1;
    }

    /* Set up signal handlers */
    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);

    return 0;
}

/* Cleanup the power management service */
static void cleanup_service(void)
{
    if (g_fd >= 0) {
        close(g_fd);
        g_fd = -1;
    }
}

/* Set power profile */
static int set_power_profile(enum vr_power_profile_type type)
{
    struct vr_power_profile profile;

    /* Initialize profile based on type */
    memset(&profile, 0, sizeof(profile));
    profile.type = type;

    switch (type) {
    case VR_POWER_PROFILE_HIGH_PERFORMANCE:
        profile.cpu_freq_min = 1800000;
        profile.cpu_freq_max = 2400000;
        strcpy(profile.cpu_governor, "performance");
        profile.gpu_freq_min = 800000;
        profile.gpu_freq_max = 1000000;
        profile.npu_freq_min = 800000;
        profile.npu_freq_max = 1000000;
        profile.display_brightness = 255;
        profile.display_refresh_rate = 90;
        profile.wifi_power_save = 0;
        profile.sensor_rate = 1000;
        break;
    case VR_POWER_PROFILE_BALANCED:
        profile.cpu_freq_min = 1200000;
        profile.cpu_freq_max = 2000000;
        strcpy(profile.cpu_governor, "schedutil");
        profile.gpu_freq_min = 600000;
        profile.gpu_freq_max = 800000;
        profile.npu_freq_min = 600000;
        profile.npu_freq_max = 800000;
        profile.display_brightness = 200;
        profile.display_refresh_rate = 90;
        profile.wifi_power_save = 0;
        profile.sensor_rate = 500;
        break;
    case VR_POWER_PROFILE_POWER_SAVE:
        profile.cpu_freq_min = 600000;
        profile.cpu_freq_max = 1500000;
        strcpy(profile.cpu_governor, "powersave");
        profile.gpu_freq_min = 400000;
        profile.gpu_freq_max = 600000;
        profile.npu_freq_min = 400000;
        profile.npu_freq_max = 600000;
        profile.display_brightness = 150;
        profile.display_refresh_rate = 60;
        profile.wifi_power_save = 1;
        profile.sensor_rate = 200;
        break;
    default:
        fprintf(stderr, "Invalid profile type: %d\n", type);
        return -1;
    }

    /* Set the profile */
    pthread_mutex_lock(&g_lock);
    if (ioctl(g_fd, VR_POWER_IOCTL_SET_PROFILE, &profile) < 0) {
        fprintf(stderr, "Failed to set profile: %s\n", strerror(errno));
        pthread_mutex_unlock(&g_lock);
        return -1;
    }
    g_current_profile = profile;
    pthread_mutex_unlock(&g_lock);

    printf("Power profile set to %d\n", type);
    return 0;
}

/* Get current power profile */
static int get_power_profile(struct vr_power_profile *profile)
{
    pthread_mutex_lock(&g_lock);
    if (ioctl(g_fd, VR_POWER_IOCTL_GET_PROFILE, profile) < 0) {
        fprintf(stderr, "Failed to get profile: %s\n", strerror(errno));
        pthread_mutex_unlock(&g_lock);
        return -1;
    }
    pthread_mutex_unlock(&g_lock);
    return 0;
}

/* Get battery status */
static int get_battery_status(struct vr_battery_status *status)
{
    pthread_mutex_lock(&g_lock);
    if (ioctl(g_fd, VR_POWER_IOCTL_GET_BATTERY, status) < 0) {
        fprintf(stderr, "Failed to get battery status: %s\n", strerror(errno));
        pthread_mutex_unlock(&g_lock);
        return -1;
    }
    pthread_mutex_unlock(&g_lock);
    return 0;
}

/* Battery monitor thread */
static void *battery_monitor_thread(void *arg)
{
    struct vr_battery_status status;

    while (g_running) {
        /* Get battery status */
        if (get_battery_status(&status) == 0) {
            pthread_mutex_lock(&g_lock);
            g_battery_status = status;
            pthread_mutex_unlock(&g_lock);

            /* Check for low battery */
            if (status.capacity <= 15 && status.status == VR_BATTERY_STATUS_DISCHARGING) {
                printf("Low battery warning: %d%%\n", status.capacity);
                
                /* If battery is critically low, switch to power save mode */
                if (status.capacity <= 5) {
                    printf("Critical battery level, switching to power save mode\n");
                    set_power_profile(VR_POWER_PROFILE_POWER_SAVE);
                }
            }
        }

        /* Sleep for 5 seconds */
        sleep(5);
    }

    return NULL;
}

/* Thermal monitor thread */
static void *thermal_monitor_thread(void *arg)
{
    /* In a real implementation, this would monitor thermal status */
    /* For now, we'll just sleep */
    while (g_running) {
        sleep(5);
    }

    return NULL;
}

/* Signal handler */
static void signal_handler(int sig)
{
    printf("Received signal %d, exiting...\n", sig);
    g_running = 0;
}

/* Print usage */
static void print_usage(const char *prog_name)
{
    printf("Usage: %s [options]\n", prog_name);
    printf("Options:\n");
    printf("  -d, --daemon        Run as daemon\n");
    printf("  -p, --profile TYPE  Set power profile (0=high, 1=balanced, 2=power_save)\n");
    printf("  -s, --status        Show current status\n");
    printf("  -h, --help          Show this help\n");
    printf("  -v, --version       Show version\n");
}

/* Main function */
int main(int argc, char *argv[])
{
    int daemon_mode = 0;
    int show_status = 0;
    int profile_type = -1;
    pthread_t battery_thread, thermal_thread;
    int ret;

    /* Parse command line arguments */
    static struct option long_options[] = {
        {"daemon", no_argument, 0, 'd'},
        {"profile", required_argument, 0, 'p'},
        {"status", no_argument, 0, 's'},
        {"help", no_argument, 0, 'h'},
        {"version", no_argument, 0, 'v'},
        {0, 0, 0, 0}
    };

    int opt;
    int option_index = 0;
    while ((opt = getopt_long(argc, argv, "dp:shv", long_options, &option_index)) != -1) {
        switch (opt) {
        case 'd':
            daemon_mode = 1;
            break;
        case 'p':
            profile_type = atoi(optarg);
            break;
        case 's':
            show_status = 1;
            break;
        case 'h':
            print_usage(argv[0]);
            return 0;
        case 'v':
            printf("%s version %s\n", SERVICE_NAME, SERVICE_VERSION);
            return 0;
        default:
            print_usage(argv[0]);
            return 1;
        }
    }

    /* Initialize the service */
    if (init_service() < 0) {
        return 1;
    }

    /* Set profile if requested */
    if (profile_type >= 0) {
        if (set_power_profile(profile_type) < 0) {
            cleanup_service();
            return 1;
        }
    }

    /* Show status if requested */
    if (show_status) {
        struct vr_power_profile profile;
        struct vr_battery_status status;

        if (get_power_profile(&profile) == 0) {
            printf("Power profile: %d\n", profile.type);
            printf("CPU: %u-%u MHz, governor: %s\n",
                   profile.cpu_freq_min / 1000, profile.cpu_freq_max / 1000,
                   profile.cpu_governor);
            printf("GPU: %u-%u MHz\n",
                   profile.gpu_freq_min / 1000, profile.gpu_freq_max / 1000);
            printf("NPU: %u-%u MHz\n",
                   profile.npu_freq_min / 1000, profile.npu_freq_max / 1000);
            printf("Display: brightness=%u, refresh=%u Hz\n",
                   profile.display_brightness, profile.display_refresh_rate);
            printf("WiFi power save: %s\n", profile.wifi_power_save ? "on" : "off");
            printf("Sensor rate: %u Hz\n", profile.sensor_rate);
        }

        if (get_battery_status(&status) == 0) {
            printf("Battery status: %d\n", status.status);
            printf("Capacity: %u%%\n", status.capacity);
            printf("Voltage: %u mV\n", status.voltage);
            printf("Current: %d mA\n", status.current);
            printf("Temperature: %.1f°C\n", status.temperature / 10.0);
            if (status.time_to_empty > 0)
                printf("Time to empty: %u minutes\n", status.time_to_empty);
            if (status.time_to_full > 0)
                printf("Time to full: %u minutes\n", status.time_to_full);
        }
    }

    /* If not daemon mode and no other actions, exit */
    if (!daemon_mode && profile_type < 0 && !show_status) {
        cleanup_service();
        return 0;
    }

    /* Daemonize if requested */
    if (daemon_mode) {
        printf("Starting %s in daemon mode...\n", SERVICE_NAME);
        
        /* Create threads */
        ret = pthread_create(&battery_thread, NULL, battery_monitor_thread, NULL);
        if (ret != 0) {
            fprintf(stderr, "Failed to create battery thread: %s\n", strerror(ret));
            cleanup_service();
            return 1;
        }

        ret = pthread_create(&thermal_thread, NULL, thermal_monitor_thread, NULL);
        if (ret != 0) {
            fprintf(stderr, "Failed to create thermal thread: %s\n", strerror(ret));
            g_running = 0;
            pthread_join(battery_thread, NULL);
            cleanup_service();
            return 1;
        }

        /* Wait for threads to exit */
        pthread_join(battery_thread, NULL);
        pthread_join(thermal_thread, NULL);
    }

    /* Cleanup */
    cleanup_service();
    return 0;
}
