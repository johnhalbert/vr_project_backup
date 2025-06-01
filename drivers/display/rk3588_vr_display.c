// SPDX-License-Identifier: GPL-2.0
/*
 * RK3588 VR Display Driver - Core Implementation
 *
 * Copyright (C) 2025 VR Headset Project
 *
 * This driver provides DRM/KMS support for RK3588 display controller
 * with specific optimizations for VR headset dual-display operation.
 */

#include "rk3588_vr_display.h"

/* Module parameters */
static bool enable_low_persistence = true;
module_param(enable_low_persistence, bool, 0644);
MODULE_PARM_DESC(enable_low_persistence, "Enable low persistence mode (default: true)");

static int low_persistence_duty = RK3588_VR_LOW_PERSISTENCE_DUTY;
module_param(low_persistence_duty, int, 0644);
MODULE_PARM_DESC(low_persistence_duty, "Low persistence duty cycle percentage (default: 70)");

static bool enable_fast_path = true;
module_param(enable_fast_path, bool, 0644);
MODULE_PARM_DESC(enable_fast_path, "Enable fast path for reduced latency (default: true)");

static bool enable_distortion = true;
module_param(enable_distortion, bool, 0644);
MODULE_PARM_DESC(enable_distortion, "Enable lens distortion correction (default: true)");

static bool enable_chromatic = true;
module_param(enable_chromatic, bool, 0644);
MODULE_PARM_DESC(enable_chromatic, "Enable chromatic aberration correction (default: true)");

static bool enable_motion_comp = true;
module_param(enable_motion_comp, bool, 0644);
MODULE_PARM_DESC(enable_motion_comp, "Enable motion compensation (default: true)");

static int target_vrefresh = RK3588_VR_TARGET_VREFRESH;
module_param(target_vrefresh, int, 0644);
MODULE_PARM_DESC(target_vrefresh, "Target vertical refresh rate (default: 90)");

static int max_latency_us = RK3588_VR_MAX_LATENCY_US;
module_param(max_latency_us, int, 0644);
MODULE_PARM_DESC(max_latency_us, "Maximum motion-to-photon latency in microseconds (default: 5000)");

/* Forward declarations */
static int rk3588_vr_display_thread(void *data);
static void rk3588_vr_display_handle_vsync(struct rk3588_vr_display *vrd, int display_idx);
static void rk3588_vr_display_handle_commit(struct rk3588_vr_display *vrd, int display_idx);

