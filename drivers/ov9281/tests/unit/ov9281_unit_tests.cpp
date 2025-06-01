#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <vector>
#include <random>
#include <chrono>

/* Mock includes to replace kernel headers */
#include "mock_kernel.h"
#include "mock_v4l2.h"

/* Include driver header with special define to handle kernel dependencies */
#define UNIT_TESTING
#include "../ov9281_core.h"

using ::testing::_;
using ::testing::Return;
using ::testing::Invoke;

class OV9281UnitTest : public ::testing::Test {
protected:
    void SetUp() override {
        /* Initialize mock device */
        client = mock_i2c_client_create();
        dev = (struct ov9281_device*)malloc(sizeof(struct ov9281_device));
        memset(dev, 0, sizeof(struct ov9281_device));
        dev->client = client;
        mutex_init(&dev->lock);
        
        /* Initialize mock registers */
        mock_registers[OV9281_REG_CHIP_ID_HIGH] = 0x92;
        mock_registers[OV9281_REG_CHIP_ID_LOW] = 0x81;
        
        /* Initialize random number generator */
        rng.seed(std::chrono::system_clock::now().time_since_epoch().count());
    }
    
    void TearDown() override {
        mutex_destroy(&dev->lock);
        free(dev);
        mock_i2c_client_destroy(client);
    }
    
    /* Mock I2C read function */
    static int mock_i2c_read(struct i2c_client *client, u16 reg, u8 *val) {
        *val = mock_registers[reg];
        return 0;
    }
    
    /* Mock I2C write function */
    static int mock_i2c_write(struct i2c_client *client, u16 reg, u8 val) {
        mock_registers[reg] = val;
        return 0;
    }
    
    struct i2c_client *client;
    struct ov9281_device *dev;
    
    std::mt19937 rng;
    std::normal_distribution<float> normal_dist{0.0, 1.0};
    
    static u8 mock_registers[0x10000];
};

/* Initialize static members */
u8 OV9281UnitTest::mock_registers[0x10000] = {0};

/* Test device initialization */
TEST_F(OV9281UnitTest, InitializationTest) {
    /* Set up mock I2C functions */
    EXPECT_CALL(*(MockI2C*)client->dev.driver_data, i2c_transfer(_, _, _))
        .WillRepeatedly(Invoke([this](struct i2c_client *client, struct i2c_msg *msgs, int num) {
            if (num == 2 && msgs[0].flags == 0 && msgs[1].flags == I2C_M_RD) {
                /* Read operation */
                u16 reg = (msgs[0].buf[0] << 8) | msgs[0].buf[1];
                msgs[1].buf[0] = mock_registers[reg];
                return 2;
            } else if (num == 1 && msgs[0].flags == 0) {
                /* Write operation */
                u16 reg = (msgs[0].buf[0] << 8) | msgs[0].buf[1];
                mock_registers[reg] = msgs[0].buf[2];
                return 1;
            }
            return -EIO;
        }));
    
    /* Call initialization function */
    int ret = ov9281_core_init(dev);
    
    /* Verify initialization was successful */
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev->state, OV9281_STATE_INITIALIZED);
    
    /* Verify default settings */
    EXPECT_EQ(dev->sync_mode, OV9281_SYNC_MODE_MASTER);
    EXPECT_EQ(dev->frame_rate, OV9281_60_FPS);
    EXPECT_TRUE(dev->is_master);
    
    /* Verify registers were written */
    EXPECT_EQ(mock_registers[OV9281_REG_STREAM_CTRL], OV9281_MODE_SW_STANDBY);
    EXPECT_EQ(mock_registers[OV9281_REG_SYNC_MODE], 0x00); /* Master mode */
}

