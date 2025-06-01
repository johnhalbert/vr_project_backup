#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <vector>
#include <random>
#include <chrono>

/* Mock includes to replace kernel headers */
#include "mock_kernel.h"
#include "mock_iio.h"

/* Include driver header with special define to handle kernel dependencies */
#define UNIT_TESTING
#include "../bno085_core.h"

using ::testing::_;
using ::testing::Return;
using ::testing::Invoke;

class BNO085SimulationTest : public ::testing::Test {
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
        
        /* Initialize random number generator */
        rng.seed(std::chrono::system_clock::now().time_since_epoch().count());
    }
    
    void TearDown() override {
        mock_device_destroy(dev);
    }
    
    /* Generate simulated IMU data for a specific motion pattern */
    void generate_motion_data(const std::string& motion_type) {
        if (motion_type == "stationary") {
            /* Stationary with slight noise */
            for (int i = 0; i < 3; i++) {
                mock_accel_data[i] = normal_dist(rng) * 0.01;
                mock_gyro_data[i] = normal_dist(rng) * 0.005;
                mock_mag_data[i] = normal_dist(rng) * 0.02;
            }
            
            /* Identity quaternion with slight noise */
            mock_quat_data[0] = 1.0 + normal_dist(rng) * 0.001;
            for (int i = 1; i < 4; i++) {
                mock_quat_data[i] = normal_dist(rng) * 0.001;
            }
        } else if (motion_type == "rotation") {
            /* Rotation around Y axis */
            mock_gyro_data[0] = 0.0 + normal_dist(rng) * 0.01;
            mock_gyro_data[1] = 1.0 + normal_dist(rng) * 0.01;
            mock_gyro_data[2] = 0.0 + normal_dist(rng) * 0.01;
            
            /* No acceleration */
            for (int i = 0; i < 3; i++) {
                mock_accel_data[i] = normal_dist(rng) * 0.01;
            }
            
            /* Quaternion representing rotation around Y */
            mock_quat_data[0] = 0.9659; /* cos(15°) */
            mock_quat_data[1] = 0.0;
            mock_quat_data[2] = 0.2588; /* sin(15°) */
            mock_quat_data[3] = 0.0;
        } else if (motion_type == "translation") {
            /* Forward acceleration */
            mock_accel_data[0] = 0.0 + normal_dist(rng) * 0.01;
            mock_accel_data[1] = 0.0 + normal_dist(rng) * 0.01;
            mock_accel_data[2] = 1.0 + normal_dist(rng) * 0.01;
            
            /* No rotation */
            for (int i = 0; i < 3; i++) {
                mock_gyro_data[i] = normal_dist(rng) * 0.01;
            }
            
            /* Identity quaternion with slight noise */
            mock_quat_data[0] = 1.0 + normal_dist(rng) * 0.001;
            for (int i = 1; i < 4; i++) {
                mock_quat_data[i] = normal_dist(rng) * 0.001;
            }
        } else if (motion_type == "vr_head_turn") {
            /* Fast rotation around Y axis (head turning) */
            mock_gyro_data[0] = 0.0 + normal_dist(rng) * 0.01;
            mock_gyro_data[1] = 3.0 + normal_dist(rng) * 0.05;
            mock_gyro_data[2] = 0.0 + normal_dist(rng) * 0.01;
            
            /* Slight acceleration due to head movement */
            mock_accel_data[0] = 0.2 + normal_dist(rng) * 0.02;
            mock_accel_data[1] = 0.0 + normal_dist(rng) * 0.01;
            mock_accel_data[2] = 0.0 + normal_dist(rng) * 0.01;
            
            /* Quaternion representing fast rotation around Y */
            mock_quat_data[0] = 0.9239; /* cos(22.5°) */
            mock_quat_data[1] = 0.0;
            mock_quat_data[2] = 0.3827; /* sin(22.5°) */
            mock_quat_data[3] = 0.0;
        } else if (motion_type == "vr_head_nod") {
            /* Rotation around X axis (head nodding) */
            mock_gyro_data[0] = 2.0 + normal_dist(rng) * 0.05;
            mock_gyro_data[1] = 0.0 + normal_dist(rng) * 0.01;
            mock_gyro_data[2] = 0.0 + normal_dist(rng) * 0.01;
            
            /* Slight acceleration due to head movement */
            mock_accel_data[0] = 0.0 + normal_dist(rng) * 0.01;
            mock_accel_data[1] = 0.2 + normal_dist(rng) * 0.02;
            mock_accel_data[2] = 0.0 + normal_dist(rng) * 0.01;
            
            /* Quaternion representing rotation around X */
            mock_quat_data[0] = 0.9659; /* cos(15°) */
            mock_quat_data[1] = 0.2588; /* sin(15°) */
            mock_quat_data[2] = 0.0;
            mock_quat_data[3] = 0.0;
        }
        
        /* Update temperature */
        mock_temp_data = 25 + normal_dist(rng) * 0.1;
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
    
    std::mt19937 rng;
    std::normal_distribution<float> normal_dist{0.0, 1.0};
    
    static u8 mock_registers[256];
    static s16 mock_accel_data[3];
    static s16 mock_gyro_data[3];
    static s16 mock_mag_data[3];
    static s16 mock_quat_data[4];
    static s16 mock_temp_data;
};

