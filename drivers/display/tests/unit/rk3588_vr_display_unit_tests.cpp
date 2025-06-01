#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <vector>
#include <random>
#include <chrono>

/* Mock includes to replace kernel headers */
#include "mock_kernel.h"
#include "mock_drm.h"

/* Include driver header with special define to handle kernel dependencies */
#define UNIT_TESTING
#include "../rk3588_vr_display.h"

using ::testing::_;
using ::testing::Return;
using ::testing::Invoke;
using ::testing::DoAll;
using ::testing::SetArgPointee;

class RK3588VRDisplayUnitTest : public ::testing::Test {
protected:
    void SetUp() override {
        /* Initialize mock device */
        dev = (struct device*)malloc(sizeof(struct device));
        memset(dev, 0, sizeof(struct device));
        
        /* Initialize mock registers */
        regs = (void*)malloc(0x10000);
        memset(regs, 0, 0x10000);
        
        /* Initialize VR display device */
        vrd = (struct rk3588_vr_display*)malloc(sizeof(struct rk3588_vr_display));
        memset(vrd, 0, sizeof(struct rk3588_vr_display));
        
        vrd->dev = dev;
        vrd->regs = regs;
        
        /* Initialize clocks */
        vrd->hclk = (struct clk*)malloc(sizeof(struct clk));
        vrd->aclk = (struct clk*)malloc(sizeof(struct clk));
        
        for (int i = 0; i < RK3588_VR_MAX_DISPLAYS; i++) {
            vrd->dclk[i] = (struct clk*)malloc(sizeof(struct clk));
        }
        
        /* Initialize reset control */
        vrd->rstc = (struct reset_control*)malloc(sizeof(struct reset_control));
        
        /* Initialize random number generator */
        rng.seed(std::chrono::system_clock::now().time_since_epoch().count());
    }
    
    void TearDown() override {
        /* Free reset control */
        free(vrd->rstc);
        
        /* Free clocks */
        for (int i = 0; i < RK3588_VR_MAX_DISPLAYS; i++) {
            free(vrd->dclk[i]);
        }
        
        free(vrd->aclk);
        free(vrd->hclk);
        
        /* Free VR display device */
        free(vrd);
        
        /* Free registers */
        free(regs);
        
        /* Free device */
        free(dev);
    }
    
    struct device *dev;
    void *regs;
    struct rk3588_vr_display *vrd;
    
    std::mt19937 rng;
};