/* Test frame rate setting */
TEST_F(OV9281UnitTest, FrameRateTest) {
    /* Set up mock I2C functions */
    EXPECT_CALL(*(MockI2C*)client->dev.driver_data, i2c_transfer(_, _, _))
        .WillRepeatedly(Invoke([this](struct i2c_client *client, struct i2c_msg *msgs, int num) {
            if (num == 2 && msgs[0].flags == 0 && msgs[1].flags == I2C_M_RD) {
                /* Read operation */
                u16 reg = (msgs[0].buf[0] << 8) | msgs[0].buf[1];
                msgs[1].buf[0] = mock_registers[reg];
                return 2;
            } else if (num == 1 && msgs[0].flags == 0) {
                /* Write operation */
                u16 reg = (msgs[0].buf[0] << 8) | msgs[0].buf[1];
                mock_registers[reg] = msgs[0].buf[2];
                return 1;
            }
            return -EIO;
        }));
    
    /* Test setting different frame rates */
    int ret;
    
    /* 30 FPS */
    ret = ov9281_set_frame_rate(dev, OV9281_30_FPS);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev->frame_rate, OV9281_30_FPS);
    EXPECT_EQ(dev->hts, 0x0A00);
    EXPECT_EQ(dev->vts, 0x0465);
    EXPECT_FALSE(dev->high_framerate);
    
    /* 60 FPS */
    ret = ov9281_set_frame_rate(dev, OV9281_60_FPS);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev->frame_rate, OV9281_60_FPS);
    EXPECT_EQ(dev->hts, 0x0500);
    EXPECT_EQ(dev->vts, 0x0465);
    EXPECT_FALSE(dev->high_framerate);
    
    /* 120 FPS */
    ret = ov9281_set_frame_rate(dev, OV9281_120_FPS);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev->frame_rate, OV9281_120_FPS);
    EXPECT_EQ(dev->hts, 0x0280);
    EXPECT_EQ(dev->vts, 0x0465);
    EXPECT_TRUE(dev->high_framerate);
    
    /* 180 FPS */
    ret = ov9281_set_frame_rate(dev, OV9281_180_FPS);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev->frame_rate, OV9281_180_FPS);
    EXPECT_EQ(dev->hts, 0x01AA);
    EXPECT_EQ(dev->vts, 0x0465);
    EXPECT_TRUE(dev->high_framerate);
    
    /* Invalid frame rate */
    ret = ov9281_set_frame_rate(dev, (enum ov9281_frame_rate)10);
    EXPECT_EQ(ret, -EINVAL);
}

/* Test sync mode setting */
TEST_F(OV9281UnitTest, SyncModeTest) {
    /* Set up mock I2C functions */
    EXPECT_CALL(*(MockI2C*)client->dev.driver_data, i2c_transfer(_, _, _))
        .WillRepeatedly(Invoke([this](struct i2c_client *client, struct i2c_msg *msgs, int num) {
            if (num == 2 && msgs[0].flags == 0 && msgs[1].flags == I2C_M_RD) {
                /* Read operation */
                u16 reg = (msgs[0].buf[0] << 8) | msgs[0].buf[1];
                msgs[1].buf[0] = mock_registers[reg];
                return 2;
            } else if (num == 1 && msgs[0].flags == 0) {
                /* Write operation */
                u16 reg = (msgs[0].buf[0] << 8) | msgs[0].buf[1];
                mock_registers[reg] = msgs[0].buf[2];
                return 1;
            }
            return -EIO;
        }));
    
    /* Test setting different sync modes */
    int ret;
    
    /* Master mode */
    ret = ov9281_set_mode(dev, OV9281_SYNC_MODE_MASTER);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev->sync_mode, OV9281_SYNC_MODE_MASTER);
    EXPECT_TRUE(dev->is_master);
    EXPECT_EQ(mock_registers[OV9281_REG_SYNC_MODE], 0x00);
    
    /* Slave mode */
    ret = ov9281_set_mode(dev, OV9281_SYNC_MODE_SLAVE);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev->sync_mode, OV9281_SYNC_MODE_SLAVE);
    EXPECT_FALSE(dev->is_master);
    EXPECT_EQ(mock_registers[OV9281_REG_SYNC_MODE], 0x01);
    
    /* External mode */
    ret = ov9281_set_mode(dev, OV9281_SYNC_MODE_EXTERNAL);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev->sync_mode, OV9281_SYNC_MODE_EXTERNAL);
    EXPECT_FALSE(dev->is_master);
    EXPECT_EQ(mock_registers[OV9281_REG_SYNC_MODE], 0x02);
    
    /* Invalid mode */
    ret = ov9281_set_mode(dev, (enum ov9281_sync_mode)10);
    EXPECT_EQ(ret, -EINVAL);
}

