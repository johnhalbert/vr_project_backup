/* SPDX-License-Identifier: GPL-2.0 */
/*
 * OV9281 Camera Driver
 *
 * Copyright (C) 2025 VR Headset Project
 */

#ifndef _OV9281_CORE_H_
#define _OV9281_CORE_H_

#include <linux/types.h>
#include <linux/v4l2-mediabus.h>
#include <media/v4l2-device.h>
#include <media/v4l2-subdev.h>
#include <media/v4l2-ctrls.h>

/* OV9281 Register Map */
#define OV9281_REG_CHIP_ID_HIGH        0x300A
#define OV9281_REG_CHIP_ID_LOW         0x300B
#define OV9281_REG_SC_CMMN_CHIP_ID     0x0000
#define OV9281_REG_SC_CMMN_SUB_ID      0x0001
#define OV9281_REG_STREAM_CTRL         0x0100
#define OV9281_REG_MODE_SELECT         0x0103
#define OV9281_REG_SC_CMMN_PAD_OEN0    0x3000
#define OV9281_REG_SC_CMMN_PAD_OEN1    0x3001
#define OV9281_REG_SC_CMMN_PAD_OEN2    0x3002
#define OV9281_REG_SC_CMMN_PAD_OUT0    0x3008
#define OV9281_REG_SC_CMMN_PAD_OUT1    0x3009
#define OV9281_REG_SC_CMMN_PAD_OUT2    0x300A
#define OV9281_REG_SC_CMMN_PAD_SEL0    0x300E
#define OV9281_REG_SC_CMMN_PAD_SEL1    0x300F
#define OV9281_REG_SC_CMMN_PAD_SEL2    0x3010
#define OV9281_REG_AEC_EXPO_H          0x3500
#define OV9281_REG_AEC_EXPO_M          0x3501
#define OV9281_REG_AEC_EXPO_L          0x3502
#define OV9281_REG_AEC_MANUAL          0x3503
#define OV9281_REG_AEC_AGC_ADJ_H       0x3508
#define OV9281_REG_AEC_AGC_ADJ_L       0x3509
#define OV9281_REG_TIMING_CONTROL      0x3800
#define OV9281_REG_TIMING_HTS_H        0x380C
#define OV9281_REG_TIMING_HTS_L        0x380D
#define OV9281_REG_TIMING_VTS_H        0x380E
#define OV9281_REG_TIMING_VTS_L        0x380F
#define OV9281_REG_TIMING_X_INC        0x3814
#define OV9281_REG_TIMING_Y_INC        0x3815
#define OV9281_REG_HVOFFS_H            0x3816
#define OV9281_REG_HVOFFS_L            0x3817
#define OV9281_REG_VFLIP               0x3820
#define OV9281_REG_HFLIP               0x3821
#define OV9281_REG_FORMAT1             0x3F00
#define OV9281_REG_FORMAT2             0x3F04
#define OV9281_REG_FORMAT_CTRL         0x3F05
#define OV9281_REG_SYNC_MODE           0x3F0A
#define OV9281_REG_FRAME_CTRL          0x4202
#define OV9281_REG_MIPI_CTRL_00        0x4800
#define OV9281_REG_MIPI_CTRL_01        0x4801
#define OV9281_REG_MIPI_CTRL_05        0x4805
#define OV9281_REG_CLK_CTRL            0x4837
#define OV9281_REG_ISP_CTRL            0x5000
#define OV9281_REG_ISP_CTRL2           0x5001
#define OV9281_REG_EXPOSURE_CTRL       0x5002