/* Test initialization */
TEST_F(RK3588VRDisplayUnitTest, InitializationTest) {
    /* Call initialization function */
    int ret = rk3588_vr_display_init(vrd);
    
    /* Verify initialization was successful */
    EXPECT_EQ(ret, 0);
    
    /* Verify system control register was set correctly */
    u32 sys_ctrl = *(u32*)(regs + RK3588_VOP_SYS_CTRL);
    EXPECT_TRUE(sys_ctrl & RK3588_VOP_SYS_CTRL_EN);
    EXPECT_TRUE(sys_ctrl & RK3588_VOP_SYS_CTRL_CORE_CLK_EN);
    EXPECT_TRUE(sys_ctrl & RK3588_VOP_SYS_CTRL_DCLK_EN);
    EXPECT_TRUE(sys_ctrl & RK3588_VOP_SYS_CTRL_MMU_EN);
    EXPECT_TRUE(sys_ctrl & RK3588_VOP_SYS_CTRL_GLOBAL_REGDONE);
    
    /* Verify VR sync control register was set correctly */
    u32 sync_ctrl = *(u32*)(regs + RK3588_VOP_VR_SYNC_CTRL);
    EXPECT_TRUE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_EN);
    EXPECT_TRUE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_MASTER);
    EXPECT_TRUE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_VSYNC);
    EXPECT_TRUE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_HSYNC);
    
    /* Verify VR low persistence register was set correctly */
    u32 low_persist = *(u32*)(regs + RK3588_VOP_VR_LOW_PERSIST);
    EXPECT_TRUE(low_persist & RK3588_VOP_VR_LOW_PERSIST_EN);
    EXPECT_EQ((low_persist >> 8) & 0xFF, RK3588_VR_LOW_PERSISTENCE_DUTY);
    
    /* Verify VR latency control register was set correctly */
    u32 latency_ctrl = *(u32*)(regs + RK3588_VOP_VR_LATENCY_CTRL);
    EXPECT_TRUE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_EN);
    EXPECT_TRUE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_FAST_PATH);
    
    /* Verify VR distortion control register was set correctly */
    u32 distortion_ctrl = *(u32*)(regs + RK3588_VOP_VR_DISTORTION_CTRL);
    EXPECT_TRUE(distortion_ctrl & RK3588_VOP_VR_DISTORTION_CTRL_EN);
    EXPECT_TRUE(distortion_ctrl & RK3588_VOP_VR_DISTORTION_CTRL_BARREL);
    
    /* Verify VR chromatic control register was set correctly */
    u32 chromatic_ctrl = *(u32*)(regs + RK3588_VOP_VR_CHROMATIC_CTRL);
    EXPECT_TRUE(chromatic_ctrl & RK3588_VOP_VR_CHROMATIC_CTRL_EN);
    EXPECT_TRUE(chromatic_ctrl & RK3588_VOP_VR_CHROMATIC_CTRL_RGB);
    
    /* Verify VR motion compensation register was set correctly */
    u32 motion_comp = *(u32*)(regs + RK3588_VOP_VR_MOTION_COMP);
    EXPECT_TRUE(motion_comp & RK3588_VOP_VR_MOTION_COMP_EN);
    EXPECT_TRUE(motion_comp & RK3588_VOP_VR_MOTION_COMP_PREDICT);
    
    /* Verify VR async commit register was set correctly */
    u32 async_commit = *(u32*)(regs + RK3588_VOP_VR_ASYNC_COMMIT);
    EXPECT_TRUE(async_commit & RK3588_VOP_VR_ASYNC_COMMIT_EN);
    
    /* Verify VR direct mode register was set correctly */
    u32 direct_mode = *(u32*)(regs + RK3588_VOP_VR_DIRECT_MODE);
    EXPECT_EQ(direct_mode, 0); /* Normal mode by default */
    
    /* Verify enabled flag was set */
    EXPECT_TRUE(vrd->enabled);
    EXPECT_FALSE(vrd->suspended);
    
    /* Finalize */
    rk3588_vr_display_fini(vrd);
}

/* Test display mode setting */
TEST_F(RK3588VRDisplayUnitTest, DisplayModeTest) {
    /* Initialize */
    int ret = rk3588_vr_display_init(vrd);
    EXPECT_EQ(ret, 0);
    
    /* Test normal mode */
    ret = rk3588_vr_display_set_mode(vrd, RK3588_VR_MODE_NORMAL);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.mode, RK3588_VR_MODE_NORMAL);
    
    u32 direct_mode = *(u32*)(regs + RK3588_VOP_VR_DIRECT_MODE);
    EXPECT_EQ(direct_mode, 0);
    
    u32 async_commit = *(u32*)(regs + RK3588_VOP_VR_ASYNC_COMMIT);
    EXPECT_TRUE(async_commit & RK3588_VOP_VR_ASYNC_COMMIT_EN);
    
    /* Test low persistence mode */
    ret = rk3588_vr_display_set_mode(vrd, RK3588_VR_MODE_LOW_PERSISTENCE);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.mode, RK3588_VR_MODE_LOW_PERSISTENCE);
    
    u32 low_persist = *(u32*)(regs + RK3588_VOP_VR_LOW_PERSIST);
    EXPECT_TRUE(low_persist & RK3588_VOP_VR_LOW_PERSIST_EN);
    EXPECT_EQ((low_persist >> 8) & 0xFF, RK3588_VR_LOW_PERSISTENCE_DUTY);
    
    /* Test direct mode */
    ret = rk3588_vr_display_set_mode(vrd, RK3588_VR_MODE_DIRECT);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.mode, RK3588_VR_MODE_DIRECT);
    
    direct_mode = *(u32*)(regs + RK3588_VOP_VR_DIRECT_MODE);
    EXPECT_TRUE(direct_mode & RK3588_VOP_VR_DIRECT_MODE_EN);
    EXPECT_TRUE(direct_mode & RK3588_VOP_VR_DIRECT_MODE_FAST_PATH);
    
    /* Test async mode */
    ret = rk3588_vr_display_set_mode(vrd, RK3588_VR_MODE_ASYNC);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.mode, RK3588_VR_MODE_ASYNC);
    
    async_commit = *(u32*)(regs + RK3588_VOP_VR_ASYNC_COMMIT);
    EXPECT_TRUE(async_commit & RK3588_VOP_VR_ASYNC_COMMIT_EN);
    
    /* Test invalid mode */
    ret = rk3588_vr_display_set_mode(vrd, (enum rk3588_vr_display_mode)10);
    EXPECT_EQ(ret, -EINVAL);
    
    /* Finalize */
    rk3588_vr_display_fini(vrd);
}