/* Test exposure and gain setting */
TEST_F(OV9281UnitTest, ExposureGainTest) {
    /* Set up mock I2C functions */
    EXPECT_CALL(*(MockI2C*)client->dev.driver_data, i2c_transfer(_, _, _))
        .WillRepeatedly(Invoke([this](struct i2c_client *client, struct i2c_msg *msgs, int num) {
            if (num == 2 && msgs[0].flags == 0 && msgs[1].flags == I2C_M_RD) {
                /* Read operation */
                u16 reg = (msgs[0].buf[0] << 8) | msgs[0].buf[1];
                msgs[1].buf[0] = mock_registers[reg];
                return 2;
            } else if (num == 1 && msgs[0].flags == 0) {
                /* Write operation */
                u16 reg = (msgs[0].buf[0] << 8) | msgs[0].buf[1];
                mock_registers[reg] = msgs[0].buf[2];
                return 1;
            }
            return -EIO;
        }));
    
    /* Test setting exposure */
    int ret;
    
    /* Normal exposure */
    ret = ov9281_set_exposure(dev, 1000);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_EXPO_H], 0x00);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_EXPO_M], 0x03);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_EXPO_L], 0xE8);
    
    /* Min exposure */
    ret = ov9281_set_exposure(dev, 0);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_EXPO_H], 0x00);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_EXPO_M], 0x00);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_EXPO_L], 0x01); /* Clamped to min */
    
    /* Max exposure */
    ret = ov9281_set_exposure(dev, 100000);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_EXPO_H], 0x00);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_EXPO_M], 0xFF);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_EXPO_L], 0xFF); /* Clamped to max */
    
    /* Test setting gain */
    
    /* Normal gain */
    ret = ov9281_set_gain(dev, 2000);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_AGC_ADJ_H], 0x07);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_AGC_ADJ_L], 0xD0);
    
    /* Min gain */
    ret = ov9281_set_gain(dev, 0);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_AGC_ADJ_H], 0x00);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_AGC_ADJ_L], 0x00);
    
    /* Max gain */
    ret = ov9281_set_gain(dev, 5000);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_AGC_ADJ_H], 0x0F);
    EXPECT_EQ(mock_registers[OV9281_REG_AEC_AGC_ADJ_L], 0xFF); /* Clamped to max */
}

/* Test flip setting */
TEST_F(OV9281UnitTest, FlipTest) {
    /* Set up mock I2C functions */
    EXPECT_CALL(*(MockI2C*)client->dev.driver_data, i2c_transfer(_, _, _))
        .WillRepeatedly(Invoke([this](struct i2c_client *client, struct i2c_msg *msgs, int num) {
            if (num == 2 && msgs[0].flags == 0 && msgs[1].flags == I2C_M_RD) {
                /* Read operation */
                u16 reg = (msgs[0].buf[0] << 8) | msgs[0].buf[1];
                msgs[1].buf[0] = mock_registers[reg];
                return 2;
            } else if (num == 1 && msgs[0].flags == 0) {
                /* Write operation */
                u16 reg = (msgs[0].buf[0] << 8) | msgs[0].buf[1];
                mock_registers[reg] = msgs[0].buf[2];
                return 1;
            }
            return -EIO;
        }));
    
    /* Initialize registers */
    mock_registers[OV9281_REG_HFLIP] = 0x00;
    mock_registers[OV9281_REG_VFLIP] = 0x00;
    
    /* Test setting flips */
    int ret;
    
    /* No flip */
    ret = ov9281_set_flip(dev, false, false);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(mock_registers[OV9281_REG_HFLIP], 0x00);
    EXPECT_EQ(mock_registers[OV9281_REG_VFLIP], 0x00);
    
    /* H flip only */
    ret = ov9281_set_flip(dev, true, false);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(mock_registers[OV9281_REG_HFLIP], 0x03);
    EXPECT_EQ(mock_registers[OV9281_REG_VFLIP], 0x00);
    
    /* V flip only */
    ret = ov9281_set_flip(dev, false, true);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(mock_registers[OV9281_REG_HFLIP], 0x00);
    EXPECT_EQ(mock_registers[OV9281_REG_VFLIP], 0x03);
    
    /* Both flips */
    ret = ov9281_set_flip(dev, true, true);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(mock_registers[OV9281_REG_HFLIP], 0x03);
    EXPECT_EQ(mock_registers[OV9281_REG_VFLIP], 0x03);
}

