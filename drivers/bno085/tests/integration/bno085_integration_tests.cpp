#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <linux/iio/iio.h>
#include <linux/iio/buffer.h>
#include <linux/iio/trigger.h>

/* Mock includes to replace kernel headers */
#include "mock_kernel.h"
#include "mock_i2c.h"
#include "mock_spi.h"
#include "mock_iio.h"

/* Include driver headers with special define to handle kernel dependencies */
#define UNIT_TESTING
#include "../bno085_core.h"
#include "../bno085_i2c.c"
#include "../bno085_spi.c"

using ::testing::_;
using ::testing::Return;
using ::testing::SetArgPointee;
using ::testing::DoAll;
using ::testing::Invoke;

class BNO085IntegrationTest : public ::testing::Test {
protected:
    void SetUp() override {
        /* Initialize mock devices */
        i2c_dev = mock_i2c_device_create();
        spi_dev = mock_spi_device_create();
        
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
        
        /* Set up mock I2C functions */
        EXPECT_CALL(*(MockI2C*)i2c_dev, i2c_smbus_read_i2c_block_data(_, _, _))
            .WillRepeatedly(Invoke(this, &BNO085IntegrationTest::mock_i2c_read));
        
        EXPECT_CALL(*(MockI2C*)i2c_dev, i2c_smbus_write_i2c_block_data(_, _, _))
            .WillRepeatedly(Invoke(this, &BNO085IntegrationTest::mock_i2c_write));
        
        /* Set up mock SPI functions */
        EXPECT_CALL(*(MockSPI*)spi_dev, spi_sync(_, _))
            .WillRepeatedly(Invoke(this, &BNO085IntegrationTest::mock_spi_sync));
    }
    
    void TearDown() override {
        mock_i2c_device_destroy(i2c_dev);
        mock_spi_device_destroy(spi_dev);
    }
    
    /* Mock I2C read function */
    int mock_i2c_read(u8 reg, int len, u8 *data) {
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
        
        return len;
    }
    
    /* Mock I2C write function */
    int mock_i2c_write(u8 reg, int len, const u8 *data) {
        if (len == 1) {
            mock_registers[reg] = *data;
        } else {
            memcpy(&mock_registers[reg], data, len);
        }
        
        return len;
    }
    
    /* Mock SPI sync function */
    int mock_spi_sync(struct spi_device *spi, struct spi_message *message) {
        struct spi_transfer *xfer;
        u8 reg;
        
        /* Get first transfer (command) */
        xfer = list_first_entry(&message->transfers, struct spi_transfer, transfer_list);
        reg = *(u8*)xfer->tx_buf;
        
        /* Check if it's a read or write */
        if (reg & 0x80) {
            /* Read operation */
            reg &= 0x7F;
            
            /* Get second transfer (data) */
            xfer = list_next_entry(xfer, transfer_list);
            
            /* Fill data based on register */
            if (reg == BNO085_REG_ACCEL_X && xfer->len == 6) {
                memcpy(xfer->rx_buf, mock_accel_data, 6);
            } else if (reg == BNO085_REG_GYRO_X && xfer->len == 6) {
                memcpy(xfer->rx_buf, mock_gyro_data, 6);
            } else if (reg == BNO085_REG_MAG_X && xfer->len == 6) {
                memcpy(xfer->rx_buf, mock_mag_data, 6);
            } else if (reg == BNO085_REG_QUAT_W && xfer->len == 8) {
                memcpy(xfer->rx_buf, mock_quat_data, 8);
            } else if (reg == BNO085_REG_TEMP && xfer->len == 2) {
                memcpy(xfer->rx_buf, &mock_temp_data, 2);
            } else if (xfer->len == 1) {
                *(u8*)xfer->rx_buf = mock_registers[reg];
            }
        } else {
            /* Write operation */
            
            /* Get second transfer (data) */
            xfer = list_next_entry(xfer, transfer_list);
            
            if (xfer->len == 1) {
                mock_registers[reg] = *(u8*)xfer->tx_buf;
            } else {
                memcpy(&mock_registers[reg], xfer->tx_buf, xfer->len);
            }
        }
        
        message->status = 0;
        message->actual_length = message->frame_length;
        
        return 0;
    }
    
    struct device *i2c_dev;
    struct device *spi_dev;
    
    static u8 mock_registers[256];
    static s16 mock_accel_data[3];
    static s16 mock_gyro_data[3];
    static s16 mock_mag_data[3];
    static s16 mock_quat_data[4];
    static s16 mock_temp_data;
};

/* Initialize static members */
u8 BNO085IntegrationTest::mock_registers[256] = {0};
s16 BNO085IntegrationTest::mock_accel_data[3] = {0};
s16 BNO085IntegrationTest::mock_gyro_data[3] = {0};
s16 BNO085IntegrationTest::mock_mag_data[3] = {0};
s16 BNO085IntegrationTest::mock_quat_data[4] = {0};
s16 BNO085IntegrationTest::mock_temp_data = 0;

/* Test I2C probe and initialization */
TEST_F(BNO085IntegrationTest, I2CProbeTest) {
    struct i2c_client *client = to_i2c_client(i2c_dev);
    int ret;
    
    /* Test I2C probe */
    ret = bno085_i2c_probe(client, NULL);
    EXPECT_EQ(ret, 0);
    
    /* Verify device initialization */
    struct iio_dev *indio_dev = dev_get_drvdata(i2c_dev);
    ASSERT_NE(indio_dev, nullptr);
    
    struct bno085_device *dev = iio_priv(indio_dev);
    ASSERT_NE(dev, nullptr);
    
    EXPECT_EQ(dev->state, BNO085_STATE_INITIALIZED);
    EXPECT_EQ(dev->mode, BNO085_MODE_NDOF);
    
    /* Test I2C remove */
    ret = bno085_i2c_remove(client);
    EXPECT_EQ(ret, 0);
}