/**
 * rk3588_vr_display_init - Initialize the RK3588 VR display driver
 * @vrd: Pointer to the RK3588 VR display device structure
 *
 * This function initializes the RK3588 VR display driver, including
 * hardware initialization, clock setup, and configuration of VR-specific
 * features.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_init(struct rk3588_vr_display *vrd)
{
    int ret, i;
    u32 val;

    if (!vrd || !vrd->dev || !vrd->regs)
        return -EINVAL;

    dev_info(vrd->dev, "Initializing RK3588 VR display driver\n");

    /* Initialize locks and completions */
    spin_lock_init(&vrd->lock);
    
    for (i = 0; i < RK3588_VR_MAX_DISPLAYS; i++) {
        init_completion(&vrd->vsync_completion[i]);
        init_completion(&vrd->commit_completion[i]);
    }
    
    init_completion(&vrd->vr_thread_completion);
    atomic_set(&vrd->vr_thread_active, 0);

    /* Enable clocks */
    ret = clk_prepare_enable(vrd->hclk);
    if (ret) {
        dev_err(vrd->dev, "Failed to enable hclk: %d\n", ret);
        return ret;
    }

    ret = clk_prepare_enable(vrd->aclk);
    if (ret) {
        dev_err(vrd->dev, "Failed to enable aclk: %d\n", ret);
        goto err_disable_hclk;
    }

    for (i = 0; i < RK3588_VR_MAX_DISPLAYS; i++) {
        if (vrd->dclk[i]) {
            ret = clk_prepare_enable(vrd->dclk[i]);
            if (ret) {
                dev_err(vrd->dev, "Failed to enable dclk[%d]: %d\n", i, ret);
                goto err_disable_dclks;
            }
        }
    }

    /* Reset the controller */
    ret = reset_control_assert(vrd->rstc);
    if (ret) {
        dev_err(vrd->dev, "Failed to assert reset: %d\n", ret);
        goto err_disable_dclks;
    }

    usleep_range(10, 20);

    ret = reset_control_deassert(vrd->rstc);
    if (ret) {
        dev_err(vrd->dev, "Failed to deassert reset: %d\n", ret);
        goto err_disable_dclks;
    }

    /* Initialize hardware */
    val = RK3588_VOP_SYS_CTRL_EN |
          RK3588_VOP_SYS_CTRL_CORE_CLK_EN |
          RK3588_VOP_SYS_CTRL_DCLK_EN |
          RK3588_VOP_SYS_CTRL_MMU_EN |
          RK3588_VOP_SYS_CTRL_AXI_OUTSTANDING_MAX(8) |
          RK3588_VOP_SYS_CTRL_GLOBAL_REGDONE;
    
    writel(val, vrd->regs + RK3588_VOP_SYS_CTRL);

    /* Configure VR-specific features */
    vrd->config.mode = RK3588_VR_MODE_NORMAL;
    vrd->config.sync_mode = RK3588_VR_SYNC_MASTER;
    vrd->config.distortion_mode = enable_distortion ? RK3588_VR_DISTORTION_BARREL : RK3588_VR_DISTORTION_NONE;
    vrd->config.chromatic_mode = enable_chromatic ? RK3588_VR_CHROMATIC_RGB : RK3588_VR_CHROMATIC_NONE;
    vrd->config.motion_comp_mode = enable_motion_comp ? RK3588_VR_MOTION_COMP_PREDICT : RK3588_VR_MOTION_COMP_NONE;
    vrd->config.low_persistence = enable_low_persistence;
    vrd->config.low_persistence_duty = low_persistence_duty;
    vrd->config.fast_path = enable_fast_path;
    vrd->config.bypass_blend = false;
    vrd->config.bypass_scale = false;
    vrd->config.bypass_gamma = false;
    vrd->config.bypass_dither = false;
    vrd->config.target_vrefresh = target_vrefresh;
    vrd->config.max_latency_us = max_latency_us;

    /* Configure VR sync mode */
    val = RK3588_VOP_VR_SYNC_CTRL_EN;
    if (vrd->config.sync_mode == RK3588_VR_SYNC_MASTER)
        val |= RK3588_VOP_VR_SYNC_CTRL_MASTER;
    else if (vrd->config.sync_mode == RK3588_VR_SYNC_SLAVE)
        val |= RK3588_VOP_VR_SYNC_CTRL_SLAVE;
    
    val |= RK3588_VOP_VR_SYNC_CTRL_VSYNC | RK3588_VOP_VR_SYNC_CTRL_HSYNC;
    val |= RK3588_VOP_VR_SYNC_CTRL_PHASE(0);
    
    writel(val, vrd->regs + RK3588_VOP_VR_SYNC_CTRL);

    /* Configure VR low persistence mode */
    val = 0;
    if (vrd->config.low_persistence) {
        val = RK3588_VOP_VR_LOW_PERSIST_EN |
              RK3588_VOP_VR_LOW_PERSIST_DUTY(vrd->config.low_persistence_duty);
    }
    
    writel(val, vrd->regs + RK3588_VOP_VR_LOW_PERSIST);

    /* Configure VR latency control */
    val = RK3588_VOP_VR_LATENCY_CTRL_EN;
    if (vrd->config.fast_path)
        val |= RK3588_VOP_VR_LATENCY_CTRL_FAST_PATH;
    if (vrd->config.bypass_blend)
        val |= RK3588_VOP_VR_LATENCY_CTRL_BYPASS_BLEND;
    if (vrd->config.bypass_scale)
        val |= RK3588_VOP_VR_LATENCY_CTRL_BYPASS_SCALE;
    if (vrd->config.bypass_gamma)
        val |= RK3588_VOP_VR_LATENCY_CTRL_BYPASS_GAMMA;
    if (vrd->config.bypass_dither)
        val |= RK3588_VOP_VR_LATENCY_CTRL_BYPASS_DITHER;
    
    writel(val, vrd->regs + RK3588_VOP_VR_LATENCY_CTRL);

    /* Configure VR distortion correction */
    val = 0;
    if (vrd->config.distortion_mode != RK3588_VR_DISTORTION_NONE) {
        val = RK3588_VOP_VR_DISTORTION_CTRL_EN;
        
        switch (vrd->config.distortion_mode) {
        case RK3588_VR_DISTORTION_BARREL:
            val |= RK3588_VOP_VR_DISTORTION_CTRL_BARREL;
            break;
        case RK3588_VR_DISTORTION_PINCUSHION:
            val |= RK3588_VOP_VR_DISTORTION_CTRL_PINCUSHION;
            break;
        case RK3588_VR_DISTORTION_MESH:
            val |= RK3588_VOP_VR_DISTORTION_CTRL_MESH;
            break;
        case RK3588_VR_DISTORTION_CUSTOM:
            val |= RK3588_VOP_VR_DISTORTION_CTRL_CUSTOM;
            break;
        default:
            break;
        }
    }
    
    writel(val, vrd->regs + RK3588_VOP_VR_DISTORTION_CTRL);

    /* Configure VR chromatic aberration correction */
    val = 0;
    if (vrd->config.chromatic_mode != RK3588_VR_CHROMATIC_NONE) {
        val = RK3588_VOP_VR_CHROMATIC_CTRL_EN;
        
        switch (vrd->config.chromatic_mode) {
        case RK3588_VR_CHROMATIC_RGB:
            val |= RK3588_VOP_VR_CHROMATIC_CTRL_RGB;
            break;
        case RK3588_VR_CHROMATIC_CUSTOM:
            val |= RK3588_VOP_VR_CHROMATIC_CTRL_CUSTOM;
            break;
        default:
            break;
        }
    }
    
    writel(val, vrd->regs + RK3588_VOP_VR_CHROMATIC_CTRL);

    /* Configure VR motion compensation */
    val = 0;
    if (vrd->config.motion_comp_mode != RK3588_VR_MOTION_COMP_NONE) {
        val = RK3588_VOP_VR_MOTION_COMP_EN;
        
        switch (vrd->config.motion_comp_mode) {
        case RK3588_VR_MOTION_COMP_PREDICT:
            val |= RK3588_VOP_VR_MOTION_COMP_PREDICT;
            break;
        case RK3588_VR_MOTION_COMP_EXTRAPOLATE:
            val |= RK3588_VOP_VR_MOTION_COMP_EXTRAPOLATE;
            break;
        default:
            break;
        }
    }
    
    writel(val, vrd->regs + RK3588_VOP_VR_MOTION_COMP);

    /* Configure VR async commit */
    val = RK3588_VOP_VR_ASYNC_COMMIT_EN;
    writel(val, vrd->regs + RK3588_VOP_VR_ASYNC_COMMIT);

    /* Configure VR direct mode */
    val = 0;
    if (vrd->config.mode == RK3588_VR_MODE_DIRECT) {
        val = RK3588_VOP_VR_DIRECT_MODE_EN;
        if (vrd->config.fast_path)
            val |= RK3588_VOP_VR_DIRECT_MODE_FAST_PATH;
    }
    
    writel(val, vrd->regs + RK3588_VOP_VR_DIRECT_MODE);

    /* Initialize VR thread */
    vrd->vr_thread = kthread_create(rk3588_vr_display_thread, vrd, "rk3588-vr-thread");
    if (IS_ERR(vrd->vr_thread)) {
        ret = PTR_ERR(vrd->vr_thread);
        dev_err(vrd->dev, "Failed to create VR thread: %d\n", ret);
        goto err_disable_dclks;
    }

    /* Initialize debugfs */
    ret = rk3588_vr_display_debugfs_init(vrd);
    if (ret) {
        dev_warn(vrd->dev, "Failed to initialize debugfs: %d\n", ret);
        /* Continue anyway */
    }

    vrd->enabled = true;
    vrd->suspended = false;

    dev_info(vrd->dev, "RK3588 VR display driver initialized successfully\n");
    return 0;

