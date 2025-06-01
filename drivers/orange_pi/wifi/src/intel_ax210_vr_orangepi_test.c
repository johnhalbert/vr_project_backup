/*
 * Intel AX210 WiFi Driver Unit Tests for Orange Pi CM5 VR
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
#include <linux/delay.h>
#include <linux/kunit/test.h>

// Include the driver headers
#include "iwl-drv.h"
#include "iwl-config.h"
#include "iwl-prph.h"
#include "iwl-io.h"
#include "iwl-trans.h"

/* Mock functions for testing */
static int mock_iwl_write_prph(struct iwl_trans *trans, u32 reg, u32 val)
{
    // Store the write in a test buffer for verification
    trans->test_write_reg = reg;
    trans->test_write_val = val;
    return 0;
}

static int mock_iwl_read_prph(struct iwl_trans *trans, u32 reg, u32 *val)
{
    // Return predefined values for testing
    switch (reg) {
    case 0x0000: // QOS_PARAM_AC_VO
        *val = 0x00020806;
        break;
    case 0x0010: // POWER_SCHEME
        *val = 0x00000001;
        break;
    case 0x0020: // CHANNEL_MONITOR_CONFIG
        *val = 0x00000001;
        break;
    case 0x0030: // LATENCY_CONFIG
        *val = 0x00000064;
        break;
    default:
        *val = 0x00000000;
        break;
    }
    return 0;
}

/* Test fixture */
struct iwl_orangepi_test {
    struct kunit *test;
    struct iwl_trans *trans;
    struct pci_dev *pdev;
    struct device_node *node;
};

/* Test setup */
static int iwl_orangepi_test_init(struct kunit *test)
{
    struct iwl_orangepi_test *ctx = test->priv;
    
    // Allocate test context
    ctx->trans = kunit_kzalloc(test, sizeof(*ctx->trans), GFP_KERNEL);
    if (!ctx->trans)
        return -ENOMEM;
    
    // Set up mock functions
    ctx->trans->write_prph = mock_iwl_write_prph;
    ctx->trans->read_prph = mock_iwl_read_prph;
    
    // Create a mock PCI device
    ctx->pdev = kunit_kzalloc(test, sizeof(*ctx->pdev), GFP_KERNEL);
    if (!ctx->pdev)
        return -ENOMEM;
    
    // Create a mock device node
    ctx->node = kunit_kzalloc(test, sizeof(*ctx->node), GFP_KERNEL);
    if (!ctx->node)
        return -ENOMEM;
    
    // Set up the device node with Orange Pi CM5 compatible string
    ctx->node->name = "intel-ax210";
    ctx->node->full_name = "intel-ax210@0";
    
    // Set up the device
    ctx->pdev->dev.of_node = ctx->node;
    ctx->trans->dev = &ctx->pdev->dev;
    
    return 0;
}

/* Test teardown */
static void iwl_orangepi_test_exit(struct kunit *test)
{
    // Cleanup is handled by KUnit's kunit_kzalloc
}

/* Test cases */

/* Test Orange Pi CM5 detection */
static void iwl_orangepi_test_detection(struct kunit *test)
{
    struct iwl_orangepi_test *ctx = test->priv;
    bool is_orangepi_cm5;
    
    // Set compatible string to Orange Pi CM5
    of_property_read_string(ctx->node, "compatible", "orangepi,intel-ax210-vr");
    
    // Check if device is detected as Orange Pi CM5
    is_orangepi_cm5 = of_device_is_compatible(ctx->trans->dev->of_node, "orangepi,intel-ax210-vr");
    
    // Assert that it is detected
    KUNIT_EXPECT_TRUE(test, is_orangepi_cm5);
}

/* Test QoS configuration */
static void iwl_orangepi_test_qos_config(struct kunit *test)
{
    struct iwl_orangepi_test *ctx = test->priv;
    int ret;
    
    // Call the configure function
    ret = iwl_configure_orangepi_cm5_vr(ctx->trans);
    
    // Assert that configuration was successful
    KUNIT_EXPECT_EQ(test, ret, 0);
    
    // Assert that QoS configuration was set
    KUNIT_EXPECT_EQ(test, ctx->trans->test_write_reg, 0x0000); // QOS_PARAM_AC_VO
    KUNIT_EXPECT_EQ(test, ctx->trans->test_write_val, 0x00020806); // tx_retry_limit=2, aggregation_limit=8, traffic_priority=6
}

/* Test power management configuration */
static void iwl_orangepi_test_power_config(struct kunit *test)
{
    struct iwl_orangepi_test *ctx = test->priv;
    int ret;
    
    // Call the configure function
    ret = iwl_configure_orangepi_cm5_vr(ctx->trans);
    
    // Assert that configuration was successful
    KUNIT_EXPECT_EQ(test, ret, 0);
    
    // Assert that power management configuration was set
    KUNIT_EXPECT_EQ(test, ctx->trans->test_write_reg, 0x0010); // POWER_SCHEME
    KUNIT_EXPECT_EQ(test, ctx->trans->test_write_val, 0x00000001); // POWER_SCHEME_CAM
}

/* Test channel monitoring configuration */
static void iwl_orangepi_test_channel_monitor(struct kunit *test)
{
    struct iwl_orangepi_test *ctx = test->priv;
    int ret;
    
    // Call the configure function
    ret = iwl_configure_orangepi_cm5_vr(ctx->trans);
    
    // Assert that configuration was successful
    KUNIT_EXPECT_EQ(test, ret, 0);
    
    // Assert that channel monitoring configuration was set
    KUNIT_EXPECT_EQ(test, ctx->trans->test_write_reg, 0x0020); // CHANNEL_MONITOR_CONFIG
    KUNIT_EXPECT_EQ(test, ctx->trans->test_write_val, 0x00000001); // CHANNEL_MONITOR_ENABLE
}

/* Test latency configuration */
static void iwl_orangepi_test_latency_config(struct kunit *test)
{
    struct iwl_orangepi_test *ctx = test->priv;
    int ret;
    
    // Call the configure function
    ret = iwl_configure_orangepi_cm5_vr(ctx->trans);
    
    // Assert that configuration was successful
    KUNIT_EXPECT_EQ(test, ret, 0);
    
    // Assert that latency configuration was set
    KUNIT_EXPECT_EQ(test, ctx->trans->test_write_reg, 0x0030); // LATENCY_CONFIG
    KUNIT_EXPECT_EQ(test, ctx->trans->test_write_val, 0x00000064); // 10000us / 100 = 100 (0x64)
}

/* Test suite definition */
static struct kunit_case iwl_orangepi_test_cases[] = {
    KUNIT_CASE(iwl_orangepi_test_detection),
    KUNIT_CASE(iwl_orangepi_test_qos_config),
    KUNIT_CASE(iwl_orangepi_test_power_config),
    KUNIT_CASE(iwl_orangepi_test_channel_monitor),
    KUNIT_CASE(iwl_orangepi_test_latency_config),
    {}
};

static struct kunit_suite iwl_orangepi_test_suite = {
    .name = "iwl_orangepi",
    .init = iwl_orangepi_test_init,
    .exit = iwl_orangepi_test_exit,
    .test_cases = iwl_orangepi_test_cases,
};

kunit_test_suite(iwl_orangepi_test_suite);

MODULE_DESCRIPTION("Intel AX210 WiFi Driver Unit Tests for Orange Pi CM5 VR");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