/* Test SPI probe and initialization */
TEST_F(BNO085IntegrationTest, SPIProbeTest) {
    struct spi_device *spi = to_spi_device(spi_dev);
    int ret;
    
    /* Test SPI probe */
    ret = bno085_spi_probe(spi);
    EXPECT_EQ(ret, 0);
    
    /* Verify device initialization */
    struct iio_dev *indio_dev = dev_get_drvdata(spi_dev);
    ASSERT_NE(indio_dev, nullptr);
    
    struct bno085_device *dev = iio_priv(indio_dev);
    ASSERT_NE(dev, nullptr);
    
    EXPECT_EQ(dev->state, BNO085_STATE_INITIALIZED);
    EXPECT_EQ(dev->mode, BNO085_MODE_NDOF);
    
    /* Test SPI remove */
    ret = bno085_spi_remove(spi);
    EXPECT_EQ(ret, 0);
}

/* Test I2C data reading */
TEST_F(BNO085IntegrationTest, I2CDataReadTest) {
    struct i2c_client *client = to_i2c_client(i2c_dev);
    int ret;
    
    /* Probe device */
    ret = bno085_i2c_probe(client, NULL);
    EXPECT_EQ(ret, 0);
    
    /* Get device data */
    struct iio_dev *indio_dev = dev_get_drvdata(i2c_dev);
    struct bno085_device *dev = iio_priv(indio_dev);
    
    /* Enable features */
    dev->enabled_features = BNO085_FEATURE_ACCELEROMETER | 
                           BNO085_FEATURE_GYROSCOPE | 
                           BNO085_FEATURE_MAGNETOMETER | 
                           BNO085_FEATURE_ROTATION_VECTOR;
    
    /* Read data */
    ret = bno085_read_data(dev);
    EXPECT_EQ(ret, 0);
    
    /* Verify data */
    for (int i = 0; i < 3; i++) {
        EXPECT_EQ(dev->accel_data[i], mock_accel_data[i]);
        EXPECT_EQ(dev->gyro_data[i], mock_gyro_data[i]);
        EXPECT_EQ(dev->mag_data[i], mock_mag_data[i]);
    }
    
    for (int i = 0; i < 4; i++) {
        EXPECT_EQ(dev->quaternion_data[i], mock_quat_data[i]);
    }
    
    EXPECT_EQ(dev->temperature_data, mock_temp_data);
    
    /* Clean up */
    bno085_i2c_remove(client);
}

/* Test SPI data reading */
TEST_F(BNO085IntegrationTest, SPIDataReadTest) {
    struct spi_device *spi = to_spi_device(spi_dev);
    int ret;
    
    /* Probe device */
    ret = bno085_spi_probe(spi);
    EXPECT_EQ(ret, 0);
    
    /* Get device data */
    struct iio_dev *indio_dev = dev_get_drvdata(spi_dev);
    struct bno085_device *dev = iio_priv(indio_dev);
    
    /* Enable features */
    dev->enabled_features = BNO085_FEATURE_ACCELEROMETER | 
                           BNO085_FEATURE_GYROSCOPE | 
                           BNO085_FEATURE_MAGNETOMETER | 
                           BNO085_FEATURE_ROTATION_VECTOR;
    
    /* Read data */
    ret = bno085_read_data(dev);
    EXPECT_EQ(ret, 0);
    
    /* Verify data */
    for (int i = 0; i < 3; i++) {
        EXPECT_EQ(dev->accel_data[i], mock_accel_data[i]);
        EXPECT_EQ(dev->gyro_data[i], mock_gyro_data[i]);
        EXPECT_EQ(dev->mag_data[i], mock_mag_data[i]);
    }
    
    for (int i = 0; i < 4; i++) {
        EXPECT_EQ(dev->quaternion_data[i], mock_quat_data[i]);
    }
    
    EXPECT_EQ(dev->temperature_data, mock_temp_data);
    
    /* Clean up */
    bno085_spi_remove(spi);
}

/* Test I2C error handling */
TEST_F(BNO085IntegrationTest, I2CErrorHandlingTest) {
    struct i2c_client *client = to_i2c_client(i2c_dev);
    int ret;
    
    /* Set invalid chip ID */
    mock_registers[BNO085_REG_CHIP_ID] = 0x00;
    
    /* Test probe with invalid chip ID */
    ret = bno085_i2c_probe(client, NULL);
    EXPECT_NE(ret, 0);
    
    /* Restore valid chip ID */
    mock_registers[BNO085_REG_CHIP_ID] = BNO085_CHIP_ID;
}

/* Test SPI error handling */
TEST_F(BNO085IntegrationTest, SPIErrorHandlingTest) {
    struct spi_device *spi = to_spi_device(spi_dev);
    int ret;
    
    /* Set invalid chip ID */
    mock_registers[BNO085_REG_CHIP_ID] = 0x00;
    
    /* Test probe with invalid chip ID */
    ret = bno085_spi_probe(spi);
    EXPECT_NE(ret, 0);
    
    /* Restore valid chip ID */
    mock_registers[BNO085_REG_CHIP_ID] = BNO085_CHIP_ID;
}

/* Main function */
int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
