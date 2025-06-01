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

#include <linux/module.h>
#include <linux/platform_device.h>
#include <linux/of.h>
#include <linux/power_supply.h>
#include <linux/thermal.h>
#include <linux/regulator/consumer.h>
#include <linux/cpufreq.h>
#include <linux/slab.h>
#include <linux/workqueue.h>
#include <linux/mutex.h>
#include <linux/sysfs.h>
#include <linux/fs.h>
#include <linux/miscdevice.h>
#include <linux/uaccess.h>

#include "orangepi_vr_power.h"

#define DRIVER_NAME "orangepi-vr-power"
#define DRIVER_DESC "Orange Pi CM5 VR Headset Power Management Driver"
#define DRIVER_VERSION "1.0.0"

/* Default power profiles */
static const struct vr_power_profile vr_power_profiles[] = {
    [VR_POWER_PROFILE_HIGH_PERFORMANCE] = {
        .type = VR_POWER_PROFILE_HIGH_PERFORMANCE,
        .cpu_freq_min = 1800000,
        .cpu_freq_max = 2400000,
        .cpu_governor = "performance",
        .gpu_freq_min = 800000,
        .gpu_freq_max = 1000000,
        .npu_freq_min = 800000,
        .npu_freq_max = 1000000,
        .display_brightness = 255,
        .display_refresh_rate = 90,
        .wifi_power_save = false,
        .sensor_rate = 1000,
    },
    [VR_POWER_PROFILE_BALANCED] = {
        .type = VR_POWER_PROFILE_BALANCED,
        .cpu_freq_min = 1200000,
        .cpu_freq_max = 2000000,
        .cpu_governor = "schedutil",
        .gpu_freq_min = 600000,
        .gpu_freq_max = 800000,
        .npu_freq_min = 600000,
        .npu_freq_max = 800000,
        .display_brightness = 200,
        .display_refresh_rate = 90,
        .wifi_power_save = false,
        .sensor_rate = 500,
    },
    [VR_POWER_PROFILE_POWER_SAVE] = {
        .type = VR_POWER_PROFILE_POWER_SAVE,
        .cpu_freq_min = 600000,
        .cpu_freq_max = 1500000,
        .cpu_governor = "powersave",
        .gpu_freq_min = 400000,
        .gpu_freq_max = 600000,
        .npu_freq_min = 400000,
        .npu_freq_max = 600000,
        .display_brightness = 150,
        .display_refresh_rate = 60,
        .wifi_power_save = true,
        .sensor_rate = 200,
    },
};

/* Default thermal configuration */
static const struct vr_thermal_config default_thermal_config = {
    .trip_points = {
        [VR_THERMAL_ZONE_CPU] = { 70000, 80000, 90000 },
        [VR_THERMAL_ZONE_GPU] = { 70000, 80000, 90000 },
        [VR_THERMAL_ZONE_NPU] = { 70000, 80000, 90000 },
        [VR_THERMAL_ZONE_BATTERY] = { 40000, 45000, 50000 },
        [VR_THERMAL_ZONE_AMBIENT] = { 35000, 40000, 45000 },
    },
    .hysteresis = {
        [VR_THERMAL_ZONE_CPU] = 5000,
        [VR_THERMAL_ZONE_GPU] = 5000,
        [VR_THERMAL_ZONE_NPU] = 5000,
        [VR_THERMAL_ZONE_BATTERY] = 3000,
        [VR_THERMAL_ZONE_AMBIENT] = 3000,
    },
};

/* Battery power supply properties */
static enum power_supply_property vr_battery_props[] = {
    POWER_SUPPLY_PROP_STATUS,
    POWER_SUPPLY_PROP_HEALTH,
    POWER_SUPPLY_PROP_PRESENT,
    POWER_SUPPLY_PROP_TECHNOLOGY,
    POWER_SUPPLY_PROP_CAPACITY,
    POWER_SUPPLY_PROP_VOLTAGE_NOW,
    POWER_SUPPLY_PROP_CURRENT_NOW,
    POWER_SUPPLY_PROP_TEMP,
    POWER_SUPPLY_PROP_CHARGE_COUNTER,
};