/* Test sync mode setting */
TEST_F(RK3588VRDisplayUnitTest, SyncModeTest) {
    /* Initialize */
    int ret = rk3588_vr_display_init(vrd);
    EXPECT_EQ(ret, 0);
    
    /* Test independent mode */
    ret = rk3588_vr_display_set_sync_mode(vrd, RK3588_VR_SYNC_INDEPENDENT);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.sync_mode, RK3588_VR_SYNC_INDEPENDENT);
    
    u32 sync_ctrl = *(u32*)(regs + RK3588_VOP_VR_SYNC_CTRL);
    EXPECT_TRUE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_EN);
    EXPECT_FALSE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_MASTER);
    EXPECT_FALSE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_SLAVE);
    
    /* Test master mode */
    ret = rk3588_vr_display_set_sync_mode(vrd, RK3588_VR_SYNC_MASTER);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.sync_mode, RK3588_VR_SYNC_MASTER);
    
    sync_ctrl = *(u32*)(regs + RK3588_VOP_VR_SYNC_CTRL);
    EXPECT_TRUE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_EN);
    EXPECT_TRUE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_MASTER);
    EXPECT_FALSE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_SLAVE);
    
    /* Test slave mode */
    ret = rk3588_vr_display_set_sync_mode(vrd, RK3588_VR_SYNC_SLAVE);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.sync_mode, RK3588_VR_SYNC_SLAVE);
    
    sync_ctrl = *(u32*)(regs + RK3588_VOP_VR_SYNC_CTRL);
    EXPECT_TRUE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_EN);
    EXPECT_FALSE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_MASTER);
    EXPECT_TRUE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_SLAVE);
    
    /* Test external mode */
    ret = rk3588_vr_display_set_sync_mode(vrd, RK3588_VR_SYNC_EXTERNAL);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.sync_mode, RK3588_VR_SYNC_EXTERNAL);
    
    sync_ctrl = *(u32*)(regs + RK3588_VOP_VR_SYNC_CTRL);
    EXPECT_TRUE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_EN);
    EXPECT_TRUE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_MASTER);
    EXPECT_TRUE(sync_ctrl & RK3588_VOP_VR_SYNC_CTRL_SLAVE);
    
    /* Test invalid mode */
    ret = rk3588_vr_display_set_sync_mode(vrd, (enum rk3588_vr_sync_mode)10);
    EXPECT_EQ(ret, -EINVAL);
    
    /* Finalize */
    rk3588_vr_display_fini(vrd);
}

