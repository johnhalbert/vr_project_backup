# Orange Pi CM5 Coral TPU Driver Adaptation

This file contains the necessary adaptations to the Coral TPU driver for the Orange Pi CM5 platform.

```c
// drivers/misc/coral/gasket/orangepi_vr_optimizations.c

#include <linux/module.h>
#include <linux/pci.h>
#include <linux/of.h>
#include <linux/of_device.h>
#include <linux/delay.h>
#include <linux/pm_runtime.h>
#include <linux/dma-mapping.h>
#include <linux/iommu.h>
#include <linux/interrupt.h>
#include <linux/fs.h>
#include <linux/uaccess.h>
#include <linux/slab.h>

// Include the original Coral TPU driver headers
#include "gasket_core.h"
#include "apex.h"
#include "apex_internal.h"

// VR-specific TPU configuration
struct coral_vr_config {
    bool vr_mode_enabled;
    u32 vr_dma_buffer_size;
    u32 vr_inference_priority;
    u32 vr_power_mode;
    u32 vr_latency_target_us;
    bool zero_copy_enabled;
};

// Orange Pi CM5 specific configuration
static int coral_configure_orangepi_cm5_vr(struct gasket_dev *gasket_dev)
{
    struct device *dev = gasket_dev->dev;
    struct coral_vr_config *vr_config;
    int ret;

    dev_info(dev, "Configuring Coral TPU for Orange Pi CM5 VR\n");

    // Allocate VR configuration
    vr_config = devm_kzalloc(dev, sizeof(*vr_config), GFP_KERNEL);
    if (!vr_config)
        return -ENOMEM;

    // Parse VR-specific device tree properties
    if (of_property_read_bool(dev->of_node, "vr,mode-enabled"))
        vr_config->vr_mode_enabled = true;

    if (of_property_read_bool(dev->of_node, "vr,zero-copy-enabled"))
        vr_config->zero_copy_enabled = true;

    of_property_read_u32(dev->of_node, "vr,dma-buffer-size", &vr_config->vr_dma_buffer_size);
    of_property_read_u32(dev->of_node, "vr,inference-priority", &vr_config->vr_inference_priority);
    of_property_read_u32(dev->of_node, "vr,power-mode", &vr_config->vr_power_mode);
    of_property_read_u32(dev->of_node, "vr,latency-target-us", &vr_config->vr_latency_target_us);

    // Set default values if not specified
    if (!vr_config->vr_dma_buffer_size)
        vr_config->vr_dma_buffer_size = 16 * 1024 * 1024; // 16MB DMA buffer

    if (!vr_config->vr_inference_priority)
        vr_config->vr_inference_priority = 2; // High priority

    if (!vr_config->vr_power_mode)
        vr_config->vr_power_mode = 1; // Performance mode

    if (!vr_config->vr_latency_target_us)
        vr_config->vr_latency_target_us = 5000; // 5ms target latency

    // Configure TPU for VR mode
    if (vr_config->vr_mode_enabled) {
        // Configure DMA buffer size
        ret = gasket_write_64(gasket_dev, vr_config->vr_dma_buffer_size, 
                             APEX_BAR_INDEX, APEX_DMA_BUFFER_SIZE_OFFSET);
        if (ret)
            return ret;

        // Configure inference priority
        ret = gasket_write_32(gasket_dev, vr_config->vr_inference_priority, 
                             APEX_BAR_INDEX, APEX_INFERENCE_PRIORITY_OFFSET);
        if (ret)
            return ret;

        // Configure power mode
        ret = gasket_write_32(gasket_dev, vr_config->vr_power_mode, 
                             APEX_BAR_INDEX, APEX_POWER_MODE_OFFSET);
        if (ret)
            return ret;

        // Configure latency target
        ret = gasket_write_32(gasket_dev, vr_config->vr_latency_target_us, 
                             APEX_BAR_INDEX, APEX_LATENCY_TARGET_OFFSET);
        if (ret)
            return ret;

        // Configure zero-copy mode if enabled
        if (vr_config->zero_copy_enabled) {
            ret = gasket_write_32(gasket_dev, 1, 
                                 APEX_BAR_INDEX, APEX_ZERO_COPY_ENABLE_OFFSET);
            if (ret)
                return ret;
        }
    }

    // Store VR configuration in private data
    gasket_dev->vr_config = vr_config;

    dev_info(dev, "Coral TPU configured for Orange Pi CM5 VR: %s, %s, buffer=%dMB, priority=%d, power=%d, latency=%dus\n",
             vr_config->vr_mode_enabled ? "VR-mode" : "normal-mode",
             vr_config->zero_copy_enabled ? "zero-copy" : "standard-copy",
             vr_config->vr_dma_buffer_size / (1024 * 1024),
             vr_config->vr_inference_priority,
             vr_config->vr_power_mode,
             vr_config->vr_latency_target_us);

    return 0;
}

// Update probe function to detect Orange Pi CM5
static int coral_pci_probe_orangepi(struct pci_dev *pdev, const struct pci_device_id *ent)
{
    struct device *dev = &pdev->dev;
    struct gasket_dev *gasket_dev;
    int ret;

    // Call original probe function
    ret = apex_pci_probe(pdev, ent);
    if (ret)
        return ret;

    // Get the gasket_dev structure
    gasket_dev = pci_get_drvdata(pdev);
    if (!gasket_dev)
        return -ENODEV;

    // Check if this is an Orange Pi CM5 device
    if (of_device_is_compatible(dev->of_node, "orangepi,coral-tpu-vr")) {
        dev_info(dev, "Detected Orange Pi CM5 VR TPU\n");
        
        // Apply Orange Pi CM5 specific configuration
        ret = coral_configure_orangepi_cm5_vr(gasket_dev);
        if (ret) {
            dev_err(dev, "Failed to configure for Orange Pi CM5: %d\n", ret);
            return ret;
        }
        
        // Set Orange Pi CM5 specific flags
        gasket_dev->is_orangepi_cm5 = true;
    }

    return 0;
}

// Update the compatible strings to include Orange Pi variant
static const struct of_device_id coral_of_match_orangepi[] = {
    { .compatible = "google,coral-tpu" },
    { .compatible = "orangepi,coral-tpu-vr" },
    { /* sentinel */ }
};
MODULE_DEVICE_TABLE(of, coral_of_match_orangepi);

// Update the pci_driver structure
static struct pci_driver coral_pci_driver_orangepi = {
    .name = "gasket_orangepi",
    .id_table = apex_pci_ids,
    .probe = coral_pci_probe_orangepi,
    .remove = apex_pci_remove,
    .driver.pm = &apex_pm_ops,
};

module_pci_driver(coral_pci_driver_orangepi);

MODULE_DESCRIPTION("Coral TPU Driver for Orange Pi CM5 VR");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
```