err_disable_dclks:
    for (i = 0; i < RK3588_VR_MAX_DISPLAYS; i++) {
        if (vrd->dclk[i])
            clk_disable_unprepare(vrd->dclk[i]);
    }
    clk_disable_unprepare(vrd->aclk);

err_disable_hclk:
    clk_disable_unprepare(vrd->hclk);
    return ret;
}

/**
 * rk3588_vr_display_fini - Finalize the RK3588 VR display driver
 * @vrd: Pointer to the RK3588 VR display device structure
 *
 * This function finalizes the RK3588 VR display driver, cleaning up
 * resources and shutting down the hardware.
 */
void rk3588_vr_display_fini(struct rk3588_vr_display *vrd)
{
    int i;

    if (!vrd)
        return;

    dev_info(vrd->dev, "Finalizing RK3588 VR display driver\n");

    /* Stop VR thread */
    if (vrd->vr_thread && !IS_ERR(vrd->vr_thread)) {
        atomic_set(&vrd->vr_thread_active, 0);
        complete(&vrd->vr_thread_completion);
        kthread_stop(vrd->vr_thread);
    }

    /* Disable hardware */
    writel(0, vrd->regs + RK3588_VOP_SYS_CTRL);

    /* Free distortion and chromatic maps */
    for (i = 0; i < RK3588_VR_MAX_DISPLAYS; i++) {
        if (vrd->distortion_map[i]) {
            dma_free_coherent(vrd->dev, vrd->distortion_map_size[i],
                             vrd->distortion_map[i], vrd->distortion_map_dma[i]);
            vrd->distortion_map[i] = NULL;
        }

        if (vrd->chromatic_map[i]) {
            dma_free_coherent(vrd->dev, vrd->chromatic_map_size[i],
                             vrd->chromatic_map[i], vrd->chromatic_map_dma[i]);
            vrd->chromatic_map[i] = NULL;
        }
    }

    /* Free motion vectors */
    if (vrd->motion_vectors) {
        dma_free_coherent(vrd->dev, vrd->motion_vectors_size,
                         vrd->motion_vectors, vrd->motion_vectors_dma);
        vrd->motion_vectors = NULL;
    }

    /* Cleanup debugfs */
    rk3588_vr_display_debugfs_fini(vrd);

    /* Disable clocks */
    for (i = 0; i < RK3588_VR_MAX_DISPLAYS; i++) {
        if (vrd->dclk[i])
            clk_disable_unprepare(vrd->dclk[i]);
    }

    clk_disable_unprepare(vrd->aclk);
    clk_disable_unprepare(vrd->hclk);

    vrd->enabled = false;
    dev_info(vrd->dev, "RK3588 VR display driver finalized\n");
}

/**
 * rk3588_vr_display_enable - Enable the RK3588 VR display
 * @vrd: Pointer to the RK3588 VR display device structure
 *
 * This function enables the RK3588 VR display hardware and starts
 * the VR thread.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_enable(struct rk3588_vr_display *vrd)
{
    u32 val;

    if (!vrd || !vrd->dev || !vrd->regs)
        return -EINVAL;

    if (vrd->enabled)
        return 0;

    dev_info(vrd->dev, "Enabling RK3588 VR display\n");

    /* Enable hardware */
    val = readl(vrd->regs + RK3588_VOP_SYS_CTRL);
    val |= RK3588_VOP_SYS_CTRL_EN;
    writel(val, vrd->regs + RK3588_VOP_SYS_CTRL);

    /* Start VR thread */
    atomic_set(&vrd->vr_thread_active, 1);
    wake_up_process(vrd->vr_thread);

    vrd->enabled = true;
    return 0;
}

/**
 * rk3588_vr_display_disable - Disable the RK3588 VR display
 * @vrd: Pointer to the RK3588 VR display device structure
 *
 * This function disables the RK3588 VR display hardware and stops
 * the VR thread.
 */
void rk3588_vr_display_disable(struct rk3588_vr_display *vrd)
{
    u32 val;

    if (!vrd || !vrd->dev || !vrd->regs || !vrd->enabled)
        return;

    dev_info(vrd->dev, "Disabling RK3588 VR display\n");

    /* Stop VR thread */
    atomic_set(&vrd->vr_thread_active, 0);
    complete(&vrd->vr_thread_completion);

    /* Disable hardware */
    val = readl(vrd->regs + RK3588_VOP_SYS_CTRL);
    val &= ~RK3588_VOP_SYS_CTRL_EN;
    writel(val, vrd->regs + RK3588_VOP_SYS_CTRL);

    vrd->enabled = false;
}