/* OV9281 Constants */
#define OV9281_CHIP_ID                 0x9281
#define OV9281_RESET_DELAY_MS          20
#define OV9281_MAX_WIDTH               1280
#define OV9281_MAX_HEIGHT              800
#define OV9281_MIN_WIDTH               320
#define OV9281_MIN_HEIGHT              200
#define OV9281_DEFAULT_WIDTH           1280
#define OV9281_DEFAULT_HEIGHT          800
#define OV9281_PIXEL_ARRAY_WIDTH       1296
#define OV9281_PIXEL_ARRAY_HEIGHT      816
#define OV9281_PIXEL_RATE              74250000
#define OV9281_XVCLK_FREQ              24000000
#define OV9281_DEFAULT_LINK_FREQ       400000000
#define OV9281_DEFAULT_MBUS_CODE       MEDIA_BUS_FMT_Y10_1X10
#define OV9281_DEFAULT_FRAMERATE       60
#define OV9281_MAX_FRAMERATE           180
#define OV9281_DEFAULT_EXPOSURE        500
#define OV9281_DEFAULT_GAIN            1000
#define OV9281_DEFAULT_TEST_PATTERN    0

/* OV9281 Register Values */
#define OV9281_MODE_SW_STANDBY         0x0
#define OV9281_MODE_STREAMING          0x1
#define OV9281_RESET_VALUE             0x1
#define OV9281_FLIP_ENABLE             0x3
#define OV9281_FLIP_DISABLE            0x0
#define OV9281_EXPOSURE_MANUAL         0x1
#define OV9281_EXPOSURE_AUTO           0x0
#define OV9281_SYNC_MASTER             0x0
#define OV9281_SYNC_SLAVE              0x1
#define OV9281_SYNC_EXTERNAL           0x2

/* OV9281 Exposure/Gain Limits */
#define OV9281_EXPOSURE_MIN            1
#define OV9281_EXPOSURE_MAX            65535
#define OV9281_EXPOSURE_STEP           1
#define OV9281_EXPOSURE_DEFAULT        1000
#define OV9281_GAIN_MIN                0
#define OV9281_GAIN_MAX                4095
#define OV9281_GAIN_STEP               1
#define OV9281_GAIN_DEFAULT            1024

/* OV9281 Sync Mode */
enum ov9281_sync_mode {
    OV9281_SYNC_MODE_MASTER = 0,
    OV9281_SYNC_MODE_SLAVE,
    OV9281_SYNC_MODE_EXTERNAL,
};

/* OV9281 Frame Rate Mode */
enum ov9281_frame_rate {
    OV9281_30_FPS = 0,
    OV9281_60_FPS,
    OV9281_90_FPS,
    OV9281_120_FPS,
    OV9281_150_FPS,
    OV9281_180_FPS,
};

/* OV9281 Test Pattern */
enum ov9281_test_pattern {
    OV9281_TEST_PATTERN_DISABLED = 0,
    OV9281_TEST_PATTERN_SOLID_COLOR,
    OV9281_TEST_PATTERN_COLOR_BARS,
    OV9281_TEST_PATTERN_GRADIENT_H,
    OV9281_TEST_PATTERN_GRADIENT_V,
};

/* OV9281 Device State */
enum ov9281_state {
    OV9281_STATE_DISABLED = 0,
    OV9281_STATE_INITIALIZING,
    OV9281_STATE_INITIALIZED,
    OV9281_STATE_STREAMING,
    OV9281_STATE_ERROR,
};

/* OV9281 Device Structure */
struct ov9281_device {
    struct v4l2_subdev sd;
    struct media_pad pad;
    struct v4l2_ctrl_handler ctrl_handler;
    struct v4l2_ctrl *exposure;
    struct v4l2_ctrl *gain;
    struct v4l2_ctrl *hflip;
    struct v4l2_ctrl *vflip;
    struct v4l2_ctrl *test_pattern;
    struct v4l2_ctrl *pixel_rate;
    struct v4l2_ctrl *link_freq;
    struct mutex lock;
    
    /* Device state */
    enum ov9281_state state;
    enum ov9281_sync_mode sync_mode;
    enum ov9281_frame_rate frame_rate;
    
    /* Format */
    struct v4l2_mbus_framefmt fmt;
    
    /* Timing */
    u32 hts;
    u32 vts;
    