/* Test distortion mode setting */
TEST_F(RK3588VRDisplayUnitTest, DistortionModeTest) {
    /* Initialize */
    int ret = rk3588_vr_display_init(vrd);
    EXPECT_EQ(ret, 0);
    
    /* Test none mode */
    ret = rk3588_vr_display_set_distortion_mode(vrd, RK3588_VR_DISTORTION_NONE);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.distortion_mode, RK3588_VR_DISTORTION_NONE);
    
    u32 distortion_ctrl = *(u32*)(regs + RK3588_VOP_VR_DISTORTION_CTRL);
    EXPECT_EQ(distortion_ctrl, 0);
    
    /* Test barrel mode */
    ret = rk3588_vr_display_set_distortion_mode(vrd, RK3588_VR_DISTORTION_BARREL);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.distortion_mode, RK3588_VR_DISTORTION_BARREL);
    
    distortion_ctrl = *(u32*)(regs + RK3588_VOP_VR_DISTORTION_CTRL);
    EXPECT_TRUE(distortion_ctrl & RK3588_VOP_VR_DISTORTION_CTRL_EN);
    EXPECT_TRUE(distortion_ctrl & RK3588_VOP_VR_DISTORTION_CTRL_BARREL);
    
    /* Test pincushion mode */
    ret = rk3588_vr_display_set_distortion_mode(vrd, RK3588_VR_DISTORTION_PINCUSHION);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.distortion_mode, RK3588_VR_DISTORTION_PINCUSHION);
    
    distortion_ctrl = *(u32*)(regs + RK3588_VOP_VR_DISTORTION_CTRL);
    EXPECT_TRUE(distortion_ctrl & RK3588_VOP_VR_DISTORTION_CTRL_EN);
    EXPECT_TRUE(distortion_ctrl & RK3588_VOP_VR_DISTORTION_CTRL_PINCUSHION);
    
    /* Test mesh mode */
    ret = rk3588_vr_display_set_distortion_mode(vrd, RK3588_VR_DISTORTION_MESH);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.distortion_mode, RK3588_VR_DISTORTION_MESH);
    
    distortion_ctrl = *(u32*)(regs + RK3588_VOP_VR_DISTORTION_CTRL);
    EXPECT_TRUE(distortion_ctrl & RK3588_VOP_VR_DISTORTION_CTRL_EN);
    EXPECT_TRUE(distortion_ctrl & RK3588_VOP_VR_DISTORTION_CTRL_MESH);
    
    /* Test custom mode */
    ret = rk3588_vr_display_set_distortion_mode(vrd, RK3588_VR_DISTORTION_CUSTOM);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.distortion_mode, RK3588_VR_DISTORTION_CUSTOM);
    
    distortion_ctrl = *(u32*)(regs + RK3588_VOP_VR_DISTORTION_CTRL);
    EXPECT_TRUE(distortion_ctrl & RK3588_VOP_VR_DISTORTION_CTRL_EN);
    EXPECT_TRUE(distortion_ctrl & RK3588_VOP_VR_DISTORTION_CTRL_CUSTOM);
    
    /* Test invalid mode */
    ret = rk3588_vr_display_set_distortion_mode(vrd, (enum rk3588_vr_distortion_mode)10);
    EXPECT_EQ(ret, -EINVAL);
    
    /* Finalize */
    rk3588_vr_display_fini(vrd);
}

/* Test chromatic mode setting */
TEST_F(RK3588VRDisplayUnitTest, ChromaticModeTest) {
    /* Initialize */
    int ret = rk3588_vr_display_init(vrd);
    EXPECT_EQ(ret, 0);
    
    /* Test none mode */
    ret = rk3588_vr_display_set_chromatic_mode(vrd, RK3588_VR_CHROMATIC_NONE);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.chromatic_mode, RK3588_VR_CHROMATIC_NONE);
    
    u32 chromatic_ctrl = *(u32*)(regs + RK3588_VOP_VR_CHROMATIC_CTRL);
    EXPECT_EQ(chromatic_ctrl, 0);
    
    /* Test RGB mode */
    ret = rk3588_vr_display_set_chromatic_mode(vrd, RK3588_VR_CHROMATIC_RGB);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.chromatic_mode, RK3588_VR_CHROMATIC_RGB);
    
    chromatic_ctrl = *(u32*)(regs + RK3588_VOP_VR_CHROMATIC_CTRL);
    EXPECT_TRUE(chromatic_ctrl & RK3588_VOP_VR_CHROMATIC_CTRL_EN);
    EXPECT_TRUE(chromatic_ctrl & RK3588_VOP_VR_CHROMATIC_CTRL_RGB);
    
    /* Test custom mode */
    ret = rk3588_vr_display_set_chromatic_mode(vrd, RK3588_VR_CHROMATIC_CUSTOM);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.chromatic_mode, RK3588_VR_CHROMATIC_CUSTOM);
    
    chromatic_ctrl = *(u32*)(regs + RK3588_VOP_VR_CHROMATIC_CTRL);
    EXPECT_TRUE(chromatic_ctrl & RK3588_VOP_VR_CHROMATIC_CTRL_EN);
    EXPECT_TRUE(chromatic_ctrl & RK3588_VOP_VR_CHROMATIC_CTRL_CUSTOM);
    
    /* Test invalid mode */
    ret = rk3588_vr_display_set_chromatic_mode(vrd, (enum rk3588_vr_chromatic_mode)10);
    EXPECT_EQ(ret, -EINVAL);
    
    /* Finalize */
    rk3588_vr_display_fini(vrd);
}

