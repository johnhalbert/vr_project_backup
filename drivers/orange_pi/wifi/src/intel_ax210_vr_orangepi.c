/*
 * Intel AX210 WiFi Driver for Orange Pi CM5 VR
 *
 * Copyright (c) 2025 VR Headset Project
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License version 2 as
 * published by the Free Software Foundation.
 */

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

/* Intel AX210 Register Map for Orange Pi CM5 VR */
#define IWL_QOS_PARAM_AC_VO           0x0000
#define IWL_POWER_SCHEME              0x0010
#define IWL_CHANNEL_MONITOR_CONFIG    0x0020
#define IWL_LATENCY_CONFIG            0x0030

/* Intel AX210 Register Values for Orange Pi CM5 VR */
#define IWL_POWER_SCHEME_CAM          0x00000001
#define IWL_CHANNEL_MONITOR_ENABLE    0x00000001

/* VR-specific QoS parameters */
struct iwl_vr_qos_config {
    bool vr_mode_enabled;
    u8 vr_traffic_priority;
    u32 vr_latency_target_us;
    u32 vr_bandwidth_target_kbps;
    u32 vr_tx_retry_limit;
    u32 vr_aggregation_limit;
};

/* Intel AX210 device structure with Orange Pi CM5 extensions */
struct iwl_orangepi_device {
    struct iwl_trans *trans;
    struct iwl_vr_qos_config vr_config;
    bool is_orangepi_cm5;
};

/* Forward declarations */
static int iwl_write_prph(struct iwl_trans *trans, u32 reg, u32 val);
static int iwl_read_prph(struct iwl_trans *trans, u32 reg, u32 *val);

/* Orange Pi CM5 specific configuration */
static int iwl_configure_orangepi_cm5_vr(struct iwl_trans *trans)
{
    struct device *dev = trans->dev;
    struct iwl_orangepi_device *orangepi_dev;
    struct iwl_vr_qos_config *vr_config;
    int ret;

    dev_info(dev, "Configuring Intel AX210 for Orange Pi CM5 VR\n");

    /* Allocate Orange Pi device structure */
    orangepi_dev = devm_kzalloc(dev, sizeof(*orangepi_dev), GFP_KERNEL);
    if (!orangepi_dev)
        return -ENOMEM;

    orangepi_dev->trans = trans;
    vr_config = &orangepi_dev->vr_config;

    /* Parse VR-specific device tree properties */
    if (of_property_read_bool(dev->of_node, "vr,mode-enabled"))
        vr_config->vr_mode_enabled = true;

    of_property_read_u8(dev->of_node, "vr,traffic-priority", &vr_config->vr_traffic_priority);
    of_property_read_u32(dev->of_node, "vr,latency-target-us", &vr_config->vr_latency_target_us);
    of_property_read_u32(dev->of_node, "vr,bandwidth-target-kbps", &vr_config->vr_bandwidth_target_kbps);
    of_property_read_u32(dev->of_node, "vr,tx-retry-limit", &vr_config->vr_tx_retry_limit);
    of_property_read_u32(dev->of_node, "vr,aggregation-limit", &vr_config->vr_aggregation_limit);

    /* Set default values if not specified */
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

    /* Configure WiFi for VR mode */
    if (vr_config->vr_mode_enabled) {
        /* Configure QoS parameters for VR traffic */
        ret = iwl_write_prph(trans, IWL_QOS_PARAM_AC_VO, 
                            (vr_config->vr_tx_retry_limit << 16) | 
                            (vr_config->vr_aggregation_limit << 8) | 
                            vr_config->vr_traffic_priority);
        if (ret)
            return ret;

        /* Configure power management for VR (disable power save) */
        ret = iwl_write_prph(trans, IWL_POWER_SCHEME, IWL_POWER_SCHEME_CAM);
        if (ret)
            return ret;

        /* Configure channel utilization monitoring */
        ret = iwl_write_prph(trans, IWL_CHANNEL_MONITOR_CONFIG, IWL_CHANNEL_MONITOR_ENABLE);
        if (ret)
            return ret;

        /* Configure latency optimization */
        ret = iwl_write_prph(trans, IWL_LATENCY_CONFIG, 
                            vr_config->vr_latency_target_us / 100); // Convert to 100us units
        if (ret)
            return ret;
    }

    /* Store Orange Pi device structure in private data */
    trans->vr_config = vr_config;
    orangepi_dev->is_orangepi_cm5 = true;

    dev_info(dev, "Intel AX210 configured for Orange Pi CM5 VR: %s, priority=%d, latency=%dus, bandwidth=%dkbps, retry=%d, agg=%d\n",
             vr_config->vr_mode_enabled ? "VR-mode" : "normal-mode",
             vr_config->vr_traffic_priority,
             vr_config->vr_latency_target_us,
             vr_config->vr_bandwidth_target_kbps,
             vr_config->vr_tx_retry_limit,
             vr_config->vr_aggregation_limit);

    return 0;
}

/* Update probe function to detect Orange Pi CM5 */
static int iwl_pci_probe_orangepi(struct pci_dev *pdev, const struct pci_device_id *ent)
{
    struct device *dev = &pdev->dev;
    struct iwl_trans *trans;
    int ret;

    dev_info(dev, "Probing Intel AX210 for Orange Pi CM5\n");

    /* Call original probe function */
    ret = iwl_pci_probe(pdev, ent);
    if (ret)
        return ret;

    /* Get the trans structure */
    trans = pci_get_drvdata(pdev);
    if (!trans)
        return -ENODEV;

    /* Check if this is an Orange Pi CM5 device */
    if (of_device_is_compatible(dev->of_node, "orangepi,intel-ax210-vr")) {
        dev_info(dev, "Detected Orange Pi CM5 VR WiFi\n");
        
        /* Apply Orange Pi CM5 specific configuration */
        ret = iwl_configure_orangepi_cm5_vr(trans);
        if (ret) {
            dev_err(dev, "Failed to configure for Orange Pi CM5: %d\n", ret);
            return ret;
        }
        
        /* Set Orange Pi CM5 specific flags */
        trans->is_orangepi_cm5 = true;
    }

    return 0;
}

/* Update the compatible strings to include Orange Pi variant */
static const struct of_device_id iwl_of_match_orangepi[] = {
    { .compatible = "pci14e4,4433" },
    { .compatible = "orangepi,intel-ax210-vr" },
    { /* sentinel */ }
};
MODULE_DEVICE_TABLE(of, iwl_of_match_orangepi);

/* Update the pci_driver structure */
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