/* Charger power supply properties */
static enum power_supply_property vr_charger_props[] = {
    POWER_SUPPLY_PROP_ONLINE,
    POWER_SUPPLY_PROP_TYPE,
    POWER_SUPPLY_PROP_CURRENT_MAX,
    POWER_SUPPLY_PROP_VOLTAGE_MAX,
};

/* Forward declarations */
static int vr_power_open(struct inode *inode, struct file *file);
static int vr_power_release(struct inode *inode, struct file *file);
static long vr_power_ioctl(struct file *file, unsigned int cmd, unsigned long arg);

/* File operations for the device */
static const struct file_operations vr_power_fops = {
    .owner = THIS_MODULE,
    .open = vr_power_open,
    .release = vr_power_release,
    .unlocked_ioctl = vr_power_ioctl,
};

/* Miscdevice for the driver */
static struct miscdevice vr_power_miscdev = {
    .minor = MISC_DYNAMIC_MINOR,
    .name = DRIVER_NAME,
    .fops = &vr_power_fops,
};

/* Battery power supply operations */
static int vr_battery_get_property(struct power_supply *psy,
                                  enum power_supply_property psp,
                                  union power_supply_propval *val)
{
    struct vr_power_data *data = power_supply_get_drvdata(psy);
    int ret = 0;

    mutex_lock(&data->lock);

    switch (psp) {
    case POWER_SUPPLY_PROP_STATUS:
        switch (data->battery_status.status) {
        case VR_BATTERY_STATUS_CHARGING:
            val->intval = POWER_SUPPLY_STATUS_CHARGING;
            break;
        case VR_BATTERY_STATUS_DISCHARGING:
            val->intval = POWER_SUPPLY_STATUS_DISCHARGING;
            break;
        case VR_BATTERY_STATUS_FULL:
            val->intval = POWER_SUPPLY_STATUS_FULL;
            break;
        default:
            val->intval = POWER_SUPPLY_STATUS_UNKNOWN;
            break;
        }
        break;
    case POWER_SUPPLY_PROP_HEALTH:
        val->intval = POWER_SUPPLY_HEALTH_GOOD;
        break;
    case POWER_SUPPLY_PROP_PRESENT:
        val->intval = 1;
        break;
    case POWER_SUPPLY_PROP_TECHNOLOGY:
        val->intval = POWER_SUPPLY_TECHNOLOGY_LION;
        break;
    case POWER_SUPPLY_PROP_CAPACITY:
        val->intval = data->battery_status.capacity;
        break;
    case POWER_SUPPLY_PROP_VOLTAGE_NOW:
        val->intval = data->battery_status.voltage * 1000; /* Convert to μV */
        break;
    case POWER_SUPPLY_PROP_CURRENT_NOW:
        val->intval = data->battery_status.current * 1000; /* Convert to μA */
        break;
    case POWER_SUPPLY_PROP_TEMP:
        val->intval = data->battery_status.temperature;
        break;
    case POWER_SUPPLY_PROP_CHARGE_COUNTER:
        /* Not implemented yet */
        val->intval = 0;
        break;
    default:
        ret = -EINVAL;
        break;
    }

    mutex_unlock(&data->lock);
    return ret;
}

/* Charger power supply operations */
static int vr_charger_get_property(struct power_supply *psy,
                                  enum power_supply_property psp,
                                  union power_supply_propval *val)
{
    struct vr_power_data *data = power_supply_get_drvdata(psy);
    int ret = 0;

    mutex_lock(&data->lock);

    switch (psp) {
    case POWER_SUPPLY_PROP_ONLINE:
        val->intval = (data->battery_status.charger_type != VR_CHARGER_TYPE_NONE);
        break;
    case POWER_SUPPLY_PROP_TYPE:
        switch (data->battery_status.charger_type) {
        case VR_CHARGER_TYPE_USB:
            val->intval = POWER_SUPPLY_TYPE_USB;
            break;
        case VR_CHARGER_TYPE_AC:
            val->intval = POWER_SUPPLY_TYPE_MAINS;
            break;
        case VR_CHARGER_TYPE_WIRELESS:
            val->intval = POWER_SUPPLY_TYPE_WIRELESS;
            break;
        default:
            val->intval = POWER_SUPPLY_TYPE_UNKNOWN;
            break;
        }
        break;
    case POWER_SUPPLY_PROP_CURRENT_MAX:
        /* Not implemented yet */
        val->intval = 2000000; /* 2A in μA */
        break;
    case POWER_SUPPLY_PROP_VOLTAGE_MAX:
        /* Not implemented yet */
        val->intval = 5000000; /* 5V in μV */
        break;
    default:
        ret = -EINVAL;
        break;
    }