/**
 * rk3588_vr_display_set_mode - Set the RK3588 VR display mode
 * @vrd: Pointer to the RK3588 VR display device structure
 * @mode: VR display mode to set
 *
 * This function sets the RK3588 VR display mode.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_set_mode(struct rk3588_vr_display *vrd, enum rk3588_vr_display_mode mode)
{
    u32 val;

    if (!vrd || !vrd->dev || !vrd->regs)
        return -EINVAL;

    if (mode >= RK3588_VR_MODE_MAX)
        return -EINVAL;

    dev_info(vrd->dev, "Setting VR display mode to %d\n", mode);

    vrd->config.mode = mode;

    /* Configure VR direct mode */
    val = 0;
    if (mode == RK3588_VR_MODE_DIRECT) {
        val = RK3588_VOP_VR_DIRECT_MODE_EN;
        if (vrd->config.fast_path)
            val |= RK3588_VOP_VR_DIRECT_MODE_FAST_PATH;
    }
    
    writel(val, vrd->regs + RK3588_VOP_VR_DIRECT_MODE);

    /* Configure VR async commit */
    val = 0;
    if (mode == RK3588_VR_MODE_ASYNC)
        val = RK3588_VOP_VR_ASYNC_COMMIT_EN;
    
    writel(val, vrd->regs + RK3588_VOP_VR_ASYNC_COMMIT);

    /* Configure VR low persistence */
    val = 0;
    if (mode == RK3588_VR_MODE_LOW_PERSISTENCE || vrd->config.low_persistence) {
        val = RK3588_VOP_VR_LOW_PERSIST_EN |
              RK3588_VOP_VR_LOW_PERSIST_DUTY(vrd->config.low_persistence_duty);
    }
    
    writel(val, vrd->regs + RK3588_VOP_VR_LOW_PERSIST);

    return 0;
}

/**
 * rk3588_vr_display_set_sync_mode - Set the RK3588 VR display sync mode
 * @vrd: Pointer to the RK3588 VR display device structure
 * @mode: VR display sync mode to set
 *
 * This function sets the RK3588 VR display sync mode.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_set_sync_mode(struct rk3588_vr_display *vrd, enum rk3588_vr_sync_mode mode)
{
    u32 val;

    if (!vrd || !vrd->dev || !vrd->regs)
        return -EINVAL;

    if (mode >= RK3588_VR_SYNC_MAX)
        return -EINVAL;

    dev_info(vrd->dev, "Setting VR display sync mode to %d\n", mode);

    vrd->config.sync_mode = mode;

    /* Configure VR sync mode */
    val = RK3588_VOP_VR_SYNC_CTRL_EN;
    
    switch (mode) {
    case RK3588_VR_SYNC_MASTER:
        val |= RK3588_VOP_VR_SYNC_CTRL_MASTER;
        break;
    case RK3588_VR_SYNC_SLAVE:
        val |= RK3588_VOP_VR_SYNC_CTRL_SLAVE;
        break;
    case RK3588_VR_SYNC_EXTERNAL:
        /* External sync uses both master and slave bits */
        val |= RK3588_VOP_VR_SYNC_CTRL_MASTER | RK3588_VOP_VR_SYNC_CTRL_SLAVE;
        break;
    case RK3588_VR_SYNC_INDEPENDENT:
    default:
        /* No additional bits for independent mode */
        break;
    }
    
    val |= RK3588_VOP_VR_SYNC_CTRL_VSYNC | RK3588_VOP_VR_SYNC_CTRL_HSYNC;
    val |= RK3588_VOP_VR_SYNC_CTRL_PHASE(0);
    
    writel(val, vrd->regs + RK3588_VOP_VR_SYNC_CTRL);

    return 0;
}

/**
 * rk3588_vr_display_set_distortion_mode - Set the RK3588 VR display distortion mode
 * @vrd: Pointer to the RK3588 VR display device structure
 * @mode: VR display distortion mode to set
 *
 * This function sets the RK3588 VR display distortion correction mode.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_set_distortion_mode(struct rk3588_vr_display *vrd, enum rk3588_vr_distortion_mode mode)
{
    u32 val;

    if (!vrd || !vrd->dev || !vrd->regs)
        return -EINVAL;

    if (mode >= RK3588_VR_DISTORTION_MAX)
        return -EINVAL;

    dev_info(vrd->dev, "Setting VR display distortion mode to %d\n", mode);

    vrd->config.distortion_mode = mode;

    /* Configure VR distortion correction */
    val = 0;
    if (mode != RK3588_VR_DISTORTION_NONE) {
        val = RK3588_VOP_VR_DISTORTION_CTRL_EN;
        
        switch (mode) {
        case RK3588_VR_DISTORTION_BARREL:
            val |= RK3588_VOP_VR_DISTORTION_CTRL_BARREL;
            break;
        case RK3588_VR_DISTORTION_PINCUSHION:
            val |= RK3588_VOP_VR_DISTORTION_CTRL_PINCUSHION;
            break;
        case RK3588_VR_DISTORTION_MESH:
            val |= RK3588_VOP_VR_DISTORTION_CTRL_MESH;
            break;
        case RK3588_VR_DISTORTION_CUSTOM:
            val |= RK3588_VOP_VR_DISTORTION_CTRL_CUSTOM;
            break;
        default:
            break;
        }
    }
    
    writel(val, vrd->regs + RK3588_VOP_VR_DISTORTION_CTRL);

    return 0;
}

