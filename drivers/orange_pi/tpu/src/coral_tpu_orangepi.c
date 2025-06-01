/*
 * Coral TPU Driver for Orange Pi CM5 VR
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
#include <linux/of_address.h>
#include <linux/of_irq.h>
#include <linux/dma-mapping.h>
#include <linux/delay.h>
#include <linux/pm_runtime.h>
#include <linux/slab.h>
#include <linux/interrupt.h>
#include <linux/io.h>
#include <linux/uaccess.h>
#include <linux/fs.h>
#include <linux/cdev.h>
#include <linux/miscdevice.h>

// Include the original Coral TPU driver headers
#include "gasket.h"
#include "apex.h"
#include "apex_driver.h"

/* Coral TPU Register Map for Orange Pi CM5 VR */
#define CORAL_TPU_CONTROL                0x0000
#define CORAL_TPU_STATUS                 0x0010
#define CORAL_TPU_DMA_CONFIG             0x0020
#define CORAL_TPU_LATENCY_CONFIG         0x0030
#define CORAL_TPU_POWER_CONFIG           0x0040
#define CORAL_TPU_BUFFER_CONFIG          0x0050
#define CORAL_TPU_VR_MODE_CONFIG         0x0060

/* Coral TPU Register Values for Orange Pi CM5 VR */
#define CORAL_TPU_CONTROL_ENABLE         0x00000001
#define CORAL_TPU_DMA_CONFIG_ZEROCOPY    0x00000001
#define CORAL_TPU_POWER_CONFIG_PERF      0x00000001
#define CORAL_TPU_VR_MODE_ENABLE         0x00000001

/* VR-specific TPU configuration */
struct coral_tpu_vr_config {
    bool vr_mode_enabled;
    u32 latency_target_ms;
    u32 buffer_size_kb;
    bool zero_copy_enabled;
    bool performance_mode;
    u8 inference_priority;
};

/* Coral TPU device structure with Orange Pi CM5 extensions */
struct coral_tpu_orangepi_device {
    struct apex_driver_data base_dev;
    struct coral_tpu_vr_config vr_config;
    bool is_orangepi_cm5;
    void __iomem *vr_regs;
    struct dma_buf *shared_buffer;
    dma_addr_t shared_dma_addr;
    void *shared_cpu_addr;
    size_t shared_size;
};

/* Forward declarations */
static int coral_tpu_write_reg(struct apex_driver_data *dev, u32 reg, u32 val);
static int coral_tpu_read_reg(struct apex_driver_data *dev, u32 reg, u32 *val);

/* Orange Pi CM5 specific configuration */
static int coral_tpu_configure_orangepi_cm5_vr(struct apex_driver_data *dev)
{
    struct device *device = dev->dev;
    struct coral_tpu_orangepi_device *orangepi_dev = 
        container_of(dev, struct coral_tpu_orangepi_device, base_dev);
    struct coral_tpu_vr_config *vr_config = &orangepi_dev->vr_config;
    int ret;

    dev_info(device, "Configuring Coral TPU for Orange Pi CM5 VR\n");

    /* Parse VR-specific device tree properties */
    if (of_property_read_bool(device->of_node, "vr,mode-enabled"))
        vr_config->vr_mode_enabled = true;

    of_property_read_u32(device->of_node, "vr,latency-target-ms", &vr_config->latency_target_ms);
    of_property_read_u32(device->of_node, "vr,buffer-size-kb", &vr_config->buffer_size_kb);
    
    if (of_property_read_bool(device->of_node, "vr,zero-copy-enabled"))
        vr_config->zero_copy_enabled = true;
        
    if (of_property_read_bool(device->of_node, "vr,performance-mode"))
        vr_config->performance_mode = true;
        
    of_property_read_u8(device->of_node, "vr,inference-priority", &vr_config->inference_priority);

    /* Set default values if not specified */
    if (!vr_config->latency_target_ms)
        vr_config->latency_target_ms = 5; // 5ms target latency
        
    if (!vr_config->buffer_size_kb)
        vr_config->buffer_size_kb = 4096; // 4MB buffer size
        
    if (!vr_config->inference_priority)
        vr_config->inference_priority = 90; // High priority (0-99)

    /* Configure TPU for VR mode */
    if (vr_config->vr_mode_enabled) {
        /* Enable VR mode */
        ret = coral_tpu_write_reg(dev, CORAL_TPU_VR_MODE_CONFIG, CORAL_TPU_VR_MODE_ENABLE);
        if (ret)
            return ret;

        /* Configure latency target */
        ret = coral_tpu_write_reg(dev, CORAL_TPU_LATENCY_CONFIG, vr_config->latency_target_ms);
        if (ret)
            return ret;

        /* Configure DMA for zero-copy if enabled */
        if (vr_config->zero_copy_enabled) {
            ret = coral_tpu_write_reg(dev, CORAL_TPU_DMA_CONFIG, CORAL_TPU_DMA_CONFIG_ZEROCOPY);
            if (ret)
                return ret;
                
            /* Allocate shared buffer for zero-copy operations */
            orangepi_dev->shared_size = vr_config->buffer_size_kb * 1024;
            orangepi_dev->shared_cpu_addr = dma_alloc_coherent(device, 
                                                              orangepi_dev->shared_size,
                                                              &orangepi_dev->shared_dma_addr,
                                                              GFP_KERNEL);
            if (!orangepi_dev->shared_cpu_addr) {
                dev_err(device, "Failed to allocate DMA buffer for zero-copy\n");
                return -ENOMEM;
            }
            
            dev_info(device, "Allocated %zu bytes for zero-copy buffer at DMA addr 0x%llx\n",
                     orangepi_dev->shared_size, (unsigned long long)orangepi_dev->shared_dma_addr);
        }

        /* Configure power mode */
        if (vr_config->performance_mode) {
            ret = coral_tpu_write_reg(dev, CORAL_TPU_POWER_CONFIG, CORAL_TPU_POWER_CONFIG_PERF);
            if (ret)
                return ret;
        }

        /* Configure buffer size */
        ret = coral_tpu_write_reg(dev, CORAL_TPU_BUFFER_CONFIG, vr_config->buffer_size_kb);
        if (ret)
            return ret;
    }

    /* Enable TPU */
    ret = coral_tpu_write_reg(dev, CORAL_TPU_CONTROL, CORAL_TPU_CONTROL_ENABLE);
    if (ret)
        return ret;

    /* Store Orange Pi device information */
    orangepi_dev->is_orangepi_cm5 = true;

    dev_info(device, "Coral TPU configured for Orange Pi CM5 VR: %s, latency=%dms, buffer=%dKB, zero-copy=%s, perf-mode=%s, priority=%d\n",
             vr_config->vr_mode_enabled ? "VR-mode" : "normal-mode",
             vr_config->latency_target_ms,
             vr_config->buffer_size_kb,
             vr_config->zero_copy_enabled ? "enabled" : "disabled",
             vr_config->performance_mode ? "enabled" : "disabled",
             vr_config->inference_priority);

    return 0;
}

