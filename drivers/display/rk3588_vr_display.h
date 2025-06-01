/* SPDX-License-Identifier: GPL-2.0
 *
 * RK3588 VR Display Driver - Core Header
 *
 * Copyright (C) 2025 VR Headset Project
 *
 * This driver provides DRM/KMS support for RK3588 display controller
 * with specific optimizations for VR headset dual-display operation.
 */

#ifndef RK3588_VR_DISPLAY_H
#define RK3588_VR_DISPLAY_H

#include <linux/module.h>
#include <linux/platform_device.h>
#include <linux/of.h>
#include <linux/of_device.h>
#include <linux/of_address.h>
#include <linux/of_irq.h>
#include <linux/clk.h>
#include <linux/reset.h>
#include <linux/regmap.h>
#include <linux/pm_runtime.h>
#include <linux/dma-mapping.h>
#include <linux/component.h>
#include <linux/iommu.h>
#include <linux/delay.h>
#include <linux/slab.h>
#include <linux/kthread.h>
#include <linux/completion.h>
#include <linux/debugfs.h>

#include <drm/drm_atomic.h>
#include <drm/drm_atomic_helper.h>
#include <drm/drm_bridge.h>
#include <drm/drm_connector.h>
#include <drm/drm_crtc.h>
#include <drm/drm_encoder.h>
#include <drm/drm_fb_helper.h>
#include <drm/drm_fourcc.h>
#include <drm/drm_gem_dma_helper.h>
#include <drm/drm_gem_framebuffer_helper.h>
#include <drm/drm_managed.h>
#include <drm/drm_of.h>
#include <drm/drm_panel.h>
#include <drm/drm_plane.h>
#include <drm/drm_probe_helper.h>
#include <drm/drm_vblank.h>

/* VR-specific defines */
#define RK3588_VR_MAX_DISPLAYS          2
#define RK3588_VR_MAX_PLANES            4
#define RK3588_VR_MAX_LAYERS            8
#define RK3588_VR_MAX_FB_SIZE           (4096 * 2160 * 4)
#define RK3588_VR_MIN_VREFRESH          60
#define RK3588_VR_MAX_VREFRESH          120
#define RK3588_VR_TARGET_VREFRESH       90
#define RK3588_VR_LOW_PERSISTENCE_DUTY  70  /* 70% duty cycle for low persistence */
#define RK3588_VR_MAX_LATENCY_US        5000 /* 5ms max motion-to-photon latency */

/* Display controller registers */
#define RK3588_VOP_SYS_CTRL             0x0000
#define RK3588_VOP_SYS_STATUS           0x0008
#define RK3588_VOP_INTR_EN              0x0010
#define RK3588_VOP_INTR_STATUS          0x0018
#define RK3588_VOP_INTR_CLEAR           0x0020
#define RK3588_VOP_GLOBAL_CTRL          0x0030
#define RK3588_VOP_DSP_CTRL             0x0040
#define RK3588_VOP_DSP_BG_COLOR         0x0048
#define RK3588_VOP_DSP_SIZE             0x0050
#define RK3588_VOP_POST_SCL_CTRL        0x0060
#define RK3588_VOP_POST_SCL_FACTOR      0x0068
#define RK3588_VOP_CLUSTER_CTRL         0x0070
#define RK3588_VOP_CLUSTER_DLY          0x0078
#define RK3588_VOP_SMART_DITHER         0x0080
#define RK3588_VOP_GAMMA_LUT_CTRL       0x0090
#define RK3588_VOP_GAMMA_LUT_DATA       0x0098
#define RK3588_VOP_DSP_CTRL1            0x00A0
#define RK3588_VOP_DSP_BG_COLOR1        0x00A8
#define RK3588_VOP_DSP_SIZE1            0x00B0
#define RK3588_VOP_POST_SCL_CTRL1       0x00C0
#define RK3588_VOP_POST_SCL_FACTOR1     0x00C8
#define RK3588_VOP_CLUSTER_CTRL1        0x00D0
#define RK3588_VOP_CLUSTER_DLY1         0x00D8
#define RK3588_VOP_SMART_DITHER1        0x00E0
#define RK3588_VOP_GAMMA_LUT_CTRL1      0x00F0
#define RK3588_VOP_GAMMA_LUT_DATA1      0x00F8