/* Initialize static members */
u8 BNO085SimulationTest::mock_registers[256] = {0};
s16 BNO085SimulationTest::mock_accel_data[3] = {0};
s16 BNO085SimulationTest::mock_gyro_data[3] = {0};
s16 BNO085SimulationTest::mock_mag_data[3] = {0};
s16 BNO085SimulationTest::mock_quat_data[4] = {0};
s16 BNO085SimulationTest::mock_temp_data = 0;

/* Test stationary motion pattern */
TEST_F(BNO085SimulationTest, StationaryMotionTest) {
    struct bno085_device dev;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    dev.enabled_features = BNO085_FEATURE_ACCELEROMETER | 
                          BNO085_FEATURE_GYROSCOPE | 
                          BNO085_FEATURE_MAGNETOMETER | 
                          BNO085_FEATURE_ROTATION_VECTOR;
    
    /* Generate stationary motion data */
    generate_motion_data("stationary");
    
    /* Read data */
    int ret = bno085_read_data(&dev);
    EXPECT_EQ(ret, 0);
    
    /* Verify data is within expected ranges for stationary motion */
    for (int i = 0; i < 3; i++) {
        EXPECT_NEAR(dev.accel_data[i], 0, 100); /* Near zero with small noise */
        EXPECT_NEAR(dev.gyro_data[i], 0, 50);   /* Near zero with small noise */
    }
    
    /* Quaternion should be near identity */
    EXPECT_NEAR(dev.quaternion_data[0], 16384, 100); /* w ~= 1.0 */
    EXPECT_NEAR(dev.quaternion_data[1], 0, 100);     /* x ~= 0.0 */
    EXPECT_NEAR(dev.quaternion_data[2], 0, 100);     /* y ~= 0.0 */
    EXPECT_NEAR(dev.quaternion_data[3], 0, 100);     /* z ~= 0.0 */
}

/* Test VR head turning motion pattern */
TEST_F(BNO085SimulationTest, VRHeadTurnTest) {
    struct bno085_device dev;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    dev.enabled_features = BNO085_FEATURE_ACCELEROMETER | 
                          BNO085_FEATURE_GYROSCOPE | 
                          BNO085_FEATURE_MAGNETOMETER | 
                          BNO085_FEATURE_ROTATION_VECTOR;
    
    /* Set VR-specific mode */
    bno085_set_mode(&dev, BNO085_MODE_AR_VR_STABILIZED);
    
    /* Generate VR head turn motion data */
    generate_motion_data("vr_head_turn");
    
    /* Read data */
    int ret = bno085_read_data(&dev);
    EXPECT_EQ(ret, 0);
    
    /* Verify gyro data shows rotation around Y axis */
    EXPECT_NEAR(dev.gyro_data[0], 0, 100);
    EXPECT_GT(dev.gyro_data[1], 2000); /* Significant Y rotation */
    EXPECT_NEAR(dev.gyro_data[2], 0, 100);
    
    /* Quaternion should represent rotation around Y */
    EXPECT_NEAR(dev.quaternion_data[0], 15126, 200); /* w ~= 0.9239 */
    EXPECT_NEAR(dev.quaternion_data[1], 0, 100);     /* x ~= 0.0 */
    EXPECT_NEAR(dev.quaternion_data[2], 6270, 200);  /* y ~= 0.3827 */
    EXPECT_NEAR(dev.quaternion_data[3], 0, 100);     /* z ~= 0.0 */
}