/* Test streaming control */
TEST_F(OV9281UnitTest, StreamingTest) {
    /* Set up mock I2C functions */
    EXPECT_CALL(*(MockI2C*)client->dev.driver_data, i2c_transfer(_, _, _))
        .WillRepeatedly(Invoke([this](struct i2c_client *client, struct i2c_msg *msgs, int num) {
            if (num == 2 && msgs[0].flags == 0 && msgs[1].flags == I2C_M_RD) {
                /* Read operation */
                u16 reg = (msgs[0].buf[0] << 8) | msgs[0].buf[1];
                msgs[1].buf[0] = mock_registers[reg];
                return 2;
            } else if (num == 1 && msgs[0].flags == 0) {
                /* Write operation */
                u16 reg = (msgs[0].buf[0] << 8) | msgs[0].buf[1];
                mock_registers[reg] = msgs[0].buf[2];
                return 1;
            }
            return -EIO;
        }));
    
    /* Initialize state */
    dev->state = OV9281_STATE_INITIALIZED;
    
    /* Test start streaming */
    int ret = ov9281_start_streaming(dev);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev->state, OV9281_STATE_STREAMING);
    EXPECT_EQ(mock_registers[OV9281_REG_STREAM_CTRL], OV9281_MODE_STREAMING);
    
    /* Test stop streaming */
    ret = ov9281_stop_streaming(dev);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev->state, OV9281_STATE_INITIALIZED);
    EXPECT_EQ(mock_registers[OV9281_REG_STREAM_CTRL], OV9281_MODE_SW_STANDBY);
}

/* Test VR mode */
TEST_F(OV9281UnitTest, VRModeTest) {
    /* Set up mock I2C functions */
    EXPECT_CALL(*(MockI2C*)client->dev.driver_data, i2c_transfer(_, _, _))
        .WillRepeatedly(Invoke([this](struct i2c_client *client, struct i2c_msg *msgs, int num) {
            if (num == 2 && msgs[0].flags == 0 && msgs[1].flags == I2C_M_RD) {
                /* Read operation */
                u16 reg = (msgs[0].buf[0] << 8) | msgs[0].buf[1];
                msgs[1].buf[0] = mock_registers[reg];
                return 2;
            } else if (num == 1 && msgs[0].flags == 0) {
                /* Write operation */
                u16 reg = (msgs[0].buf[0] << 8) | msgs[0].buf[1];
                mock_registers[reg] = msgs[0].buf[2];
                return 1;
            }
            return -EIO;
        }));
    
    /* Initialize state */
    dev->state = OV9281_STATE_INITIALIZED;
    dev->vr_mode = true;
    
    /* Test start streaming with VR mode */
    int ret = ov9281_start_streaming(dev);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev->state, OV9281_STATE_STREAMING);
    EXPECT_EQ(mock_registers[OV9281_REG_STREAM_CTRL], OV9281_MODE_STREAMING);
    EXPECT_EQ(mock_registers[OV9281_REG_EXPOSURE_CTRL], 0x01);
    EXPECT_EQ(mock_registers[OV9281_REG_MIPI_CTRL_00], 0x24);
    EXPECT_EQ(mock_registers[OV9281_REG_MIPI_CTRL_01], 0x0F);
}

/* Main function */
int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