## Zero-Copy Buffer Management

A key feature of the Orange Pi CM5 TPU driver adaptation is the zero-copy buffer management system, which enables direct DMA buffer sharing between the camera, CPU, and TPU. This is implemented through the following components:

```c
// drivers/misc/coral/gasket/apex_dma_orangepi.c

#include <linux/module.h>
#include <linux/pci.h>
#include <linux/dma-mapping.h>
#include <linux/iommu.h>
#include <linux/dma-buf.h>
#include <linux/scatterlist.h>
#include <linux/slab.h>
#include <linux/vmalloc.h>

// Include the original Coral TPU driver headers
#include "gasket_core.h"
#include "apex.h"
#include "apex_internal.h"

// Zero-copy DMA buffer structure
struct apex_zero_copy_buffer {
    struct dma_buf *dmabuf;
    struct dma_buf_attachment *attachment;
    struct sg_table *sgt;
    dma_addr_t dma_addr;
    void *vaddr;
    size_t size;
    bool is_imported;
};

// Import a DMA buffer from another device (e.g., camera)
int apex_import_dma_buf_orangepi(struct gasket_dev *gasket_dev, int dma_buf_fd, 
                               struct apex_zero_copy_buffer **buffer_out)
{
    struct device *dev = gasket_dev->dev;
    struct apex_zero_copy_buffer *buffer;
    int ret;

    buffer = kzalloc(sizeof(*buffer), GFP_KERNEL);
    if (!buffer)
        return -ENOMEM;

    // Import the DMA buffer
    buffer->dmabuf = dma_buf_get(dma_buf_fd);
    if (IS_ERR(buffer->dmabuf)) {
        ret = PTR_ERR(buffer->dmabuf);
        dev_err(dev, "Failed to import DMA buffer: %d\n", ret);
        goto err_free_buffer;
    }

    // Attach to the DMA buffer
    buffer->attachment = dma_buf_attach(buffer->dmabuf, dev);
    if (IS_ERR(buffer->attachment)) {
        ret = PTR_ERR(buffer->attachment);
        dev_err(dev, "Failed to attach to DMA buffer: %d\n", ret);
        goto err_put_dmabuf;
    }

    // Map the DMA buffer
    buffer->sgt = dma_buf_map_attachment(buffer->attachment, DMA_BIDIRECTIONAL);
    if (IS_ERR(buffer->sgt)) {
        ret = PTR_ERR(buffer->sgt);
        dev_err(dev, "Failed to map DMA buffer: %d\n", ret);
        goto err_detach_dmabuf;
    }

    // Get the DMA address
    buffer->dma_addr = sg_dma_address(buffer->sgt->sgl);
    buffer->size = buffer->dmabuf->size;
    buffer->is_imported = true;

    // Map the buffer for CPU access if needed
    buffer->vaddr = dma_buf_vmap(buffer->dmabuf);
    if (!buffer->vaddr) {
        dev_warn(dev, "Failed to map DMA buffer for CPU access\n");
    }

    *buffer_out = buffer;
    return 0;

err_detach_dmabuf:
    dma_buf_detach(buffer->dmabuf, buffer->attachment);
err_put_dmabuf:
    dma_buf_put(buffer->dmabuf);
err_free_buffer:
    kfree(buffer);
    return ret;
}

// Release a zero-copy buffer
void apex_release_dma_buf_orangepi(struct gasket_dev *gasket_dev, 
                                 struct apex_zero_copy_buffer *buffer)
{
    if (!buffer)
        return;

    if (buffer->vaddr)
        dma_buf_vunmap(buffer->dmabuf, buffer->vaddr);

    if (buffer->sgt)
        dma_buf_unmap_attachment(buffer->attachment, buffer->sgt, DMA_BIDIRECTIONAL);

    if (buffer->attachment)
        dma_buf_detach(buffer->dmabuf, buffer->attachment);

    if (buffer->dmabuf)
        dma_buf_put(buffer->dmabuf);

    kfree(buffer);
}

// Allocate a zero-copy buffer for TPU use
int apex_alloc_dma_buf_orangepi(struct gasket_dev *gasket_dev, size_t size,
                              struct apex_zero_copy_buffer **buffer_out)
{
    struct device *dev = gasket_dev->dev;
    struct apex_zero_copy_buffer *buffer;
    void *cpu_addr;
    dma_addr_t dma_addr;
    int ret;

    buffer = kzalloc(sizeof(*buffer), GFP_KERNEL);
    if (!buffer)
        return -ENOMEM;

    // Allocate coherent DMA memory
    cpu_addr = dma_alloc_coherent(dev, size, &dma_addr, GFP_KERNEL);
    if (!cpu_addr) {
        ret = -ENOMEM;
        dev_err(dev, "Failed to allocate DMA buffer: %d\n", ret);
        goto err_free_buffer;
    }

    buffer->vaddr = cpu_addr;
    buffer->dma_addr = dma_addr;
    buffer->size = size;
    buffer->is_imported = false;

    *buffer_out = buffer;
    return 0;

err_free_buffer:
    kfree(buffer);
    return ret;
}

// Free an allocated zero-copy buffer
void apex_free_dma_buf_orangepi(struct gasket_dev *gasket_dev,
                              struct apex_zero_copy_buffer *buffer)
{
    if (!buffer)
        return;

    if (!buffer->is_imported && buffer->vaddr)
        dma_free_coherent(gasket_dev->dev, buffer->size, buffer->vaddr, buffer->dma_addr);

    kfree(buffer);
}

// Export functions for use by other modules
EXPORT_SYMBOL_GPL(apex_import_dma_buf_orangepi);
EXPORT_SYMBOL_GPL(apex_release_dma_buf_orangepi);
EXPORT_SYMBOL_GPL(apex_alloc_dma_buf_orangepi);
EXPORT_SYMBOL_GPL(apex_free_dma_buf_orangepi);
```