/* VR-specific registers */
#define RK3588_VOP_VR_SYNC_CTRL         0x0100
#define RK3588_VOP_VR_LOW_PERSIST       0x0108
#define RK3588_VOP_VR_LATENCY_CTRL      0x0110
#define RK3588_VOP_VR_DISTORTION_CTRL   0x0118
#define RK3588_VOP_VR_DISTORTION_COEF   0x0120
#define RK3588_VOP_VR_CHROMATIC_CTRL    0x0128
#define RK3588_VOP_VR_CHROMATIC_COEF    0x0130
#define RK3588_VOP_VR_MOTION_COMP       0x0138
#define RK3588_VOP_VR_MOTION_VECTOR     0x0140
#define RK3588_VOP_VR_ASYNC_COMMIT      0x0148
#define RK3588_VOP_VR_DIRECT_MODE       0x0150

/* Plane registers (per plane) */
#define RK3588_VOP_PLANE_CTRL(i)        (0x1000 + (i) * 0x100)
#define RK3588_VOP_PLANE_SRC_INFO(i)    (0x1008 + (i) * 0x100)
#define RK3588_VOP_PLANE_DST_INFO(i)    (0x1010 + (i) * 0x100)
#define RK3588_VOP_PLANE_SRC_ALPHA(i)   (0x1018 + (i) * 0x100)
#define RK3588_VOP_PLANE_OFFSET(i)      (0x1020 + (i) * 0x100)
#define RK3588_VOP_PLANE_ADDR_BASE(i)   (0x1028 + (i) * 0x100)
#define RK3588_VOP_PLANE_ADDR_OFFSET(i) (0x1030 + (i) * 0x100)
#define RK3588_VOP_PLANE_STRIDE(i)      (0x1038 + (i) * 0x100)
#define RK3588_VOP_PLANE_SCL_CTRL(i)    (0x1040 + (i) * 0x100)
#define RK3588_VOP_PLANE_SCL_FACTOR(i)  (0x1048 + (i) * 0x100)
#define RK3588_VOP_PLANE_AFBC_CTRL(i)   (0x1050 + (i) * 0x100)
#define RK3588_VOP_PLANE_AFBC_HDR(i)    (0x1058 + (i) * 0x100)

/* VR-specific plane registers */
#define RK3588_VOP_PLANE_VR_CTRL(i)     (0x1060 + (i) * 0x100)
#define RK3588_VOP_PLANE_VR_WARP(i)     (0x1068 + (i) * 0x100)
#define RK3588_VOP_PLANE_VR_MESH(i)     (0x1070 + (i) * 0x100)

/* Register bit definitions */
#define RK3588_VOP_SYS_CTRL_EN          BIT(0)
#define RK3588_VOP_SYS_CTRL_STANDBY     BIT(1)
#define RK3588_VOP_SYS_CTRL_RESET       BIT(2)
#define RK3588_VOP_SYS_CTRL_AUTO_GATING BIT(3)
#define RK3588_VOP_SYS_CTRL_OVERLAY     BIT(4)
#define RK3588_VOP_SYS_CTRL_CORE_CLK_EN BIT(5)
#define RK3588_VOP_SYS_CTRL_DCLK_EN     BIT(6)
#define RK3588_VOP_SYS_CTRL_MMU_EN      BIT(7)
#define RK3588_VOP_SYS_CTRL_AXI_OUTSTANDING_MAX(x) (((x) & 0xF) << 8)
#define RK3588_VOP_SYS_CTRL_GLOBAL_REGDONE BIT(12)

/* VR-specific register bit definitions */
#define RK3588_VOP_VR_SYNC_CTRL_EN      BIT(0)
#define RK3588_VOP_VR_SYNC_CTRL_MASTER  BIT(1)
#define RK3588_VOP_VR_SYNC_CTRL_SLAVE   BIT(2)
#define RK3588_VOP_VR_SYNC_CTRL_VSYNC   BIT(3)
#define RK3588_VOP_VR_SYNC_CTRL_HSYNC   BIT(4)
#define RK3588_VOP_VR_SYNC_CTRL_PHASE(x) (((x) & 0xFF) << 8)

#define RK3588_VOP_VR_LOW_PERSIST_EN    BIT(0)
#define RK3588_VOP_VR_LOW_PERSIST_DUTY(x) (((x) & 0xFF) << 8)

#define RK3588_VOP_VR_LATENCY_CTRL_EN   BIT(0)
#define RK3588_VOP_VR_LATENCY_CTRL_FAST_PATH BIT(1)
#define RK3588_VOP_VR_LATENCY_CTRL_BYPASS_BLEND BIT(2)
#define RK3588_VOP_VR_LATENCY_CTRL_BYPASS_SCALE BIT(3)
#define RK3588_VOP_VR_LATENCY_CTRL_BYPASS_GAMMA BIT(4)
#define RK3588_VOP_VR_LATENCY_CTRL_BYPASS_DITHER BIT(5)

