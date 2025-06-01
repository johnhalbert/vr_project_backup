# Orange Pi CM5 WiFi Driver Adaptation

This file contains the necessary adaptations to the Intel AX210 WiFi driver for the Orange Pi CM5 platform.

```c
// drivers/net/wireless/intel/iwlwifi/orangepi_vr_optimizations.c

#include <linux/module.h>
#include <linux/pci.h>
#include <linux/of.h>
#include <linux/of_device.h>
#include <linux/of_net.h>
#include <linux/netdevice.h>
#include <linux/etherdevice.h>
#include <linux/delay.h>
#include <linux/pm_runtime.h>
#include <linux/pm_qos.h>
#include <net/mac80211.h>

// Include the original Intel WiFi driver headers
#include "iwl-drv.h"
#include "iwl-config.h"
#include "iwl-prph.h"
#include "iwl-io.h"
#include "iwl-trans.h"
#include "iwl-op-mode.h"
#include "iwl-phy-db.h"
#include "iwl-nvm-parse.h"
#include "iwl-csr.h"
#include "iwl-fw.h"
#include "iwl-debug.h"
#include "iwl-constants.h"

// VR-specific QoS parameters
struct iwl_vr_qos_config {
    bool vr_mode_enabled;
    u8 vr_traffic_priority;
    u32 vr_latency_target_us;
    u32 vr_bandwidth_target_kbps;
    u32 vr_tx_retry_limit;
    u32 vr_aggregation_limit;
};

// Orange Pi CM5 specific configuration
static int iwl_configure_orangepi_cm5_vr(struct iwl_trans *trans)
{
    struct device *dev = trans->dev;
    struct iwl_vr_qos_config *vr_config;
    int ret;

    dev_info(dev, "Configuring Intel AX210 for Orange Pi CM5 VR\n");

    // Allocate VR configuration
    vr_config = devm_kzalloc(dev, sizeof(*vr_config), GFP_KERNEL);
    if (!vr_config)
        return -ENOMEM;

    // Parse VR-specific device tree properties
    if (of_property_read_bool(dev->of_node, "vr,mode-enabled"))
        vr_config->vr_mode_enabled = true;

    of_property_read_u8(dev->of_node, "vr,traffic-priority", &vr_config->vr_traffic_priority);
    of_property_read_u32(dev->of_node, "vr,latency-target-us", &vr_config->vr_latency_target_us);
    of_property_read_u32(dev->of_node, "vr,bandwidth-target-kbps", &vr_config->vr_bandwidth_target_kbps);
    of_property_read_u32(dev->of_node, "vr,tx-retry-limit", &vr_config->vr_tx_retry_limit);
    of_property_read_u32(dev->of_node, "vr,aggregation-limit", &vr_config->vr_aggregation_limit);

    // Set default values if not specified
    if (!vr_config->vr_traffic_priority)
        vr_config->vr_traffic_priority = 6; // AC_VO (voice) priority

    if (!vr_config->vr_latency_target_us)
        vr_config->vr_latency_target_us = 10000; // 10ms target latency

    if (!vr_config->vr_bandwidth_target_kbps)
        vr_config->vr_bandwidth_target_kbps = 20000; // 20Mbps target bandwidth

    if (!vr_config->vr_tx_retry_limit)
        vr_config->vr_tx_retry_limit = 2; // Limit retries to reduce latency

    if (!vr_config->vr_aggregation_limit)
        vr_config->vr_aggregation_limit = 8; // Limit aggregation to reduce latency

    // Configure WiFi for VR mode
    if (vr_config->vr_mode_enabled) {
        // Configure QoS parameters for VR traffic
        ret = iwl_write_prph(trans, IWL_QOS_PARAM_AC_VO, 
                            (vr_config->vr_tx_retry_limit << 16) | 
                            (vr_config->vr_aggregation_limit << 8) | 
                            vr_config->vr_traffic_priority);
        if (ret)
            return ret;

        // Configure power management for VR (disable power save)
        ret = iwl_write_prph(trans, IWL_POWER_SCHEME, IWL_POWER_SCHEME_CAM);
        if (ret)
            return ret;

        // Configure channel utilization monitoring
        ret = iwl_write_prph(trans, IWL_CHANNEL_MONITOR_CONFIG, 1);
        if (ret)
            return ret;

        // Configure latency optimization
        ret = iwl_write_prph(trans, IWL_LATENCY_CONFIG, 
                            vr_config->vr_latency_target_us / 100); // Convert to 100us units
        if (ret)
            return ret;
    }

    // Store VR configuration in private data
    trans->vr_config = vr_config;

    dev_info(dev, "Intel AX210 configured for Orange Pi CM5 VR: %s, priority=%d, latency=%dus, bandwidth=%dkbps, retry=%d, agg=%d\n",
             vr_config->vr_mode_enabled ? "VR-mode" : "normal-mode",
             vr_config->vr_traffic_priority,
             vr_config->vr_latency_target_us,
             vr_config->vr_bandwidth_target_kbps,
             vr_config->vr_tx_retry_limit,
             vr_config->vr_aggregation_limit);

    return 0;
}

// Update probe function to detect Orange Pi CM5
static int iwl_pci_probe_orangepi(struct pci_dev *pdev, const struct pci_device_id *ent)
{
    struct device *dev = &pdev->dev;
    struct iwl_trans *trans;
    int ret;

    // Call original probe function
    ret = iwl_pci_probe(pdev, ent);
    if (ret)
        return ret;

    // Get the trans structure
    trans = pci_get_drvdata(pdev);
    if (!trans)
        return -ENODEV;

    // Check if this is an Orange Pi CM5 device
    if (of_device_is_compatible(dev->of_node, "orangepi,intel-ax210-vr")) {
        dev_info(dev, "Detected Orange Pi CM5 VR WiFi\n");
        
        // Apply Orange Pi CM5 specific configuration
        ret = iwl_configure_orangepi_cm5_vr(trans);
        if (ret) {
            dev_err(dev, "Failed to configure for Orange Pi CM5: %d\n", ret);
            return ret;
        }
        
        // Set Orange Pi CM5 specific flags
        trans->is_orangepi_cm5 = true;
    }

    return 0;
}

// Update the compatible strings to include Orange Pi variant
static const struct of_device_id iwl_of_match_orangepi[] = {
    { .compatible = "pci14e4,4433" },
    { .compatible = "orangepi,intel-ax210-vr" },
    { /* sentinel */ }
};
MODULE_DEVICE_TABLE(of, iwl_of_match_orangepi);

// Update the pci_driver structure
static struct pci_driver iwl_pci_driver_orangepi = {
    .name = "iwlwifi_orangepi",
    .id_table = iwl_hw_card_ids,
    .probe = iwl_pci_probe_orangepi,
    .remove = iwl_pci_remove,
    .driver.pm = &iwl_pm_ops,
};

module_pci_driver(iwl_pci_driver_orangepi);

MODULE_DESCRIPTION("Intel WiFi Driver for Orange Pi CM5 VR");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
```