/**
 * rk3588_vr_display_set_chromatic_mode - Set the RK3588 VR display chromatic mode
 * @vrd: Pointer to the RK3588 VR display device structure
 * @mode: VR display chromatic mode to set
 *
 * This function sets the RK3588 VR display chromatic aberration correction mode.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_set_chromatic_mode(struct rk3588_vr_display *vrd, enum rk3588_vr_chromatic_mode mode)
{
    u32 val;

    if (!vrd || !vrd->dev || !vrd->regs)
        return -EINVAL;

    if (mode >= RK3588_VR_CHROMATIC_MAX)
        return -EINVAL;

    dev_info(vrd->dev, "Setting VR display chromatic mode to %d\n", mode);

    vrd->config.chromatic_mode = mode;

    /* Configure VR chromatic aberration correction */
    val = 0;
    if (mode != RK3588_VR_CHROMATIC_NONE) {
        val = RK3588_VOP_VR_CHROMATIC_CTRL_EN;
        
        switch (mode) {
        case RK3588_VR_CHROMATIC_RGB:
            val |= RK3588_VOP_VR_CHROMATIC_CTRL_RGB;
            break;
        case RK3588_VR_CHROMATIC_CUSTOM:
            val |= RK3588_VOP_VR_CHROMATIC_CTRL_CUSTOM;
            break;
        default:
            break;
        }
    }
    
    writel(val, vrd->regs + RK3588_VOP_VR_CHROMATIC_CTRL);

    return 0;
}

/**
 * rk3588_vr_display_set_motion_comp_mode - Set the RK3588 VR display motion compensation mode
 * @vrd: Pointer to the RK3588 VR display device structure
 * @mode: VR display motion compensation mode to set
 *
 * This function sets the RK3588 VR display motion compensation mode.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_set_motion_comp_mode(struct rk3588_vr_display *vrd, enum rk3588_vr_motion_comp_mode mode)
{
    u32 val;

    if (!vrd || !vrd->dev || !vrd->regs)
        return -EINVAL;

    if (mode >= RK3588_VR_MOTION_COMP_MAX)
        return -EINVAL;

    dev_info(vrd->dev, "Setting VR display motion compensation mode to %d\n", mode);

    vrd->config.motion_comp_mode = mode;

    /* Configure VR motion compensation */
    val = 0;
    if (mode != RK3588_VR_MOTION_COMP_NONE) {
        val = RK3588_VOP_VR_MOTION_COMP_EN;
        
        switch (mode) {
        case RK3588_VR_MOTION_COMP_PREDICT:
            val |= RK3588_VOP_VR_MOTION_COMP_PREDICT;
            break;
        case RK3588_VR_MOTION_COMP_EXTRAPOLATE:
            val |= RK3588_VOP_VR_MOTION_COMP_EXTRAPOLATE;
            break;
        default:
            break;
        }
    }
    
    writel(val, vrd->regs + RK3588_VOP_VR_MOTION_COMP);

    return 0;
}

/**
 * rk3588_vr_display_set_low_persistence - Set the RK3588 VR display low persistence mode
 * @vrd: Pointer to the RK3588 VR display device structure
 * @enable: Whether to enable low persistence mode
 * @duty: Low persistence duty cycle percentage (0-100)
 *
 * This function sets the RK3588 VR display low persistence mode.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_set_low_persistence(struct rk3588_vr_display *vrd, bool enable, u8 duty)
{
    u32 val;

    if (!vrd || !vrd->dev || !vrd->regs)
        return -EINVAL;

    if (duty > 100)
        duty = 100;

    dev_info(vrd->dev, "Setting VR display low persistence mode to %s, duty=%d%%\n",
             enable ? "enabled" : "disabled", duty);

    vrd->config.low_persistence = enable;
    vrd->config.low_persistence_duty = duty;

    /* Configure VR low persistence mode */
    val = 0;
    if (enable) {
        val = RK3588_VOP_VR_LOW_PERSIST_EN |
              RK3588_VOP_VR_LOW_PERSIST_DUTY(duty);
    }
    
    writel(val, vrd->regs + RK3588_VOP_VR_LOW_PERSIST);

    return 0;
}

/**
 * rk3588_vr_display_set_fast_path - Set the RK3588 VR display fast path mode
 * @vrd: Pointer to the RK3588 VR display device structure
 * @enable: Whether to enable fast path mode
 *
 * This function sets the RK3588 VR display fast path mode for reduced latency.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_set_fast_path(struct rk3588_vr_display *vrd, bool enable)
{
    u32 val;

    if (!vrd || !vrd->dev || !vrd->regs)
        return -EINVAL;

    dev_info(vrd->dev, "Setting VR display fast path mode to %s\n",
             enable ? "enabled" : "disabled");

    vrd->config.fast_path = enable;

    /* Configure VR latency control */
    val = readl(vrd->regs + RK3588_VOP_VR_LATENCY_CTRL);
    
    if (enable)
        val |= RK3588_VOP_VR_LATENCY_CTRL_FAST_PATH;
    else
        val &= ~RK3588_VOP_VR_LATENCY_CTRL_FAST_PATH;
    
    writel(val, vrd->regs + RK3588_VOP_VR_LATENCY_CTRL);

    /* Configure VR direct mode if active */
    if (vrd->config.mode == RK3588_VR_MODE_DIRECT) {
        val = readl(vrd->regs + RK3588_VOP_VR_DIRECT_MODE);
        
        if (enable)
            val |= RK3588_VOP_VR_DIRECT_MODE_FAST_PATH;
        else
            val &= ~RK3588_VOP_VR_DIRECT_MODE_FAST_PATH;
        
        writel(val, vrd->regs + RK3588_VOP_VR_DIRECT_MODE);
    }

    return 0;
}