    /* GPIO */
    int reset_gpio;
    int pwdn_gpio;
    int sync_gpio;
    
    /* Clock */
    struct clk *xvclk;
    u32 xvclk_freq;
    
    /* Regulator */
    struct regulator *avdd;
    struct regulator *dovdd;
    struct regulator *dvdd;
    
    /* I2C client */
    struct i2c_client *client;
    
    /* Zero-copy buffer support */
    bool zero_copy_enabled;
    void *dma_buffer;
    dma_addr_t dma_addr;
    size_t dma_size;
    
    /* Multi-camera synchronization */
    bool is_master;
    int num_slaves;
    struct ov9281_device **slaves;
    
    /* VR-specific optimizations */
    bool vr_mode;
    bool low_latency;
    bool high_framerate;
    
    /* Debug */
    struct dentry *debugfs_root;
};

/* Core driver functions */
int ov9281_core_probe(struct i2c_client *client, const struct i2c_device_id *id);
int ov9281_core_remove(struct i2c_client *client);
int ov9281_core_init(struct ov9281_device *dev);
int ov9281_set_mode(struct ov9281_device *dev, enum ov9281_sync_mode mode);
int ov9281_set_frame_rate(struct ov9281_device *dev, enum ov9281_frame_rate rate);
int ov9281_set_test_pattern(struct ov9281_device *dev, enum ov9281_test_pattern pattern);
int ov9281_set_exposure(struct ov9281_device *dev, u32 exposure);
int ov9281_set_gain(struct ov9281_device *dev, u32 gain);
int ov9281_set_flip(struct ov9281_device *dev, bool hflip, bool vflip);
int ov9281_start_streaming(struct ov9281_device *dev);
int ov9281_stop_streaming(struct ov9281_device *dev);
int ov9281_reset(struct ov9281_device *dev);
int ov9281_enable_zero_copy(struct ov9281_device *dev, bool enable);
int ov9281_sync_sensors(struct ov9281_device *dev);

/* V4L2 subdev operations */
int ov9281_s_power(struct v4l2_subdev *sd, int on);
int ov9281_g_frame_interval(struct v4l2_subdev *sd, struct v4l2_subdev_frame_interval *fi);
int ov9281_s_frame_interval(struct v4l2_subdev *sd, struct v4l2_subdev_frame_interval *fi);
int ov9281_enum_mbus_code(struct v4l2_subdev *sd, struct v4l2_subdev_pad_config *cfg, struct v4l2_subdev_mbus_code_enum *code);
int ov9281_enum_frame_size(struct v4l2_subdev *sd, struct v4l2_subdev_pad_config *cfg, struct v4l2_subdev_frame_size_enum *fse);
int ov9281_get_fmt(struct v4l2_subdev *sd, struct v4l2_subdev_pad_config *cfg, struct v4l2_subdev_format *format);
int ov9281_set_fmt(struct v4l2_subdev *sd, struct v4l2_subdev_pad_config *cfg, struct v4l2_subdev_format *format);
int ov9281_s_stream(struct v4l2_subdev *sd, int enable);

/* V4L2 control operations */
int ov9281_s_ctrl(struct v4l2_ctrl *ctrl);

/* External declarations */
extern const struct v4l2_subdev_core_ops ov9281_core_ops;
extern const struct v4l2_subdev_video_ops ov9281_video_ops;
extern const struct v4l2_subdev_pad_ops ov9281_pad_ops;
extern const struct v4l2_ctrl_ops ov9281_ctrl_ops;
extern const struct v4l2_subdev_ops ov9281_subdev_ops;
extern const struct v4l2_ctrl_config ov9281_ctrl_sync_mode;
extern const struct v4l2_ctrl_config ov9281_ctrl_frame_rate;
extern const struct v4l2_ctrl_config ov9281_ctrl_vr_mode;
extern const struct v4l2_ctrl_config ov9281_ctrl_low_latency;

#endif /* _OV9281_CORE_H_ */