/* Update probe function to detect Orange Pi CM5 */
static int coral_tpu_probe_orangepi(struct platform_device *pdev)
{
    struct device *dev = &pdev->dev;
    struct coral_tpu_orangepi_device *orangepi_dev;
    int ret;

    dev_info(dev, "Probing Coral TPU for Orange Pi CM5\n");

    /* Allocate device structure */
    orangepi_dev = devm_kzalloc(dev, sizeof(*orangepi_dev), GFP_KERNEL);
    if (!orangepi_dev)
        return -ENOMEM;

    /* Initialize base device */
    ret = apex_driver_probe(pdev, &orangepi_dev->base_dev);
    if (ret)
        return ret;

    /* Check if this is an Orange Pi CM5 device */
    if (of_device_is_compatible(dev->of_node, "orangepi,coral-tpu-vr")) {
        dev_info(dev, "Detected Orange Pi CM5 VR TPU\n");
        
        /* Map VR-specific registers */
        orangepi_dev->vr_regs = devm_platform_ioremap_resource(pdev, 1);
        if (IS_ERR(orangepi_dev->vr_regs)) {
            dev_err(dev, "Failed to map VR registers\n");
            return PTR_ERR(orangepi_dev->vr_regs);
        }
        
        /* Apply Orange Pi CM5 specific configuration */
        ret = coral_tpu_configure_orangepi_cm5_vr(&orangepi_dev->base_dev);
        if (ret) {
            dev_err(dev, "Failed to configure for Orange Pi CM5: %d\n", ret);
            return ret;
        }
    }

    return 0;
}

/* Update remove function to clean up Orange Pi CM5 resources */
static int coral_tpu_remove_orangepi(struct platform_device *pdev)
{
    struct apex_driver_data *dev = platform_get_drvdata(pdev);
    struct coral_tpu_orangepi_device *orangepi_dev = 
        container_of(dev, struct coral_tpu_orangepi_device, base_dev);
        
    /* Free zero-copy buffer if allocated */
    if (orangepi_dev->shared_cpu_addr) {
        dma_free_coherent(&pdev->dev, orangepi_dev->shared_size,
                         orangepi_dev->shared_cpu_addr, orangepi_dev->shared_dma_addr);
    }
    
    /* Call original remove function */
    return apex_driver_remove(pdev);
}

/* Update the compatible strings to include Orange Pi variant */
static const struct of_device_id coral_tpu_of_match_orangepi[] = {
    { .compatible = "google,apex" },
    { .compatible = "orangepi,coral-tpu-vr" },
    { /* sentinel */ }
};
MODULE_DEVICE_TABLE(of, coral_tpu_of_match_orangepi);

/* Update the platform_driver structure */
static struct platform_driver coral_tpu_platform_driver_orangepi = {
    .probe = coral_tpu_probe_orangepi,
    .remove = coral_tpu_remove_orangepi,
    .driver = {
        .name = "apex-orangepi",
        .of_match_table = coral_tpu_of_match_orangepi,
    },
};

module_platform_driver(coral_tpu_platform_driver_orangepi);

MODULE_DESCRIPTION("Coral TPU Driver for Orange Pi CM5 VR");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