/**
 * rk3588_vr_display_set_bypass_options - Set the RK3588 VR display bypass options
 * @vrd: Pointer to the RK3588 VR display device structure
 * @blend: Whether to bypass blend
 * @scale: Whether to bypass scale
 * @gamma: Whether to bypass gamma
 * @dither: Whether to bypass dither
 *
 * This function sets the RK3588 VR display bypass options for reduced latency.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_set_bypass_options(struct rk3588_vr_display *vrd, bool blend, bool scale, bool gamma, bool dither)
{
    u32 val;

    if (!vrd || !vrd->dev || !vrd->regs)
        return -EINVAL;

    dev_info(vrd->dev, "Setting VR display bypass options: blend=%d, scale=%d, gamma=%d, dither=%d\n",
             blend, scale, gamma, dither);

    vrd->config.bypass_blend = blend;
    vrd->config.bypass_scale = scale;
    vrd->config.bypass_gamma = gamma;
    vrd->config.bypass_dither = dither;

    /* Configure VR latency control */
    val = readl(vrd->regs + RK3588_VOP_VR_LATENCY_CTRL);
    
    if (blend)
        val |= RK3588_VOP_VR_LATENCY_CTRL_BYPASS_BLEND;
    else
        val &= ~RK3588_VOP_VR_LATENCY_CTRL_BYPASS_BLEND;
    
    if (scale)
        val |= RK3588_VOP_VR_LATENCY_CTRL_BYPASS_SCALE;
    else
        val &= ~RK3588_VOP_VR_LATENCY_CTRL_BYPASS_SCALE;
    
    if (gamma)
        val |= RK3588_VOP_VR_LATENCY_CTRL_BYPASS_GAMMA;
    else
        val &= ~RK3588_VOP_VR_LATENCY_CTRL_BYPASS_GAMMA;
    
    if (dither)
        val |= RK3588_VOP_VR_LATENCY_CTRL_BYPASS_DITHER;
    else
        val &= ~RK3588_VOP_VR_LATENCY_CTRL_BYPASS_DITHER;
    
    writel(val, vrd->regs + RK3588_VOP_VR_LATENCY_CTRL);

    return 0;
}

/**
 * rk3588_vr_display_set_distortion_map - Set the RK3588 VR display distortion map
 * @vrd: Pointer to the RK3588 VR display device structure
 * @display_idx: Display index (0 or 1)
 * @map: Pointer to the distortion map data
 * @size: Size of the distortion map data in bytes
 *
 * This function sets the RK3588 VR display distortion map for the specified display.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_set_distortion_map(struct rk3588_vr_display *vrd, int display_idx, const void *map, size_t size)
{
    void *new_map;
    dma_addr_t new_map_dma;

    if (!vrd || !vrd->dev || !map || !size)
        return -EINVAL;

    if (display_idx < 0 || display_idx >= RK3588_VR_MAX_DISPLAYS)
        return -EINVAL;

    dev_info(vrd->dev, "Setting VR display distortion map for display %d, size=%zu bytes\n",
             display_idx, size);

    /* Allocate new map */
    new_map = dma_alloc_coherent(vrd->dev, size, &new_map_dma, GFP_KERNEL);
    if (!new_map)
        return -ENOMEM;

    /* Copy map data */
    memcpy(new_map, map, size);

    /* Free old map if exists */
    if (vrd->distortion_map[display_idx]) {
        dma_free_coherent(vrd->dev, vrd->distortion_map_size[display_idx],
                         vrd->distortion_map[display_idx], vrd->distortion_map_dma[display_idx]);
    }

    /* Set new map */
    vrd->distortion_map[display_idx] = new_map;
    vrd->distortion_map_dma[display_idx] = new_map_dma;
    vrd->distortion_map_size[display_idx] = size;

    /* Configure hardware to use the map */
    if (vrd->config.distortion_mode == RK3588_VR_DISTORTION_MESH ||
        vrd->config.distortion_mode == RK3588_VR_DISTORTION_CUSTOM) {
        /* Set distortion map address */
        writel((u32)new_map_dma, vrd->regs + RK3588_VOP_VR_DISTORTION_COEF);
    }

    return 0;
}

/**
 * rk3588_vr_display_set_chromatic_map - Set the RK3588 VR display chromatic map
 * @vrd: Pointer to the RK3588 VR display device structure
 * @display_idx: Display index (0 or 1)
 * @map: Pointer to the chromatic map data
 * @size: Size of the chromatic map data in bytes
 *
 * This function sets the RK3588 VR display chromatic aberration map for the specified display.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_set_chromatic_map(struct rk3588_vr_display *vrd, int display_idx, const void *map, size_t size)
{
    void *new_map;
    dma_addr_t new_map_dma;

    if (!vrd || !vrd->dev || !map || !size)
        return -EINVAL;

    if (display_idx < 0 || display_idx >= RK3588_VR_MAX_DISPLAYS)
        return -EINVAL;

    dev_info(vrd->dev, "Setting VR display chromatic map for display %d, size=%zu bytes\n",
             display_idx, size);

    /* Allocate new map */
    new_map = dma_alloc_coherent(vrd->dev, size, &new_map_dma, GFP_KERNEL);
    if (!new_map)
        return -ENOMEM;

    /* Copy map data */
    memcpy(new_map, map, size);

    /* Free old map if exists */
    if (vrd->chromatic_map[display_idx]) {
        dma_free_coherent(vrd->dev, vrd->chromatic_map_size[display_idx],
                         vrd->chromatic_map[display_idx], vrd->chromatic_map_dma[display_idx]);
    }

    /* Set new map */
    vrd->chromatic_map[display_idx] = new_map;
    vrd->chromatic_map_dma[display_idx] = new_map_dma;
    vrd->chromatic_map_size[display_idx] = size;

    /* Configure hardware to use the map */
    if (vrd->config.chromatic_mode == RK3588_VR_CHROMATIC_CUSTOM) {
        /* Set chromatic map address */
        writel((u32)new_map_dma, vrd->regs + RK3588_VOP_VR_CHROMATIC_COEF);
    }

    return 0;
}