    mutex_unlock(&data->lock);
    return ret;
}

/* Power supply descriptions */
static const struct power_supply_desc vr_battery_desc = {
    .name = "vr_battery",
    .type = POWER_SUPPLY_TYPE_BATTERY,
    .properties = vr_battery_props,
    .num_properties = ARRAY_SIZE(vr_battery_props),
    .get_property = vr_battery_get_property,
};

static const struct power_supply_desc vr_charger_desc = {
    .name = "vr_charger",
    .type = POWER_SUPPLY_TYPE_UNKNOWN,
    .properties = vr_charger_props,
    .num_properties = ARRAY_SIZE(vr_charger_props),
    .get_property = vr_charger_get_property,
};

/* Battery work function */
static void vr_power_battery_work(struct work_struct *work)
{
    struct vr_power_data *data = container_of(work, struct vr_power_data,
                                             battery_work.work);
    
    vr_power_update_battery_status(data);
    
    /* Schedule next update */
    queue_delayed_work(data->pm_wq, &data->battery_work,
                      msecs_to_jiffies(1000)); /* Update every second */
}

/* Thermal work function */
static void vr_power_thermal_work(struct work_struct *work)
{
    struct vr_power_data *data = container_of(work, struct vr_power_data,
                                             thermal_work.work);
    
    vr_power_update_thermal_status(data);
    
    /* Schedule next update */
    queue_delayed_work(data->pm_wq, &data->thermal_work,
                      msecs_to_jiffies(1000)); /* Update every second */
}

/* File operations implementations */
static int vr_power_open(struct inode *inode, struct file *file)
{
    return 0;
}

static int vr_power_release(struct inode *inode, struct file *file)
{
    return 0;
}

static long vr_power_ioctl(struct file *file, unsigned int cmd, unsigned long arg)
{
    struct vr_power_data *data = platform_get_drvdata(
        container_of(file->private_data, struct platform_device, dev));
    void __user *argp = (void __user *)arg;
    int ret = 0;

    if (!data)
        return -ENODEV;

    mutex_lock(&data->lock);

    switch (cmd) {
    case VR_POWER_IOCTL_SET_PROFILE:
        {
            struct vr_power_profile profile;
            if (copy_from_user(&profile, argp, sizeof(profile))) {
                ret = -EFAULT;
                break;
            }
            ret = vr_power_set_profile(data, &profile);
        }
        break;
    case VR_POWER_IOCTL_GET_PROFILE:
        {
            struct vr_power_profile profile;
            ret = vr_power_get_profile(data, &profile);
            if (ret)
                break;
            if (copy_to_user(argp, &profile, sizeof(profile)))
                ret = -EFAULT;
        }
        break;
    case VR_POWER_IOCTL_SET_THERMAL:
        /* Not implemented yet */
        ret = -ENOSYS;
        break;
    case VR_POWER_IOCTL_GET_THERMAL:
        /* Not implemented yet */
        ret = -ENOSYS;
        break;
    case VR_POWER_IOCTL_SET_BATTERY:
        /* Not implemented yet */
        ret = -ENOSYS;
        break;
    case VR_POWER_IOCTL_GET_BATTERY:
        {
            if (copy_to_user(argp, &data->battery_status, sizeof(data->battery_status)))
                ret = -EFAULT;
        }
        break;
    default:
        ret = -ENOTTY;
        break;
    }

    mutex_unlock(&data->lock);
    return ret;
}