/* Test motion compensation mode setting */
TEST_F(RK3588VRDisplayUnitTest, MotionCompModeTest) {
    /* Initialize */
    int ret = rk3588_vr_display_init(vrd);
    EXPECT_EQ(ret, 0);
    
    /* Test none mode */
    ret = rk3588_vr_display_set_motion_comp_mode(vrd, RK3588_VR_MOTION_COMP_NONE);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.motion_comp_mode, RK3588_VR_MOTION_COMP_NONE);
    
    u32 motion_comp = *(u32*)(regs + RK3588_VOP_VR_MOTION_COMP);
    EXPECT_EQ(motion_comp, 0);
    
    /* Test predict mode */
    ret = rk3588_vr_display_set_motion_comp_mode(vrd, RK3588_VR_MOTION_COMP_PREDICT);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.motion_comp_mode, RK3588_VR_MOTION_COMP_PREDICT);
    
    motion_comp = *(u32*)(regs + RK3588_VOP_VR_MOTION_COMP);
    EXPECT_TRUE(motion_comp & RK3588_VOP_VR_MOTION_COMP_EN);
    EXPECT_TRUE(motion_comp & RK3588_VOP_VR_MOTION_COMP_PREDICT);
    
    /* Test extrapolate mode */
    ret = rk3588_vr_display_set_motion_comp_mode(vrd, RK3588_VR_MOTION_COMP_EXTRAPOLATE);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(vrd->config.motion_comp_mode, RK3588_VR_MOTION_COMP_EXTRAPOLATE);
    
    motion_comp = *(u32*)(regs + RK3588_VOP_VR_MOTION_COMP);
    EXPECT_TRUE(motion_comp & RK3588_VOP_VR_MOTION_COMP_EN);
    EXPECT_TRUE(motion_comp & RK3588_VOP_VR_MOTION_COMP_EXTRAPOLATE);
    
    /* Test invalid mode */
    ret = rk3588_vr_display_set_motion_comp_mode(vrd, (enum rk3588_vr_motion_comp_mode)10);
    EXPECT_EQ(ret, -EINVAL);
    
    /* Finalize */
    rk3588_vr_display_fini(vrd);
}