#define RK3588_VOP_VR_DISTORTION_CTRL_EN BIT(0)
#define RK3588_VOP_VR_DISTORTION_CTRL_MESH BIT(1)
#define RK3588_VOP_VR_DISTORTION_CTRL_BARREL BIT(2)
#define RK3588_VOP_VR_DISTORTION_CTRL_PINCUSHION BIT(3)
#define RK3588_VOP_VR_DISTORTION_CTRL_CUSTOM BIT(4)

#define RK3588_VOP_VR_CHROMATIC_CTRL_EN BIT(0)
#define RK3588_VOP_VR_CHROMATIC_CTRL_RGB BIT(1)
#define RK3588_VOP_VR_CHROMATIC_CTRL_CUSTOM BIT(2)

#define RK3588_VOP_VR_MOTION_COMP_EN    BIT(0)
#define RK3588_VOP_VR_MOTION_COMP_PREDICT BIT(1)
#define RK3588_VOP_VR_MOTION_COMP_EXTRAPOLATE BIT(2)

#define RK3588_VOP_VR_ASYNC_COMMIT_EN   BIT(0)
#define RK3588_VOP_VR_ASYNC_COMMIT_READY BIT(1)
#define RK3588_VOP_VR_ASYNC_COMMIT_TRIGGER BIT(2)

#define RK3588_VOP_VR_DIRECT_MODE_EN    BIT(0)
#define RK3588_VOP_VR_DIRECT_MODE_BYPASS BIT(1)
#define RK3588_VOP_VR_DIRECT_MODE_FAST_PATH BIT(2)

/* VR display operation modes */
enum rk3588_vr_display_mode {
    RK3588_VR_MODE_NORMAL = 0,
    RK3588_VR_MODE_LOW_PERSISTENCE,
    RK3588_VR_MODE_DIRECT,
    RK3588_VR_MODE_ASYNC,
    RK3588_VR_MODE_MAX
};

/* VR display synchronization modes */
enum rk3588_vr_sync_mode {
    RK3588_VR_SYNC_INDEPENDENT = 0,
    RK3588_VR_SYNC_MASTER,
    RK3588_VR_SYNC_SLAVE,
    RK3588_VR_SYNC_EXTERNAL,
    RK3588_VR_SYNC_MAX
};

/* VR display distortion correction modes */
enum rk3588_vr_distortion_mode {
    RK3588_VR_DISTORTION_NONE = 0,
    RK3588_VR_DISTORTION_BARREL,
    RK3588_VR_DISTORTION_PINCUSHION,
    RK3588_VR_DISTORTION_MESH,
    RK3588_VR_DISTORTION_CUSTOM,
    RK3588_VR_DISTORTION_MAX
};

/* VR display chromatic aberration correction modes */
enum rk3588_vr_chromatic_mode {
    RK3588_VR_CHROMATIC_NONE = 0,
    RK3588_VR_CHROMATIC_RGB,
    RK3588_VR_CHROMATIC_CUSTOM,
    RK3588_VR_CHROMATIC_MAX
};

/* VR display motion compensation modes */
enum rk3588_vr_motion_comp_mode {
    RK3588_VR_MOTION_COMP_NONE = 0,
    RK3588_VR_MOTION_COMP_PREDICT,
    RK3588_VR_MOTION_COMP_EXTRAPOLATE,
    RK3588_VR_MOTION_COMP_MAX
};

/* VR display configuration */
struct rk3588_vr_display_config {
    enum rk3588_vr_display_mode mode;
    enum rk3588_vr_sync_mode sync_mode;
    enum rk3588_vr_distortion_mode distortion_mode;
    enum rk3588_vr_chromatic_mode chromatic_mode;
    enum rk3588_vr_motion_comp_mode motion_comp_mode;
    bool low_persistence;
    u8 low_persistence_duty;
    bool fast_path;
    bool bypass_blend;
    bool bypass_scale;
    bool bypass_gamma;
    bool bypass_dither;
    u32 target_vrefresh;
    u32 max_latency_us;
};

/* VR display device */
struct rk3588_vr_display {
    struct drm_device *drm;
    struct device *dev;
    
    void __iomem *regs;
    struct regmap *grf;
    
    struct clk *hclk;
    struct clk *aclk;
    struct clk *dclk[RK3588_VR_MAX_DISPLAYS];
    struct reset_control *rstc;
    