/* Initialize battery management */
int vr_power_init_battery(struct vr_power_data *data)
{
    struct power_supply_config psy_cfg = {};
    int ret;

    /* Initialize battery status */
    data->battery_status.status = VR_BATTERY_STATUS_DISCHARGING;
    data->battery_status.charger_type = VR_CHARGER_TYPE_NONE;
    data->battery_status.capacity = 100;
    data->battery_status.voltage = 4200;
    data->battery_status.current = -500;
    data->battery_status.temperature = 250;
    data->battery_status.time_to_empty = 180;
    data->battery_status.time_to_full = 0;

    /* Register battery power supply */
    psy_cfg.drv_data = data;
    data->battery_psy = power_supply_register(data->dev, &vr_battery_desc, &psy_cfg);
    if (IS_ERR(data->battery_psy)) {
        ret = PTR_ERR(data->battery_psy);
        dev_err(data->dev, "Failed to register battery power supply: %d\n", ret);
        return ret;
    }

    /* Register charger power supply */
    data->charger_psy = power_supply_register(data->dev, &vr_charger_desc, &psy_cfg);
    if (IS_ERR(data->charger_psy)) {
        ret = PTR_ERR(data->charger_psy);
        dev_err(data->dev, "Failed to register charger power supply: %d\n", ret);
        power_supply_unregister(data->battery_psy);
        return ret;
    }

    /* Initialize and schedule battery work */
    INIT_DELAYED_WORK(&data->battery_work, vr_power_battery_work);
    queue_delayed_work(data->pm_wq, &data->battery_work, 0);

    return 0;
}

/* Cleanup battery management */
void vr_power_exit_battery(struct vr_power_data *data)
{
    cancel_delayed_work_sync(&data->battery_work);
    power_supply_unregister(data->charger_psy);
    power_supply_unregister(data->battery_psy);
}

/* Update battery status */
int vr_power_update_battery_status(struct vr_power_data *data)
{
    /* In a real driver, this would read from hardware */
    /* For now, we'll just simulate battery discharge */
    
    mutex_lock(&data->lock);
    
    /* Simulate battery discharge */
    if (data->battery_status.status == VR_BATTERY_STATUS_DISCHARGING) {
        if (data->battery_status.capacity > 0)
            data->battery_status.capacity--;
    } else if (data->battery_status.status == VR_BATTERY_STATUS_CHARGING) {
        if (data->battery_status.capacity < 100)
            data->battery_status.capacity++;
        else
            data->battery_status.status = VR_BATTERY_STATUS_FULL;
    }
    
    /* Update time estimates */
    if (data->battery_status.status == VR_BATTERY_STATUS_DISCHARGING) {
        data->battery_status.time_to_empty = data->battery_status.capacity * 2;
        data->battery_status.time_to_full = 0;
    } else if (data->battery_status.status == VR_BATTERY_STATUS_CHARGING) {
        data->battery_status.time_to_empty = 0;
        data->battery_status.time_to_full = (100 - data->battery_status.capacity) * 2;
    } else {
        data->battery_status.time_to_empty = 0;
        data->battery_status.time_to_full = 0;
    }
    
    mutex_unlock(&data->lock);
    
    /* Notify power supply change */
    power_supply_changed(data->battery_psy);
    
    return 0;
}

/* Initialize thermal management */
int vr_power_init_thermal(struct vr_power_data *data)
{
    int i;
    
    /* Initialize thermal status */
    for (i = 0; i < VR_THERMAL_ZONE_COUNT; i++) {
        data->thermal_status.status[i] = VR_THERMAL_STATUS_NORMAL;
        data->thermal_status.temperature[i] = 250; /* 25.0°C */
    }
    
    /* In a real driver, we would register with the thermal framework */
    /* For now, we'll just simulate thermal monitoring */
    
    /* Initialize and schedule thermal work */
    INIT_DELAYED_WORK(&data->thermal_work, vr_power_thermal_work);
    queue_delayed_work(data->pm_wq, &data->thermal_work, 0);
    
    return 0;
}

/* Cleanup thermal management */
void vr_power_exit_thermal(struct vr_power_data *data)
{
    cancel_delayed_work_sync(&data->thermal_work);
}