/* Test low persistence setting */
TEST_F(RK3588VRDisplayUnitTest, LowPersistenceTest) {
    /* Initialize */
    int ret = rk3588_vr_display_init(vrd);
    EXPECT_EQ(ret, 0);
    
    /* Test disable */
    ret = rk3588_vr_display_set_low_persistence(vrd, false, 0);
    EXPECT_EQ(ret, 0);
    EXPECT_FALSE(vrd->config.low_persistence);
    
    u32 low_persist = *(u32*)(regs + RK3588_VOP_VR_LOW_PERSIST);
    EXPECT_EQ(low_persist, 0);
    
    /* Test enable with 50% duty cycle */
    ret = rk3588_vr_display_set_low_persistence(vrd, true, 50);
    EXPECT_EQ(ret, 0);
    EXPECT_TRUE(vrd->config.low_persistence);
    EXPECT_EQ(vrd->config.low_persistence_duty, 50);
    
    low_persist = *(u32*)(regs + RK3588_VOP_VR_LOW_PERSIST);
    EXPECT_TRUE(low_persist & RK3588_VOP_VR_LOW_PERSIST_EN);
    EXPECT_EQ((low_persist >> 8) & 0xFF, 50);
    
    /* Test enable with 100% duty cycle (should be clamped) */
    ret = rk3588_vr_display_set_low_persistence(vrd, true, 150);
    EXPECT_EQ(ret, 0);
    EXPECT_TRUE(vrd->config.low_persistence);
    EXPECT_EQ(vrd->config.low_persistence_duty, 100);
    
    low_persist = *(u32*)(regs + RK3588_VOP_VR_LOW_PERSIST);
    EXPECT_TRUE(low_persist & RK3588_VOP_VR_LOW_PERSIST_EN);
    EXPECT_EQ((low_persist >> 8) & 0xFF, 100);
    
    /* Finalize */
    rk3588_vr_display_fini(vrd);
}

/* Test fast path setting */
TEST_F(RK3588VRDisplayUnitTest, FastPathTest) {
    /* Initialize */
    int ret = rk3588_vr_display_init(vrd);
    EXPECT_EQ(ret, 0);
    
    /* Test disable */
    ret = rk3588_vr_display_set_fast_path(vrd, false);
    EXPECT_EQ(ret, 0);
    EXPECT_FALSE(vrd->config.fast_path);
    
    u32 latency_ctrl = *(u32*)(regs + RK3588_VOP_VR_LATENCY_CTRL);
    EXPECT_FALSE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_FAST_PATH);
    
    /* Test enable */
    ret = rk3588_vr_display_set_fast_path(vrd, true);
    EXPECT_EQ(ret, 0);
    EXPECT_TRUE(vrd->config.fast_path);
    
    latency_ctrl = *(u32*)(regs + RK3588_VOP_VR_LATENCY_CTRL);
    EXPECT_TRUE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_FAST_PATH);
    
    /* Test with direct mode */
    ret = rk3588_vr_display_set_mode(vrd, RK3588_VR_MODE_DIRECT);
    EXPECT_EQ(ret, 0);
    
    u32 direct_mode = *(u32*)(regs + RK3588_VOP_VR_DIRECT_MODE);
    EXPECT_TRUE(direct_mode & RK3588_VOP_VR_DIRECT_MODE_FAST_PATH);
    
    /* Disable fast path with direct mode */
    ret = rk3588_vr_display_set_fast_path(vrd, false);
    EXPECT_EQ(ret, 0);
    
    direct_mode = *(u32*)(regs + RK3588_VOP_VR_DIRECT_MODE);
    EXPECT_FALSE(direct_mode & RK3588_VOP_VR_DIRECT_MODE_FAST_PATH);
    
    /* Finalize */
    rk3588_vr_display_fini(vrd);
}