/**
 * rk3588_vr_display_set_motion_vectors - Set the RK3588 VR display motion vectors
 * @vrd: Pointer to the RK3588 VR display device structure
 * @vectors: Pointer to the motion vectors data
 * @size: Size of the motion vectors data in bytes
 *
 * This function sets the RK3588 VR display motion vectors for motion compensation.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_set_motion_vectors(struct rk3588_vr_display *vrd, const void *vectors, size_t size)
{
    void *new_vectors;
    dma_addr_t new_vectors_dma;

    if (!vrd || !vrd->dev || !vectors || !size)
        return -EINVAL;

    dev_info(vrd->dev, "Setting VR display motion vectors, size=%zu bytes\n", size);

    /* Allocate new vectors */
    new_vectors = dma_alloc_coherent(vrd->dev, size, &new_vectors_dma, GFP_KERNEL);
    if (!new_vectors)
        return -ENOMEM;

    /* Copy vectors data */
    memcpy(new_vectors, vectors, size);

    /* Free old vectors if exists */
    if (vrd->motion_vectors) {
        dma_free_coherent(vrd->dev, vrd->motion_vectors_size,
                         vrd->motion_vectors, vrd->motion_vectors_dma);
    }

    /* Set new vectors */
    vrd->motion_vectors = new_vectors;
    vrd->motion_vectors_dma = new_vectors_dma;
    vrd->motion_vectors_size = size;

    /* Configure hardware to use the vectors */
    if (vrd->config.motion_comp_mode != RK3588_VR_MOTION_COMP_NONE) {
        /* Set motion vectors address */
        writel((u32)new_vectors_dma, vrd->regs + RK3588_VOP_VR_MOTION_VECTOR);
    }

    return 0;
}

/**
 * rk3588_vr_display_wait_for_vsync - Wait for vsync on the specified display
 * @vrd: Pointer to the RK3588 VR display device structure
 * @display_idx: Display index (0 or 1)
 *
 * This function waits for a vsync event on the specified display.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_wait_for_vsync(struct rk3588_vr_display *vrd, int display_idx)
{
    int ret;

    if (!vrd || !vrd->dev)
        return -EINVAL;

    if (display_idx < 0 || display_idx >= RK3588_VR_MAX_DISPLAYS)
        return -EINVAL;

    /* Wait for vsync completion */
    ret = wait_for_completion_timeout(&vrd->vsync_completion[display_idx],
                                     msecs_to_jiffies(100));
    if (ret == 0)
        return -ETIMEDOUT;

    return 0;
}

/**
 * rk3588_vr_display_wait_for_commit - Wait for commit on the specified display
 * @vrd: Pointer to the RK3588 VR display device structure
 * @display_idx: Display index (0 or 1)
 *
 * This function waits for a commit completion on the specified display.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_wait_for_commit(struct rk3588_vr_display *vrd, int display_idx)
{
    int ret;

    if (!vrd || !vrd->dev)
        return -EINVAL;

    if (display_idx < 0 || display_idx >= RK3588_VR_MAX_DISPLAYS)
        return -EINVAL;

    /* Wait for commit completion */
    ret = wait_for_completion_timeout(&vrd->commit_completion[display_idx],
                                     msecs_to_jiffies(100));
    if (ret == 0)
        return -ETIMEDOUT;

    return 0;
}

/**
 * rk3588_vr_display_get_commit_latency - Get commit latency for the specified display
 * @vrd: Pointer to the RK3588 VR display device structure
 * @display_idx: Display index (0 or 1)
 * @latency_us: Pointer to store the latency in microseconds
 *
 * This function gets the commit latency for the specified display.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_get_commit_latency(struct rk3588_vr_display *vrd, int display_idx, u32 *latency_us)
{
    if (!vrd || !vrd->dev || !latency_us)
        return -EINVAL;

    if (display_idx < 0 || display_idx >= RK3588_VR_MAX_DISPLAYS)
        return -EINVAL;

    *latency_us = vrd->commit_latency_us[display_idx];
    return 0;
}

/**
 * rk3588_vr_display_get_vsync_period - Get vsync period for the specified display
 * @vrd: Pointer to the RK3588 VR display device structure
 * @display_idx: Display index (0 or 1)
 * @period_us: Pointer to store the period in microseconds
 *
 * This function gets the vsync period for the specified display.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_get_vsync_period(struct rk3588_vr_display *vrd, int display_idx, u32 *period_us)
{
    if (!vrd || !vrd->dev || !period_us)
        return -EINVAL;

    if (display_idx < 0 || display_idx >= RK3588_VR_MAX_DISPLAYS)
        return -EINVAL;

    *period_us = vrd->vsync_period_us[display_idx];
    return 0;
}

/**
 * rk3588_vr_display_thread - VR display thread function
 * @data: Pointer to the RK3588 VR display device structure
 *
 * This function is the main thread for the RK3588 VR display driver.
 * It handles vsync events, commit completions, and other periodic tasks.
 *
 * Return: 0 on success
 */
