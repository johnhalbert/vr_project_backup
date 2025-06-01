#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <linux/iio/iio.h>
#include <linux/iio/buffer.h>
#include <linux/iio/trigger.h>
#include <linux/regmap.h>

/* Mock includes to replace kernel headers */
#include "mock_kernel.h"
#include "mock_i2c.h"
#include "mock_spi.h"
#include "mock_iio.h"

/* Include driver header with special define to handle kernel dependencies */
#define UNIT_TESTING
#include "../bno085_core.h"

using ::testing::_;
using ::testing::Return;
using ::testing::SetArgPointee;
using ::testing::DoAll;
using ::testing::Invoke;

class BNO085Test : public ::testing::Test {
protected:
    void SetUp() override {
        /* Initialize mock device */
        dev = mock_device_create();
        
        /* Initialize mock transport */
        transport.read = mock_read;
        transport.write = mock_write;
        transport.read_fifo = mock_read_fifo;
        
        /* Initialize mock registers */
        mock_registers[BNO085_REG_CHIP_ID] = BNO085_CHIP_ID;
        mock_registers[BNO085_REG_STATUS] = BNO085_STATUS_RESET_DONE;
        
        /* Initialize mock sensor data */
        for (int i = 0; i < 3; i++) {
            mock_accel_data[i] = i + 1;
            mock_gyro_data[i] = i + 4;
            mock_mag_data[i] = i + 7;
        }
        
        for (int i = 0; i < 4; i++) {
            mock_quat_data[i] = i + 10;
        }
        
        mock_temp_data = 25;
    }
    
    void TearDown() override {
        mock_device_destroy(dev);
    }
    
    /* Mock read function */
    static int mock_read(struct device *dev, u8 reg, u8 *data, int len) {
        if (reg == BNO085_REG_ACCEL_X && len == 6) {
            memcpy(data, mock_accel_data, 6);
        } else if (reg == BNO085_REG_GYRO_X && len == 6) {
            memcpy(data, mock_gyro_data, 6);
        } else if (reg == BNO085_REG_MAG_X && len == 6) {
            memcpy(data, mock_mag_data, 6);
        } else if (reg == BNO085_REG_QUAT_W && len == 8) {
            memcpy(data, mock_quat_data, 8);
        } else if (reg == BNO085_REG_TEMP && len == 2) {
            memcpy(data, &mock_temp_data, 2);
        } else if (len == 1) {
            *data = mock_registers[reg];
        } else {
            return -EIO;
        }
        
        return 0;
    }
    
    /* Mock write function */
    static int mock_write(struct device *dev, u8 reg, const u8 *data, int len) {
        if (len == 1) {
            mock_registers[reg] = *data;
        } else {
            memcpy(&mock_registers[reg], data, len);
        }
        
        return 0;
    }
    
    /* Mock FIFO read function */
    static int mock_read_fifo(struct device *dev, u8 *data, int len) {
        /* Simulate FIFO data */
        for (int i = 0; i < len; i++) {
            data[i] = i & 0xFF;
        }
        
        return 0;
    }
    
    struct device *dev;
    struct bno085_transport transport;
    
    static u8 mock_registers[256];
    static s16 mock_accel_data[3];
    static s16 mock_gyro_data[3];
    static s16 mock_mag_data[3];
    static s16 mock_quat_data[4];
    static s16 mock_temp_data;
};

/* Initialize static members */
u8 BNO085Test::mock_registers[256] = {0};
s16 BNO085Test::mock_accel_data[3] = {0};
s16 BNO085Test::mock_gyro_data[3] = {0};
s16 BNO085Test::mock_mag_data[3] = {0};
s16 BNO085Test::mock_quat_data[4] = {0};
s16 BNO085Test::mock_temp_data = 0;

/* Test device initialization */
TEST_F(BNO085Test, InitializationTest) {
    struct bno085_device dev;
    int ret;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    
    /* Test initialization */
    ret = bno085_core_init(&dev);
    EXPECT_EQ(ret, 0);
    
    /* Verify device state */
    EXPECT_EQ(dev.state, BNO085_STATE_INITIALIZED);
    EXPECT_EQ(dev.mode, BNO085_MODE_NDOF);
    EXPECT_NE(dev.enabled_features & BNO085_FEATURE_ACCELEROMETER, 0);
    EXPECT_NE(dev.enabled_features & BNO085_FEATURE_GYROSCOPE, 0);
    EXPECT_NE(dev.enabled_features & BNO085_FEATURE_MAGNETOMETER, 0);
    EXPECT_NE(dev.enabled_features & BNO085_FEATURE_ROTATION_VECTOR, 0);
    EXPECT_EQ(dev.sampling_frequency, 100);
}