/* Test bypass options setting */
TEST_F(RK3588VRDisplayUnitTest, BypassOptionsTest) {
    /* Initialize */
    int ret = rk3588_vr_display_init(vrd);
    EXPECT_EQ(ret, 0);
    
    /* Test all bypasses disabled */
    ret = rk3588_vr_display_set_bypass_options(vrd, false, false, false, false);
    EXPECT_EQ(ret, 0);
    EXPECT_FALSE(vrd->config.bypass_blend);
    EXPECT_FALSE(vrd->config.bypass_scale);
    EXPECT_FALSE(vrd->config.bypass_gamma);
    EXPECT_FALSE(vrd->config.bypass_dither);
    
    u32 latency_ctrl = *(u32*)(regs + RK3588_VOP_VR_LATENCY_CTRL);
    EXPECT_FALSE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_BYPASS_BLEND);
    EXPECT_FALSE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_BYPASS_SCALE);
    EXPECT_FALSE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_BYPASS_GAMMA);
    EXPECT_FALSE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_BYPASS_DITHER);
    
    /* Test all bypasses enabled */
    ret = rk3588_vr_display_set_bypass_options(vrd, true, true, true, true);
    EXPECT_EQ(ret, 0);
    EXPECT_TRUE(vrd->config.bypass_blend);
    EXPECT_TRUE(vrd->config.bypass_scale);
    EXPECT_TRUE(vrd->config.bypass_gamma);
    EXPECT_TRUE(vrd->config.bypass_dither);
    
    latency_ctrl = *(u32*)(regs + RK3588_VOP_VR_LATENCY_CTRL);
    EXPECT_TRUE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_BYPASS_BLEND);
    EXPECT_TRUE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_BYPASS_SCALE);
    EXPECT_TRUE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_BYPASS_GAMMA);
    EXPECT_TRUE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_BYPASS_DITHER);
    
    /* Test mixed bypasses */
    ret = rk3588_vr_display_set_bypass_options(vrd, true, false, true, false);
    EXPECT_EQ(ret, 0);
    EXPECT_TRUE(vrd->config.bypass_blend);
    EXPECT_FALSE(vrd->config.bypass_scale);
    EXPECT_TRUE(vrd->config.bypass_gamma);
    EXPECT_FALSE(vrd->config.bypass_dither);
    
    latency_ctrl = *(u32*)(regs + RK3588_VOP_VR_LATENCY_CTRL);
    EXPECT_TRUE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_BYPASS_BLEND);
    EXPECT_FALSE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_BYPASS_SCALE);
    EXPECT_TRUE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_BYPASS_GAMMA);
    EXPECT_FALSE(latency_ctrl & RK3588_VOP_VR_LATENCY_CTRL_BYPASS_DITHER);
    
    /* Finalize */
    rk3588_vr_display_fini(vrd);
}

/* Test enable/disable */
TEST_F(RK3588VRDisplayUnitTest, EnableDisableTest) {
    /* Initialize */
    int ret = rk3588_vr_display_init(vrd);
    EXPECT_EQ(ret, 0);
    
    /* Test disable */
    rk3588_vr_display_disable(vrd);
    EXPECT_FALSE(vrd->enabled);
    
    u32 sys_ctrl = *(u32*)(regs + RK3588_VOP_SYS_CTRL);
    EXPECT_FALSE(sys_ctrl & RK3588_VOP_SYS_CTRL_EN);
    
    /* Test enable */
    ret = rk3588_vr_display_enable(vrd);
    EXPECT_EQ(ret, 0);
    EXPECT_TRUE(vrd->enabled);
    
    sys_ctrl = *(u32*)(regs + RK3588_VOP_SYS_CTRL);
    EXPECT_TRUE(sys_ctrl & RK3588_VOP_SYS_CTRL_EN);
    
    /* Finalize */
    rk3588_vr_display_fini(vrd);
}

/* Test suspend/resume */
TEST_F(RK3588VRDisplayUnitTest, SuspendResumeTest) {
    /* Initialize */
    int ret = rk3588_vr_display_init(vrd);
    EXPECT_EQ(ret, 0);
    
    /* Test suspend */
    ret = rk3588_vr_display_suspend(vrd);
    EXPECT_EQ(ret, 0);
    EXPECT_TRUE(vrd->suspended);
    EXPECT_FALSE(vrd->enabled);
    
    u32 sys_ctrl = *(u32*)(regs + RK3588_VOP_SYS_CTRL);
    EXPECT_FALSE(sys_ctrl & RK3588_VOP_SYS_CTRL_EN);
    
    /* Test resume */
    ret = rk3588_vr_display_resume(vrd);
    EXPECT_EQ(ret, 0);
    EXPECT_FALSE(vrd->suspended);
    EXPECT_TRUE(vrd->enabled);
    
    sys_ctrl = *(u32*)(regs + RK3588_VOP_SYS_CTRL);
    EXPECT_TRUE(sys_ctrl & RK3588_VOP_SYS_CTRL_EN);
    
    /* Finalize */
    rk3588_vr_display_fini(vrd);
}

/* Main function */
int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