static int rk3588_vr_display_thread(void *data)
{
    struct rk3588_vr_display *vrd = data;
    int i;
    u32 intr_status;
    ktime_t now;

    if (!vrd || !vrd->dev || !vrd->regs)
        return -EINVAL;

    dev_info(vrd->dev, "VR display thread started\n");

    while (!kthread_should_stop()) {
        /* Check if thread should be active */
        if (!atomic_read(&vrd->vr_thread_active)) {
            /* Wait for activation */
            wait_for_completion_interruptible(&vrd->vr_thread_completion);
            if (kthread_should_stop())
                break;
            continue;
        }

        /* Check for interrupts */
        intr_status = readl(vrd->regs + RK3588_VOP_INTR_STATUS);
        if (intr_status) {
            /* Clear interrupts */
            writel(intr_status, vrd->regs + RK3588_VOP_INTR_CLEAR);

            /* Handle vsync interrupts */
            for (i = 0; i < RK3588_VR_MAX_DISPLAYS; i++) {
                if (intr_status & (1 << i)) {
                    /* Handle vsync for this display */
                    rk3588_vr_display_handle_vsync(vrd, i);
                }
            }

            /* Handle commit interrupts */
            for (i = 0; i < RK3588_VR_MAX_DISPLAYS; i++) {
                if (intr_status & (1 << (i + 8))) {
                    /* Handle commit for this display */
                    rk3588_vr_display_handle_commit(vrd, i);
                }
            }
        }

        /* Perform periodic tasks */
        now = ktime_get();
        for (i = 0; i < RK3588_VR_MAX_DISPLAYS; i++) {
            /* Check if vsync period needs update */
            if (vrd->last_vsync[i] && ktime_to_us(ktime_sub(now, vrd->last_vsync[i])) > 1000000) {
                /* No vsync for 1 second, reset period */
                vrd->vsync_period_us[i] = 1000000 / vrd->config.target_vrefresh;
            }
        }

        /* Sleep for a short time */
        usleep_range(1000, 2000);
    }

    dev_info(vrd->dev, "VR display thread stopped\n");
    return 0;
}

/**
 * rk3588_vr_display_handle_vsync - Handle vsync event for the specified display
 * @vrd: Pointer to the RK3588 VR display device structure
 * @display_idx: Display index (0 or 1)
 *
 * This function handles a vsync event for the specified display.
 */
static void rk3588_vr_display_handle_vsync(struct rk3588_vr_display *vrd, int display_idx)
{
    ktime_t now = ktime_get();
    ktime_t diff;
    u64 diff_us;

    if (!vrd || display_idx < 0 || display_idx >= RK3588_VR_MAX_DISPLAYS)
        return;

    /* Update frame counter */
    vrd->frame_counter[display_idx]++;

    /* Calculate vsync period */
    if (vrd->last_vsync[display_idx]) {
        diff = ktime_sub(now, vrd->last_vsync[display_idx]);
        diff_us = ktime_to_us(diff);
        
        /* Update vsync period with exponential moving average */
        vrd->vsync_period_us[display_idx] = (vrd->vsync_period_us[display_idx] * 7 + diff_us) / 8;
    }

    /* Update last vsync time */
    vrd->last_vsync[display_idx] = now;

    /* Signal vsync completion */
    complete(&vrd->vsync_completion[display_idx]);
}

/**
 * rk3588_vr_display_handle_commit - Handle commit completion for the specified display
 * @vrd: Pointer to the RK3588 VR display device structure
 * @display_idx: Display index (0 or 1)
 *
 * This function handles a commit completion event for the specified display.
 */
static void rk3588_vr_display_handle_commit(struct rk3588_vr_display *vrd, int display_idx)
{
    ktime_t now = ktime_get();
    ktime_t diff;
    u64 diff_us;

    if (!vrd || display_idx < 0 || display_idx >= RK3588_VR_MAX_DISPLAYS)
        return;

    /* Calculate commit latency */
    if (vrd->last_commit[display_idx]) {
        diff = ktime_sub(now, vrd->last_commit[display_idx]);
        diff_us = ktime_to_us(diff);
        
        /* Update commit latency with exponential moving average */
        vrd->commit_latency_us[display_idx] = (vrd->commit_latency_us[display_idx] * 7 + diff_us) / 8;
    }

    /* Update last commit time */
    vrd->last_commit[display_idx] = now;

    /* Signal commit completion */
    complete(&vrd->commit_completion[display_idx]);
}

/**
 * rk3588_vr_display_debugfs_init - Initialize debugfs for the RK3588 VR display
 * @vrd: Pointer to the RK3588 VR display device structure
 *
 * This function initializes debugfs for the RK3588 VR display driver.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_debugfs_init(struct rk3588_vr_display *vrd)
{
    /* Implementation omitted for brevity */
    return 0;
}

/**
 * rk3588_vr_display_debugfs_fini - Finalize debugfs for the RK3588 VR display
 * @vrd: Pointer to the RK3588 VR display device structure
 *
 * This function finalizes debugfs for the RK3588 VR display driver.
 */
void rk3588_vr_display_debugfs_fini(struct rk3588_vr_display *vrd)
{
    /* Implementation omitted for brevity */
}

/**
 * rk3588_vr_display_suspend - Suspend the RK3588 VR display
 * @vrd: Pointer to the RK3588 VR display device structure
 *
 * This function suspends the RK3588 VR display driver.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_suspend(struct rk3588_vr_display *vrd)
{
    if (!vrd || !vrd->dev || !vrd->regs)
        return -EINVAL;

    if (vrd->suspended)
        return 0;

    dev_info(vrd->dev, "Suspending RK3588 VR display\n");

    /* Disable display */
    rk3588_vr_display_disable(vrd);

    vrd->suspended = true;
    return 0;
}

/**
 * rk3588_vr_display_resume - Resume the RK3588 VR display
 * @vrd: Pointer to the RK3588 VR display device structure
 *
 * This function resumes the RK3588 VR display driver.
 *
 * Return: 0 on success, negative error code on failure
 */
int rk3588_vr_display_resume(struct rk3588_vr_display *vrd)
{
    if (!vrd || !vrd->dev || !vrd->regs)
        return -EINVAL;

    if (!vrd->suspended)
        return 0;

    dev_info(vrd->dev, "Resuming RK3588 VR display\n");

    /* Enable display */
    rk3588_vr_display_enable(vrd);

    vrd->suspended = false;
    return 0;
}

/* Module initialization and cleanup */
MODULE_AUTHOR("VR Headset Project");
MODULE_DESCRIPTION("RK3588 VR Display Driver");
MODULE_LICENSE("GPL v2");
MODULE_VERSION("1.0");