## Integration with Existing Driver

To integrate this adaptation with the existing Coral TPU driver, we need to:

1. Add the Orange Pi CM5 specific configuration to the existing driver
2. Update the device tree bindings to include the Orange Pi CM5 compatible string
3. Add the Orange Pi CM5 specific flags and VR configuration to the driver structure
4. Implement the zero-copy buffer management system

## Device Tree Binding Updates

```
Required properties for Orange Pi CM5 VR TPU:
- compatible: Must include "orangepi,coral-tpu-vr" for Orange Pi CM5 VR TPU
- vr,mode-enabled: Boolean property indicating VR mode is enabled
- vr,zero-copy-enabled: Boolean property indicating zero-copy mode is enabled
- vr,dma-buffer-size: Integer property specifying DMA buffer size in bytes
- vr,inference-priority: Integer property specifying inference priority (0-3)
- vr,power-mode: Integer property specifying power mode (0=low, 1=performance)
- vr,latency-target-us: Integer property specifying target latency in microseconds

Example:
&pcie2x1l2 {
    status = "okay";
    reset-gpios = <&gpio4 RK_PB6 GPIO_ACTIVE_HIGH>;
    vpcie-supply = <&vcc3v3_pcie20>;
    
    tpu@0,0 {
        compatible = "orangepi,coral-tpu-vr";
        reg = <0x000000 0 0 0 0>;
        vr,mode-enabled;
        vr,zero-copy-enabled;
        vr,dma-buffer-size = <16777216>; /* 16MB */
        vr,inference-priority = <2>;
        vr,power-mode = <1>;
        vr,latency-target-us = <5000>;
    };
};
```