    struct drm_crtc crtc[RK3588_VR_MAX_DISPLAYS];
    struct drm_encoder encoder[RK3588_VR_MAX_DISPLAYS];
    struct drm_connector *connector[RK3588_VR_MAX_DISPLAYS];
    struct drm_plane primary[RK3588_VR_MAX_DISPLAYS];
    struct drm_plane overlay[RK3588_VR_MAX_PLANES];
    
    struct rk3588_vr_display_config config;
    
    bool enabled;
    bool suspended;
    
    struct completion vsync_completion[RK3588_VR_MAX_DISPLAYS];
    struct completion commit_completion[RK3588_VR_MAX_DISPLAYS];
    
    struct dentry *debugfs;
    
    /* VR-specific members */
    struct task_struct *vr_thread;
    struct completion vr_thread_completion;
    atomic_t vr_thread_active;
    
    void *distortion_map[RK3588_VR_MAX_DISPLAYS];
    dma_addr_t distortion_map_dma[RK3588_VR_MAX_DISPLAYS];
    size_t distortion_map_size[RK3588_VR_MAX_DISPLAYS];
    
    void *chromatic_map[RK3588_VR_MAX_DISPLAYS];
    dma_addr_t chromatic_map_dma[RK3588_VR_MAX_DISPLAYS];
    size_t chromatic_map_size[RK3588_VR_MAX_DISPLAYS];
    
    void *motion_vectors;
    dma_addr_t motion_vectors_dma;
    size_t motion_vectors_size;
    
    u64 frame_counter[RK3588_VR_MAX_DISPLAYS];
    ktime_t last_vsync[RK3588_VR_MAX_DISPLAYS];
    ktime_t last_commit[RK3588_VR_MAX_DISPLAYS];
    u32 commit_latency_us[RK3588_VR_MAX_DISPLAYS];
    u32 vsync_period_us[RK3588_VR_MAX_DISPLAYS];
    
    spinlock_t lock;
};

/* Function prototypes */
int rk3588_vr_display_init(struct rk3588_vr_display *vrd);
void rk3588_vr_display_fini(struct rk3588_vr_display *vrd);

int rk3588_vr_display_enable(struct rk3588_vr_display *vrd);
void rk3588_vr_display_disable(struct rk3588_vr_display *vrd);

int rk3588_vr_display_set_mode(struct rk3588_vr_display *vrd, enum rk3588_vr_display_mode mode);
int rk3588_vr_display_set_sync_mode(struct rk3588_vr_display *vrd, enum rk3588_vr_sync_mode mode);
int rk3588_vr_display_set_distortion_mode(struct rk3588_vr_display *vrd, enum rk3588_vr_distortion_mode mode);
int rk3588_vr_display_set_chromatic_mode(struct rk3588_vr_display *vrd, enum rk3588_vr_chromatic_mode mode);
int rk3588_vr_display_set_motion_comp_mode(struct rk3588_vr_display *vrd, enum rk3588_vr_motion_comp_mode mode);

int rk3588_vr_display_set_low_persistence(struct rk3588_vr_display *vrd, bool enable, u8 duty);
int rk3588_vr_display_set_fast_path(struct rk3588_vr_display *vrd, bool enable);
int rk3588_vr_display_set_bypass_options(struct rk3588_vr_display *vrd, bool blend, bool scale, bool gamma, bool dither);

int rk3588_vr_display_set_distortion_map(struct rk3588_vr_display *vrd, int display_idx, const void *map, size_t size);
int rk3588_vr_display_set_chromatic_map(struct rk3588_vr_display *vrd, int display_idx, const void *map, size_t size);
int rk3588_vr_display_set_motion_vectors(struct rk3588_vr_display *vrd, const void *vectors, size_t size);

int rk3588_vr_display_wait_for_vsync(struct rk3588_vr_display *vrd, int display_idx);
int rk3588_vr_display_wait_for_commit(struct rk3588_vr_display *vrd, int display_idx);

int rk3588_vr_display_get_commit_latency(struct rk3588_vr_display *vrd, int display_idx, u32 *latency_us);
int rk3588_vr_display_get_vsync_period(struct rk3588_vr_display *vrd, int display_idx, u32 *period_us);

int rk3588_vr_display_debugfs_init(struct rk3588_vr_display *vrd);
void rk3588_vr_display_debugfs_fini(struct rk3588_vr_display *vrd);

/* DRM driver functions */
int rk3588_vr_display_drm_init(struct rk3588_vr_display *vrd);
void rk3588_vr_display_drm_fini(struct rk3588_vr_display *vrd);

int rk3588_vr_display_suspend(struct rk3588_vr_display *vrd);
int rk3588_vr_display_resume(struct rk3588_vr_display *vrd);

#endif /* RK3588_VR_DISPLAY_H */