/* Update thermal status */
int vr_power_update_thermal_status(struct vr_power_data *data)
{
    int i;
    bool throttling_changed = false;
    
    mutex_lock(&data->lock);
    
    /* In a real driver, this would read from hardware */
    /* For now, we'll just simulate thermal monitoring */
    
    /* Simulate temperature changes based on current profile */
    for (i = 0; i < VR_THERMAL_ZONE_COUNT; i++) {
        enum vr_thermal_status old_status = data->thermal_status.status[i];
        int temp = data->thermal_status.temperature[i];
        
        /* Adjust temperature based on profile */
        switch (data->current_profile.type) {
        case VR_POWER_PROFILE_HIGH_PERFORMANCE:
            /* Temperature increases in high performance mode */
            temp += 10;
            if (temp > 800) /* Max 80.0°C */
                temp = 800;
            break;
        case VR_POWER_PROFILE_BALANCED:
            /* Temperature stable in balanced mode */
            break;
        case VR_POWER_PROFILE_POWER_SAVE:
            /* Temperature decreases in power save mode */
            temp -= 10;
            if (temp < 250) /* Min 25.0°C */
                temp = 250;
            break;
        default:
            break;
        }
        
        data->thermal_status.temperature[i] = temp;
        
        /* Update thermal status */
        if (temp >= default_thermal_config.trip_points[i][2]) {
            data->thermal_status.status[i] = VR_THERMAL_STATUS_EMERGENCY;
        } else if (temp >= default_thermal_config.trip_points[i][1]) {
            data->thermal_status.status[i] = VR_THERMAL_STATUS_CRITICAL;
        } else if (temp >= default_thermal_config.trip_points[i][0]) {
            data->thermal_status.status[i] = VR_THERMAL_STATUS_WARNING;
        } else {
            data->thermal_status.status[i] = VR_THERMAL_STATUS_NORMAL;
        }
        
        if (old_status != data->thermal_status.status[i]) {
            throttling_changed = true;
            vr_power_handle_thermal_event(data, i);
        }
    }
    
    mutex_unlock(&data->lock);
    
    return 0;
}

/* Handle thermal event */
int vr_power_handle_thermal_event(struct vr_power_data *data, enum vr_thermal_zone zone)
{
    /* In a real driver, this would implement thermal throttling */
    /* For now, we'll just log the event */
    
    dev_info(data->dev, "Thermal event: zone %d, status %d, temp %d\n",
             zone, data->thermal_status.status[zone],
             data->thermal_status.temperature[zone]);
    
    /* If emergency, switch to power save mode */
    if (data->thermal_status.status[zone] == VR_THERMAL_STATUS_EMERGENCY) {
        struct vr_power_profile profile = vr_power_profiles[VR_POWER_PROFILE_POWER_SAVE];
        vr_power_set_profile(data, &profile);
    }
    
    return 0;
}

/* Initialize power profile */
int vr_power_init_profile(struct vr_power_data *data)
{
    /* Start with balanced profile */
    data->current_profile = vr_power_profiles[VR_POWER_PROFILE_BALANCED];
    
    /* Apply initial profile */
    return vr_power_set_profile(data, &data->current_profile);
}

/* Cleanup power profile */
void vr_power_exit_profile(struct vr_power_data *data)
{
    /* Nothing to do */
}

/* Set power profile */
int vr_power_set_profile(struct vr_power_data *data, struct vr_power_profile *profile)
{
    int ret = 0;
    
    mutex_lock(&data->lock);
    
    /* Save the new profile */
    data->current_profile = *profile;
    
    /* Apply CPU settings */
    ret = vr_power_set_cpu_freq(data, profile->cpu_freq_min, profile->cpu_freq_max);
    if (ret) {
        dev_err(data->dev, "Failed to set CPU frequency: %d\n", ret);
        goto out;
    }
    
    /* Apply GPU settings */
    ret = vr_power_set_gpu_freq(data, profile->gpu_freq_min, profile->gpu_freq_max);
    if (ret) {
        dev_err(data->dev, "Failed to set GPU frequency: %d\n", ret);
        goto out;
    }
    
    /* Apply NPU settings */
    ret = vr_power_set_npu_freq(data, profile->npu_freq_min, profile->npu_freq_max);
    if (ret) {
        dev_err(data->dev, "Failed to set NPU frequency: %d\n", ret);
        goto out;
    }
    
    /* In a real driver, we would also apply display and other settings */
    
    dev_info(data->dev, "Power profile set to %d\n", profile->type);
    
out:
    mutex_unlock(&data->lock);
    return ret;
}

/* Get power profile */
int vr_power_get_profile(struct vr_power_data *data, struct vr_power_profile *profile)
{
    mutex_lock(&data->lock);
    *profile = data->current_profile;
    mutex_unlock(&data->lock);
    
    return 0;
}