/* Test VR head nodding motion pattern */
TEST_F(BNO085SimulationTest, VRHeadNodTest) {
    struct bno085_device dev;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    dev.enabled_features = BNO085_FEATURE_ACCELEROMETER | 
                          BNO085_FEATURE_GYROSCOPE | 
                          BNO085_FEATURE_MAGNETOMETER | 
                          BNO085_FEATURE_ROTATION_VECTOR;
    
    /* Set VR-specific mode */
    bno085_set_mode(&dev, BNO085_MODE_AR_VR_STABILIZED);
    
    /* Generate VR head nod motion data */
    generate_motion_data("vr_head_nod");
    
    /* Read data */
    int ret = bno085_read_data(&dev);
    EXPECT_EQ(ret, 0);
    
    /* Verify gyro data shows rotation around X axis */
    EXPECT_GT(dev.gyro_data[0], 1500); /* Significant X rotation */
    EXPECT_NEAR(dev.gyro_data[1], 0, 100);
    EXPECT_NEAR(dev.gyro_data[2], 0, 100);
    
    /* Quaternion should represent rotation around X */
    EXPECT_NEAR(dev.quaternion_data[0], 15824, 200); /* w ~= 0.9659 */
    EXPECT_NEAR(dev.quaternion_data[1], 4240, 200);  /* x ~= 0.2588 */
    EXPECT_NEAR(dev.quaternion_data[2], 0, 100);     /* y ~= 0.0 */
    EXPECT_NEAR(dev.quaternion_data[3], 0, 100);     /* z ~= 0.0 */
}

/* Test motion sequence simulation */
TEST_F(BNO085SimulationTest, MotionSequenceTest) {
    struct bno085_device dev;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    dev.enabled_features = BNO085_FEATURE_ACCELEROMETER | 
                          BNO085_FEATURE_GYROSCOPE | 
                          BNO085_FEATURE_MAGNETOMETER | 
                          BNO085_FEATURE_ROTATION_VECTOR;
    
    /* Set VR-specific mode */
    bno085_set_mode(&dev, BNO085_MODE_AR_VR_PREDICTIVE);
    
    /* Define motion sequence */
    std::vector<std::string> motion_sequence = {
        "stationary", "vr_head_turn", "stationary", 
        "vr_head_nod", "stationary", "translation"
    };
    
    /* Run through motion sequence */
    for (const auto& motion : motion_sequence) {
        /* Generate motion data */
        generate_motion_data(motion);
        
        /* Read data */
        int ret = bno085_read_data(&dev);
        EXPECT_EQ(ret, 0);
        
        /* Verify data is consistent with motion type */
        if (motion == "stationary") {
            for (int i = 0; i < 3; i++) {
                EXPECT_NEAR(dev.accel_data[i], 0, 100);
                EXPECT_NEAR(dev.gyro_data[i], 0, 50);
            }
        } else if (motion == "vr_head_turn") {
            EXPECT_NEAR(dev.gyro_data[0], 0, 100);
            EXPECT_GT(dev.gyro_data[1], 2000);
            EXPECT_NEAR(dev.gyro_data[2], 0, 100);
        } else if (motion == "vr_head_nod") {
            EXPECT_GT(dev.gyro_data[0], 1500);
            EXPECT_NEAR(dev.gyro_data[1], 0, 100);
            EXPECT_NEAR(dev.gyro_data[2], 0, 100);
        } else if (motion == "translation") {
            EXPECT_NEAR(dev.accel_data[0], 0, 100);
            EXPECT_NEAR(dev.accel_data[1], 0, 100);
            EXPECT_GT(dev.accel_data[2], 500);
        }
    }
}

/* Test VR-specific modes */
TEST_F(BNO085SimulationTest, VRModesTest) {
    struct bno085_device dev;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    dev.enabled_features = BNO085_FEATURE_ACCELEROMETER | 
                          BNO085_FEATURE_GYROSCOPE | 
                          BNO085_FEATURE_ROTATION_VECTOR;
    
    /* Test AR/VR stabilized mode */
    int ret = bno085_set_mode(&dev, BNO085_MODE_AR_VR_STABILIZED);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev.mode, BNO085_MODE_AR_VR_STABILIZED);
    
    /* Generate VR head turn motion data */
    generate_motion_data("vr_head_turn");
    
    /* Read data */
    ret = bno085_read_data(&dev);
    EXPECT_EQ(ret, 0);
    
    /* Test AR/VR predictive mode */
    ret = bno085_set_mode(&dev, BNO085_MODE_AR_VR_PREDICTIVE);
    EXPECT_EQ(ret, 0);
    EXPECT_EQ(dev.mode, BNO085_MODE_AR_VR_PREDICTIVE);
    
    /* Generate VR head turn motion data */
    generate_motion_data("vr_head_turn");
    
    /* Read data */
    ret = bno085_read_data(&dev);
    EXPECT_EQ(ret, 0);
}

/* Main function */
int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