/* Test device reset */
TEST_F(BNO085Test, ResetTest) {
    struct bno085_device dev;
    int ret;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    dev.state = BNO085_STATE_RUNNING;
    dev.mode = BNO085_MODE_NDOF;
    dev.enabled_features = 0xFF;
    dev.sampling_frequency = 100;
    
    /* Test reset */
    ret = bno085_reset(&dev);
    EXPECT_EQ(ret, 0);
    
    /* Verify device state */
    EXPECT_EQ(dev.state, BNO085_STATE_INITIALIZING);
    EXPECT_EQ(dev.mode, BNO085_MODE_CONFIG);
    EXPECT_EQ(dev.enabled_features, 0);
    EXPECT_EQ(dev.sampling_frequency, 0);
}

/* Test mode setting */
TEST_F(BNO085Test, ModeTest) {
    struct bno085_device dev;
    int ret;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    
    /* Test setting different modes */
    ret = bno085_set_mode(&dev, BNO085_MODE_IMU);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev.mode, BNO085_MODE_IMU);
    
    ret = bno085_set_mode(&dev, BNO085_MODE_NDOF);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev.mode, BNO085_MODE_NDOF);
    
    ret = bno085_set_mode(&dev, BNO085_MODE_AR_VR_STABILIZED);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev.mode, BNO085_MODE_AR_VR_STABILIZED);
}

/* Test feature control */
TEST_F(BNO085Test, FeatureTest) {
    struct bno085_device dev;
    int ret;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    
    /* Test enabling features */
    ret = bno085_set_feature(&dev, BNO085_FEATURE_ACCELEROMETER, true);
    EXPECT_EQ(ret, 0);
    EXPECT_NE(dev.enabled_features & BNO085_FEATURE_ACCELEROMETER, 0);
    
    ret = bno085_set_feature(&dev, BNO085_FEATURE_GYROSCOPE, true);
    EXPECT_EQ(ret, 0);
    EXPECT_NE(dev.enabled_features & BNO085_FEATURE_GYROSCOPE, 0);
    
    /* Test disabling features */
    ret = bno085_set_feature(&dev, BNO085_FEATURE_ACCELEROMETER, false);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev.enabled_features & BNO085_FEATURE_ACCELEROMETER, 0);
}

/* Test sampling frequency setting */
TEST_F(BNO085Test, SamplingFrequencyTest) {
    struct bno085_device dev;
    int ret;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    
    /* Test setting different sampling frequencies */
    ret = bno085_set_sampling_frequency(&dev, 100);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev.sampling_frequency, 100);
    
    ret = bno085_set_sampling_frequency(&dev, 200);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev.sampling_frequency, 200);
    
    ret = bno085_set_sampling_frequency(&dev, 1000);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev.sampling_frequency, 1000);
    
    /* Test invalid sampling frequency */
    ret = bno085_set_sampling_frequency(&dev, 0);
    EXPECT_NE(ret, 0);
    
    ret = bno085_set_sampling_frequency(&dev, 1001);
    EXPECT_NE(ret, 0);
}

/* Test data reading */
TEST_F(BNO085Test, DataReadTest) {
    struct bno085_device dev;
    int ret;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    dev.enabled_features = BNO085_FEATURE_ACCELEROMETER | 
                          BNO085_FEATURE_GYROSCOPE | 
                          BNO085_FEATURE_MAGNETOMETER | 
                          BNO085_FEATURE_ROTATION_VECTOR;
    
    /* Test reading data */
    ret = bno085_read_data(&dev);
    EXPECT_EQ(ret, 0);
    
    /* Verify data */
    for (int i = 0; i < 3; i++) {
        EXPECT_EQ(dev.accel_data[i], mock_accel_data[i]);
        EXPECT_EQ(dev.gyro_data[i], mock_gyro_data[i]);
        EXPECT_EQ(dev.mag_data[i], mock_mag_data[i]);
    }
    
    for (int i = 0; i < 4; i++) {
        EXPECT_EQ(dev.quaternion_data[i], mock_quat_data[i]);
    }
    
    EXPECT_EQ(dev.temperature_data, mock_temp_data);
}

/* Test calibration update */
TEST_F(BNO085Test, CalibrationTest) {
    struct bno085_device dev;
    int ret;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    
    /* Set up mock calibration status */
    mock_registers[BNO085_REG_CALIB_STATUS] = 0x3F; /* All sensors calibrated */
    
    /* Test updating calibration */
    ret = bno085_update_calibration(&dev);
    EXPECT_EQ(ret, 0);
    
    /* Verify calibration state */
    EXPECT_TRUE(dev.calibrated);
}

/* Test error handling */
TEST_F(BNO085Test, ErrorHandlingTest) {
    struct bno085_device dev;
    int ret;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    
    /* Test invalid chip ID */
    mock_registers[BNO085_REG_CHIP_ID] = 0x00;
    ret = bno085_core_init(&dev);
    EXPECT_NE(ret, 0);
    
    /* Restore valid chip ID */
    mock_registers[BNO085_REG_CHIP_ID] = BNO085_CHIP_ID;
    
    /* Test reset failure */
    mock_registers[BNO085_REG_STATUS] = 0x00; /* Reset not done */
    ret = bno085_reset(&dev);
    EXPECT_NE(ret, 0);
}

/* Main function */
int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