/* Initialize DVFS */
int vr_power_init_dvfs(struct vr_power_data *data)
{
    /* In a real driver, this would initialize regulators and clocks */
    /* For now, we'll just simulate DVFS */
    
    return 0;
}

/* Cleanup DVFS */
void vr_power_exit_dvfs(struct vr_power_data *data)
{
    /* Nothing to do */
}

/* Set CPU frequency */
int vr_power_set_cpu_freq(struct vr_power_data *data, unsigned int min, unsigned int max)
{
    /* In a real driver, this would set CPU frequency */
    /* For now, we'll just log the request */
    
    dev_info(data->dev, "Setting CPU frequency: min=%u, max=%u\n", min, max);
    
    return 0;
}

/* Set GPU frequency */
int vr_power_set_gpu_freq(struct vr_power_data *data, unsigned int min, unsigned int max)
{
    /* In a real driver, this would set GPU frequency */
    /* For now, we'll just log the request */
    
    dev_info(data->dev, "Setting GPU frequency: min=%u, max=%u\n", min, max);
    
    return 0;
}

/* Set NPU frequency */
int vr_power_set_npu_freq(struct vr_power_data *data, unsigned int min, unsigned int max)
{
    /* In a real driver, this would set NPU frequency */
    /* For now, we'll just log the request */
    
    dev_info(data->dev, "Setting NPU frequency: min=%u, max=%u\n", min, max);
    
    return 0;
}

/* Sysfs attribute for power profile */
static ssize_t power_profile_show(struct device *dev,
                                 struct device_attribute *attr, char *buf)
{
    struct vr_power_data *data = dev_get_drvdata(dev);
    
    return sprintf(buf, "%d\n", data->current_profile.type);
}

static ssize_t power_profile_store(struct device *dev,
                                  struct device_attribute *attr,
                                  const char *buf, size_t count)
{
    struct vr_power_data *data = dev_get_drvdata(dev);
    int profile_type;
    int ret;
    
    ret = kstrtoint(buf, 10, &profile_type);
    if (ret)
        return ret;
    
    if (profile_type < 0 || profile_type >= VR_POWER_PROFILE_CUSTOM)
        return -EINVAL;
    
    ret = vr_power_set_profile(data, &vr_power_profiles[profile_type]);
    if (ret)
        return ret;
    
    return count;
}

static DEVICE_ATTR_RW(power_profile);

/* Sysfs attribute for battery status */
static ssize_t battery_status_show(struct device *dev,
                                  struct device_attribute *attr, char *buf)
{
    struct vr_power_data *data = dev_get_drvdata(dev);
    
    return sprintf(buf, "status=%d capacity=%d voltage=%d current=%d temp=%d\n",
                  data->battery_status.status,
                  data->battery_status.capacity,
                  data->battery_status.voltage,
                  data->battery_status.current,
                  data->battery_status.temperature);
}

static DEVICE_ATTR_RO(battery_status);

/* Sysfs attribute for thermal status */
static ssize_t thermal_status_show(struct device *dev,
                                  struct device_attribute *attr, char *buf)
{
    struct vr_power_data *data = dev_get_drvdata(dev);
    int i;
    int len = 0;
    
    for (i = 0; i < VR_THERMAL_ZONE_COUNT; i++) {
        len += sprintf(buf + len, "zone=%d status=%d temp=%d\n",
                      i, data->thermal_status.status[i],
                      data->thermal_status.temperature[i]);
    }
    
    return len;
}

static DEVICE_ATTR_RO(thermal_status);

/* Initialize sysfs interface */
int vr_power_init_sysfs(struct vr_power_data *data)
{
    int ret;
    
    ret = device_create_file(data->dev, &dev_attr_power_profile);
    if (ret)
        return ret;
    
    ret = device_create_file(data->dev, &dev_attr_battery_status);
    if (ret)
        goto err_battery;
    
    ret = device_create_file(data->dev, &dev_attr_thermal_status);
    if (ret)
        goto err_thermal;
    
    return 0;
    
err_thermal:
    device_remove_file(data->dev, &dev_attr_battery_status);
err_battery:
    device_remove_file(data->dev, &dev_attr_power_profile);
    return ret;
}