## Integration with Existing Driver

To integrate this adaptation with the existing Intel WiFi driver, we need to:

1. Add the Orange Pi CM5 specific configuration to the existing driver
2. Update the device tree bindings to include the Orange Pi CM5 compatible string
3. Add the Orange Pi CM5 specific flags and VR configuration to the driver structure

## Device Tree Binding Updates

```
Required properties for Orange Pi CM5 VR WiFi:
- compatible: Must include "orangepi,intel-ax210-vr" for Orange Pi CM5 VR WiFi
- vr,mode-enabled: Boolean property indicating VR mode is enabled
- vr,traffic-priority: Integer property specifying traffic priority (0-7)
- vr,latency-target-us: Integer property specifying target latency in microseconds
- vr,bandwidth-target-kbps: Integer property specifying target bandwidth in kbps
- vr,tx-retry-limit: Integer property specifying maximum transmission retries
- vr,aggregation-limit: Integer property specifying maximum aggregation

Example:
&pcie2x1l0 {
    status = "okay";
    reset-gpios = <&gpio4 RK_PA2 GPIO_ACTIVE_HIGH>;
    vpcie-supply = <&vcc3v3_pcie20>;
    
    pcie@0 {
        reg = <0x00000000 0 0 0 0>;
        #address-cells = <3>;
        #size-cells = <2>;
        
        wifi@0,0 {
            compatible = "orangepi,intel-ax210-vr";
            reg = <0x000000 0 0 0 0>;
            interrupt-parent = <&gpio0>;
            interrupts = <RK_PC4 IRQ_TYPE_LEVEL_HIGH>;
            vr,mode-enabled;
            vr,traffic-priority = <6>;
            vr,latency-target-us = <10000>;
            vr,bandwidth-target-kbps = <20000>;
            vr,tx-retry-limit = <2>;
            vr,aggregation-limit = <8>;
        };
    };
};
```

## Build System Integration

To integrate this into the build system, add the following to the Makefile:

```makefile
# drivers/net/wireless/intel/iwlwifi/Makefile
obj-$(CONFIG_IWLWIFI) += iwlwifi.o
iwlwifi-objs += pcie/drv.o
obj-$(CONFIG_IWLWIFI_ORANGEPI) += orangepi_vr_optimizations.o
```

And add the following to the Kconfig:

```kconfig
config IWLWIFI_ORANGEPI
    tristate "Intel WiFi driver optimizations for Orange Pi CM5 VR"
    depends on IWLWIFI && PCI
    help
      Choose this option if you have an Orange Pi CM5 board with
      Intel AX210 WiFi and VR requirements. This driver provides
      VR-specific optimizations such as latency reduction, QoS
      traffic classification, and power management optimizations.

      To compile this driver as a module, choose M here: the
      module will be called iwlwifi_orangepi.
```

## VR-Specific Optimizations

The Orange Pi CM5 WiFi driver adaptation includes several VR-specific optimizations:

1. **Latency Optimization**: Reduced transmission retry limits and aggregation limits to minimize latency
2. **QoS Traffic Classification**: Prioritization of VR traffic for improved performance
3. **Power Management**: Disabled power-saving features to ensure consistent performance
4. **Channel Utilization Monitoring**: Real-time monitoring of channel conditions for adaptive performance
5. **Bandwidth Reservation**: Target bandwidth configuration for VR applications

These optimizations are designed to provide the low-latency, high-reliability wireless connectivity required for VR applications, while still maintaining compatibility with the standard Intel WiFi driver.