## Build System Integration

To integrate this into the build system, add the following to the Makefile:

```makefile
# drivers/misc/coral/gasket/Makefile
obj-$(CONFIG_GASKET_FRAMEWORK) += gasket.o
gasket-objs += gasket_core.o gasket_ioctl.o gasket_interrupt.o gasket_page_table.o
obj-$(CONFIG_APEX_DRIVER) += apex.o
apex-objs += apex_driver.o
obj-$(CONFIG_APEX_DRIVER_ORANGEPI) += orangepi_vr_optimizations.o apex_dma_orangepi.o
```

And add the following to the Kconfig:

```kconfig
config APEX_DRIVER_ORANGEPI
    tristate "Coral TPU driver optimizations for Orange Pi CM5 VR"
    depends on APEX_DRIVER && PCI
    help
      Choose this option if you have an Orange Pi CM5 board with
      Coral TPU and VR requirements. This driver provides
      VR-specific optimizations such as zero-copy buffer management,
      latency optimization, and power management for VR applications.

      To compile this driver as a module, choose M here: the
      module will be called gasket_orangepi.
```

## VR-Specific Optimizations

The Orange Pi CM5 TPU driver adaptation includes several VR-specific optimizations:

1. **Zero-Copy Buffer Management**: Direct DMA buffer sharing between camera, CPU, and TPU
2. **Latency Optimization**: Configurable latency targets for inference operations
3. **Power Management**: Performance mode for consistent inference speed
4. **Inference Prioritization**: High-priority scheduling for VR-critical inferences
5. **DMA Buffer Size Configuration**: Optimized buffer sizes for VR workloads

These optimizations are designed to provide the low-latency, high-performance TPU acceleration required for VR applications, while still maintaining compatibility with the standard Coral TPU driver.