/* Cleanup sysfs interface */
void vr_power_exit_sysfs(struct vr_power_data *data)
{
    device_remove_file(data->dev, &dev_attr_thermal_status);
    device_remove_file(data->dev, &dev_attr_battery_status);
    device_remove_file(data->dev, &dev_attr_power_profile);
}

/* Probe function */
static int vr_power_probe(struct platform_device *pdev)
{
    struct vr_power_data *data;
    int ret;
    
    /* Allocate driver data */
    data = devm_kzalloc(&pdev->dev, sizeof(*data), GFP_KERNEL);
    if (!data)
        return -ENOMEM;
    
    data->dev = &pdev->dev;
    mutex_init(&data->lock);
    
    /* Create workqueue */
    data->pm_wq = create_singlethread_workqueue("vr_power_wq");
    if (!data->pm_wq) {
        dev_err(&pdev->dev, "Failed to create workqueue\n");
        return -ENOMEM;
    }
    
    /* Register misc device */
    ret = misc_register(&vr_power_miscdev);
    if (ret) {
        dev_err(&pdev->dev, "Failed to register misc device: %d\n", ret);
        goto err_misc;
    }
    
    /* Initialize components */
    ret = vr_power_init_battery(data);
    if (ret) {
        dev_err(&pdev->dev, "Failed to initialize battery: %d\n", ret);
        goto err_battery;
    }
    
    ret = vr_power_init_thermal(data);
    if (ret) {
        dev_err(&pdev->dev, "Failed to initialize thermal: %d\n", ret);
        goto err_thermal;
    }
    
    ret = vr_power_init_dvfs(data);
    if (ret) {
        dev_err(&pdev->dev, "Failed to initialize DVFS: %d\n", ret);
        goto err_dvfs;
    }
    
    ret = vr_power_init_profile(data);
    if (ret) {
        dev_err(&pdev->dev, "Failed to initialize profile: %d\n", ret);
        goto err_profile;
    }
    
    ret = vr_power_init_sysfs(data);
    if (ret) {
        dev_err(&pdev->dev, "Failed to initialize sysfs: %d\n", ret);
        goto err_sysfs;
    }
    
    /* Save driver data */
    platform_set_drvdata(pdev, data);
    
    dev_info(&pdev->dev, "Orange Pi CM5 VR Power Management Driver initialized\n");
    
    return 0;
    
err_sysfs:
    vr_power_exit_profile(data);
err_profile:
    vr_power_exit_dvfs(data);
err_dvfs:
    vr_power_exit_thermal(data);
err_thermal:
    vr_power_exit_battery(data);
err_battery:
    misc_deregister(&vr_power_miscdev);
err_misc:
    destroy_workqueue(data->pm_wq);
    return ret;
}

/* Remove function */
static int vr_power_remove(struct platform_device *pdev)
{
    struct vr_power_data *data = platform_get_drvdata(pdev);
    
    vr_power_exit_sysfs(data);
    vr_power_exit_profile(data);
    vr_power_exit_dvfs(data);
    vr_power_exit_thermal(data);
    vr_power_exit_battery(data);
    misc_deregister(&vr_power_miscdev);
    destroy_workqueue(data->pm_wq);
    
    return 0;
}

/* Device tree match table */
static const struct of_device_id vr_power_of_match[] = {
    { .compatible = "orangepi,cm5-vr-power", },
    { /* sentinel */ }
};
MODULE_DEVICE_TABLE(of, vr_power_of_match);

/* Platform driver structure */
static struct platform_driver vr_power_driver = {
    .probe = vr_power_probe,
    .remove = vr_power_remove,
    .driver = {
        .name = DRIVER_NAME,
        .of_match_table = vr_power_of_match,
    },
};

/* Module initialization */
static int __init vr_power_init(void)
{
    return platform_driver_register(&vr_power_driver);
}

/* Module cleanup */
static void __exit vr_power_exit(void)
{
    platform_driver_unregister(&vr_power_driver);
}

module_init(vr_power_init);
module_exit(vr_power_exit);

MODULE_AUTHOR("VR Headset Project");
MODULE_DESCRIPTION(DRIVER_DESC);
MODULE_LICENSE("GPL");
MODULE_VERSION(DRIVER_VERSION);
