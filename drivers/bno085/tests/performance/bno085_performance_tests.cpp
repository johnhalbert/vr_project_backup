#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <chrono>
#include <thread>
#include <vector>
#include <algorithm>

/* Mock includes to replace kernel headers */
#include "mock_kernel.h"
#include "mock_iio.h"

/* Include driver header with special define to handle kernel dependencies */
#define UNIT_TESTING
#include "../bno085_core.h"

using ::testing::_;
using ::testing::Return;
using ::testing::Invoke;

class BNO085PerformanceTest : public ::testing::Test {
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
        /* Simulate register read latency */
        std::this_thread::sleep_for(std::chrono::microseconds(10));
        
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
        /* Simulate register write latency */
        std::this_thread::sleep_for(std::chrono::microseconds(10));
        
        if (len == 1) {
            mock_registers[reg] = *data;
        } else {
            memcpy(&mock_registers[reg], data, len);
        }
        
        return 0;
    }
    
    /* Mock FIFO read function */
    static int mock_read_fifo(struct device *dev, u8 *data, int len) {
        /* Simulate FIFO read latency */
        std::this_thread::sleep_for(std::chrono::microseconds(len));
        
        /* Simulate FIFO data */
        for (int i = 0; i < len; i++) {
            data[i] = i & 0xFF;
        }
        
        return 0;
    }
    
    /* Measure execution time of a function */
    template<typename Func>
    long long measure_execution_time(Func func) {
        auto start = std::chrono::high_resolution_clock::now();
        func();
        auto end = std::chrono::high_resolution_clock::now();
        return std::chrono::duration_cast<std::chrono::microseconds>(end - start).count();
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
u8 BNO085PerformanceTest::mock_registers[256] = {0};
s16 BNO085PerformanceTest::mock_accel_data[3] = {0};
s16 BNO085PerformanceTest::mock_gyro_data[3] = {0};
s16 BNO085PerformanceTest::mock_mag_data[3] = {0};
s16 BNO085PerformanceTest::mock_quat_data[4] = {0};
s16 BNO085PerformanceTest::mock_temp_data = 0;

/* Test initialization performance */
TEST_F(BNO085PerformanceTest, InitializationPerformanceTest) {
    struct bno085_device dev;
    std::vector<long long> execution_times;
    
    /* Run initialization multiple times to get average performance */
    for (int i = 0; i < 10; i++) {
        memset(&dev, 0, sizeof(dev));
        dev.dev = this->dev;
        dev.transport = this->transport;
        
        execution_times.push_back(measure_execution_time([&]() {
            bno085_core_init(&dev);
        }));
    }
    
    /* Calculate statistics */
    long long total = 0;
    for (auto time : execution_times) {
        total += time;
    }
    
    long long avg = total / execution_times.size();
    std::sort(execution_times.begin(), execution_times.end());
    long long median = execution_times[execution_times.size() / 2];
    long long min = execution_times.front();
    long long max = execution_times.back();
    
    /* Print performance statistics */
    std::cout << "Initialization Performance (microseconds):" << std::endl;
    std::cout << "  Average: " << avg << std::endl;
    std::cout << "  Median: " << median << std::endl;
    std::cout << "  Min: " << min << std::endl;
    std::cout << "  Max: " << max << std::endl;
    
    /* Verify performance meets requirements */
    EXPECT_LT(avg, 1000); /* Less than 1ms */
}

/* Test data reading performance */
TEST_F(BNO085PerformanceTest, DataReadPerformanceTest) {
    struct bno085_device dev;
    std::vector<long long> execution_times;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    dev.enabled_features = BNO085_FEATURE_ACCELEROMETER | 
                          BNO085_FEATURE_GYROSCOPE | 
                          BNO085_FEATURE_MAGNETOMETER | 
                          BNO085_FEATURE_ROTATION_VECTOR;
    
    /* Run data reading multiple times to get average performance */
    for (int i = 0; i < 100; i++) {
        execution_times.push_back(measure_execution_time([&]() {
            bno085_read_data(&dev);
        }));
    }
    
    /* Calculate statistics */
    long long total = 0;
    for (auto time : execution_times) {
        total += time;
    }
    
    long long avg = total / execution_times.size();
    std::sort(execution_times.begin(), execution_times.end());
    long long median = execution_times[execution_times.size() / 2];
    long long min = execution_times.front();
    long long max = execution_times.back();
    
    /* Print performance statistics */
    std::cout << "Data Read Performance (microseconds):" << std::endl;
    std::cout << "  Average: " << avg << std::endl;
    std::cout << "  Median: " << median << std::endl;
    std::cout << "  Min: " << min << std::endl;
    std::cout << "  Max: " << max << std::endl;
    
    /* Verify performance meets requirements */
    EXPECT_LT(avg, 500); /* Less than 500us */
}

/* Test mode switching performance */
TEST_F(BNO085PerformanceTest, ModeSwitchPerformanceTest) {
    struct bno085_device dev;
    std::vector<long long> execution_times;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    
    /* Run mode switching multiple times to get average performance */
    for (int i = 0; i < 10; i++) {
        execution_times.push_back(measure_execution_time([&]() {
            bno085_set_mode(&dev, BNO085_MODE_AR_VR_STABILIZED);
        }));
    }
    
    /* Calculate statistics */
    long long total = 0;
    for (auto time : execution_times) {
        total += time;
    }
    
    long long avg = total / execution_times.size();
    std::sort(execution_times.begin(), execution_times.end());
    long long median = execution_times[execution_times.size() / 2];
    long long min = execution_times.front();
    long long max = execution_times.back();
    
    /* Print performance statistics */
    std::cout << "Mode Switch Performance (microseconds):" << std::endl;
    std::cout << "  Average: " << avg << std::endl;
    std::cout << "  Median: " << median << std::endl;
    std::cout << "  Min: " << min << std::endl;
    std::cout << "  Max: " << max << std::endl;
    
    /* Verify performance meets requirements */
    EXPECT_LT(avg, 60000); /* Less than 60ms */
}

/* Test high-rate sampling performance */
TEST_F(BNO085PerformanceTest, HighRateSamplingTest) {
    struct bno085_device dev;
    std::vector<long long> execution_times;
    
    /* Initialize device */
    memset(&dev, 0, sizeof(dev));
    dev.dev = this->dev;
    dev.transport = this->transport;
    dev.enabled_features = BNO085_FEATURE_ACCELEROMETER | 
                          BNO085_FEATURE_GYROSCOPE | 
                          BNO085_FEATURE_ROTATION_VECTOR;
    
    /* Set high sampling rate */
    bno085_set_sampling_frequency(&dev, 1000);
    
    /* Simulate high-rate sampling */
    auto start = std::chrono::high_resolution_clock::now();
    int samples = 0;
    
    for (int i = 0; i < 1000; i++) {
        bno085_read_data(&dev);
        samples++;
    }
    
    auto end = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start).count();
    
    /* Calculate effective sampling rate */
    double rate = (samples * 1000.0) / duration;
    
    /* Print performance statistics */
    std::cout << "High-Rate Sampling Performance:" << std::endl;
    std::cout << "  Samples: " << samples << std::endl;
    std::cout << "  Duration (ms): " << duration << std::endl;
    std::cout << "  Effective Rate (Hz): " << rate << std::endl;
    
    /* Verify performance meets requirements */
    EXPECT_GT(rate, 900); /* At least 900Hz */
}

/* Main function */
int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
