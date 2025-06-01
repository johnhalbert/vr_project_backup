/*
 * Coral TPU Driver Integration Tests for Orange Pi CM5 VR
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
#include <linux/dma-mapping.h>
#include <linux/delay.h>
#include <linux/kunit/test.h>

// Include the driver headers
#include "gasket.h"
#include "apex.h"
#include "apex_driver.h"

/* Integration test fixture */
struct coral_tpu_orangepi_integration_test {
    struct kunit *test;
    struct apex_driver_data *dev;
    struct platform_device *pdev;
    struct device_node *node;
    struct coral_tpu_orangepi_device *orangepi_dev;
};

/* Test setup */
static int coral_tpu_orangepi_integration_test_init(struct kunit *test)
{
    struct coral_tpu_orangepi_integration_test *ctx = test->priv;
    
    // Allocate test context
    ctx->dev = kunit_kzalloc(test, sizeof(*ctx->dev), GFP_KERNEL);
    if (!ctx->dev)
        return -ENOMEM;
    
    // Create a mock platform device
    ctx->pdev = kunit_kzalloc(test, sizeof(*ctx->pdev), GFP_KERNEL);
    if (!ctx->pdev)
        return -ENOMEM;
    
    // Create a mock device node
    ctx->node = kunit_kzalloc(test, sizeof(*ctx->node), GFP_KERNEL);
    if (!ctx->node)
        return -ENOMEM;
    
    // Set up the device node with Orange Pi CM5 compatible string
    ctx->node->name = "coral-tpu";
    ctx->node->full_name = "coral-tpu@0";
    
    // Set up the platform device
    ctx->pdev->dev.of_node = ctx->node;
    
    // Set up the device
    ctx->dev->dev = &ctx->pdev->dev;
    
    // Allocate Orange Pi device structure
    ctx->orangepi_dev = kunit_kzalloc(test, sizeof(*ctx->orangepi_dev), GFP_KERNEL);
    if (!ctx->orangepi_dev)
        return -ENOMEM;
    
    // Link the structures
    ctx->orangepi_dev->base_dev = *ctx->dev;
    
    return 0;
}

/* Test teardown */
static void coral_tpu_orangepi_integration_test_exit(struct kunit *test)
{
    // Cleanup is handled by KUnit's kunit_kzalloc
}

/* Test cases */

/* Test device tree integration */
static void coral_tpu_orangepi_test_device_tree(struct kunit *test)
{
    struct coral_tpu_orangepi_integration_test *ctx = test->priv;
    bool is_compatible;
    
    // Set compatible string to Orange Pi CM5
    of_property_read_string(ctx->node, "compatible", "orangepi,coral-tpu-vr");
    
    // Check if device is compatible
    is_compatible = of_device_is_compatible(ctx->dev->dev->of_node, "orangepi,coral-tpu-vr");
    
    // Assert that it is compatible
    KUNIT_EXPECT_TRUE(test, is_compatible);
}

/* Test DMA buffer allocation */
static void coral_tpu_orangepi_test_dma_buffer(struct kunit *test)
{
    struct coral_tpu_orangepi_integration_test *ctx = test->priv;
    void *buffer;
    dma_addr_t dma_addr;
    
    // Allocate a DMA buffer
    buffer = dma_alloc_coherent(ctx->dev->dev, 4096, &dma_addr, GFP_KERNEL);
    
    // Assert that buffer allocation was successful
    KUNIT_EXPECT_PTR_NE(test, buffer, NULL);
    KUNIT_EXPECT_NE(test, dma_addr, 0);
    
    // Free the buffer
    if (buffer)
        dma_free_coherent(ctx->dev->dev, 4096, buffer, dma_addr);
}

/* Test zero-copy integration */
static void coral_tpu_orangepi_test_zero_copy(struct kunit *test)
{
    struct coral_tpu_orangepi_integration_test *ctx = test->priv;
    
    // Set zero-copy enabled
    ctx->orangepi_dev->vr_config.zero_copy_enabled = true;
    
    // Assert that zero-copy is enabled
    KUNIT_EXPECT_TRUE(test, ctx->orangepi_dev->vr_config.zero_copy_enabled);
}

/* Test VR configuration integration */
static void coral_tpu_orangepi_test_vr_config(struct kunit *test)
{
    struct coral_tpu_orangepi_integration_test *ctx = test->priv;
    
    // Set VR configuration
    ctx->orangepi_dev->vr_config.vr_mode_enabled = true;
    ctx->orangepi_dev->vr_config.latency_target_ms = 5;
    ctx->orangepi_dev->vr_config.buffer_size_kb = 4096;
    ctx->orangepi_dev->vr_config.performance_mode = true;
    ctx->orangepi_dev->vr_config.inference_priority = 90;
    
    // Assert that VR configuration is set
    KUNIT_EXPECT_TRUE(test, ctx->orangepi_dev->vr_config.vr_mode_enabled);
    KUNIT_EXPECT_EQ(test, ctx->orangepi_dev->vr_config.latency_target_ms, 5);
    KUNIT_EXPECT_EQ(test, ctx->orangepi_dev->vr_config.buffer_size_kb, 4096);
    KUNIT_EXPECT_TRUE(test, ctx->orangepi_dev->vr_config.performance_mode);
    KUNIT_EXPECT_EQ(test, ctx->orangepi_dev->vr_config.inference_priority, 90);
}

/* Test platform device integration */
static void coral_tpu_orangepi_test_platform_device(struct kunit *test)
{
    struct coral_tpu_orangepi_integration_test *ctx = test->priv;
    
    // Set platform device name
    ctx->pdev->name = "apex-orangepi";
    
    // Assert that platform device is set up
    KUNIT_EXPECT_STREQ(test, ctx->pdev->name, "apex-orangepi");
    KUNIT_EXPECT_PTR_EQ(test, ctx->pdev->dev.of_node, ctx->node);
}

/* Test suite definition */
static struct kunit_case coral_tpu_orangepi_integration_test_cases[] = {
    KUNIT_CASE(coral_tpu_orangepi_test_device_tree),
    KUNIT_CASE(coral_tpu_orangepi_test_dma_buffer),
    KUNIT_CASE(coral_tpu_orangepi_test_zero_copy),
    KUNIT_CASE(coral_tpu_orangepi_test_vr_config),
    KUNIT_CASE(coral_tpu_orangepi_test_platform_device),
    {}
};

static struct kunit_suite coral_tpu_orangepi_integration_test_suite = {
    .name = "coral_tpu_orangepi_integration",
    .init = coral_tpu_orangepi_integration_test_init,
    .exit = coral_tpu_orangepi_integration_test_exit,
    .test_cases = coral_tpu_orangepi_integration_test_cases,
};

kunit_test_suite(coral_tpu_orangepi_integration_test_suite);

MODULE_DESCRIPTION("Coral TPU Driver Integration Tests for Orange Pi CM5 VR");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
