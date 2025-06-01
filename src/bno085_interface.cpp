#include "include/bno085_interface.hpp"
#include <iostream>
#include <chrono>
#include <algorithm>
#include <cstring>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <linux/i2c-dev.h>
#include <linux/spi/spidev.h>
#include <termios.h>
#include <cmath>

// For BNO085 specific registers and commands
#define BNO085_I2C_ADDR_DEFAULT 0x4A
#define BNO085_PRODUCT_ID 0x42

// SH-2 Report IDs
#define SHTP_REPORT_PRODUCT_ID_REQ 0xF9
#define SHTP_REPORT_PRODUCT_ID_RESP 0xF8
#define SHTP_REPORT_FRS_READ_REQ 0xF4
#define SHTP_REPORT_FRS_READ_RESP 0xF3
#define SHTP_REPORT_SET_FEATURE_CMD 0xFD
#define SHTP_REPORT_GET_FEATURE_REQ 0xFE
#define SHTP_REPORT_GET_FEATURE_RESP 0xFC

// SH-2 Feature Reports
#define SENSOR_REPORTID_ACCELEROMETER 0x01
#define SENSOR_REPORTID_GYROSCOPE 0x02
#define SENSOR_REPORTID_MAGNETIC_FIELD 0x03
#define SENSOR_REPORTID_LINEAR_ACCELERATION 0x04
#define SENSOR_REPORTID_ROTATION_VECTOR 0x05
#define SENSOR_REPORTID_GAME_ROTATION_VECTOR 0x08
#define SENSOR_REPORTID_ARVR_STABILIZED_ROTATION_VECTOR 0x28
#define SENSOR_REPORTID_ARVR_STABILIZED_GAME_ROTATION_VECTOR 0x29

namespace ORB_SLAM3
{

//------------------------------------------------------------------------------
// Constructor & Destructor
//------------------------------------------------------------------------------

BNO085Interface::BNO085Interface(const Config& config)
    : mConfig(config),
      mDeviceHandle(-1),
      mRunning(false),
      mMaxQueueSize(1000),
      mIsConnected(false),
      mSensorStatus(0),
      mTemperature(0.0f)
{
    // Initialize calibration with default values
    mCalibration = IMU::Calib(
        Sophus::SE3<float>(
            Eigen::Matrix3f::Identity(),
            Eigen::Vector3f::Zero()
        ),
        mConfig.gyro_noise,
        mConfig.accel_noise,
        mConfig.gyro_walk,
        mConfig.accel_walk
    );
    
    // Initialize bias with zeros
    mCurrentBias = IMU::Bias();
    
    // Initialize IMU to camera transform
    mT_bc = Sophus::SE3<float>(
        Eigen::Matrix3f(mConfig.T_bc.block<3,3>(0,0)),
        Eigen::Vector3f(mConfig.T_bc.block<3,1>(0,3))
    );
    
    // Initialize calibration status
    mCalibrationStatus.resize(4, 0);
    
    // Initialize orientation quaternion
    mLastOrientation = Eigen::Quaternionf::Identity();
    
    std::cout << "BNO085Interface created with " 
              << (mConfig.interface_type == Interface::I2C ? "I2C" : 
                 (mConfig.interface_type == Interface::SPI ? "SPI" : "UART"))
              << " interface." << std::endl;
}

BNO085Interface::~BNO085Interface()
{
    // Stop acquisition if running
    if (mRunning) {
        StopAcquisition();
    }
    
    // Close interface if open
    if (mDeviceHandle >= 0) {
        CloseInterface();
    }
    
    std::cout << "BNO085Interface destroyed." << std::endl;
}

//------------------------------------------------------------------------------
// Public Methods
//------------------------------------------------------------------------------

bool BNO085Interface::Initialize()
{
    // Check if already initialized
    if (mDeviceHandle >= 0) {
        std::cerr << "BNO085 already initialized." << std::endl;
        return false;
    }
    
    // Open communication interface
    if (!OpenInterface()) {
        std::cerr << "Failed to open interface to BNO085." << std::endl;
        return false;
    }
    
    // Reset the sensor
    if (!Reset()) {
        std::cerr << "Failed to reset BNO085." << std::endl;
        CloseInterface();
        return false;
    }
    
    // Wait for sensor to boot
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
    
    // Check if sensor is responding
    if (!IsConnected()) {
        std::cerr << "BNO085 not responding after reset." << std::endl;
        CloseInterface();
        return false;
    }
    
    // Get firmware version
    mFirmwareVersion = GetFirmwareVersion();
    std::cout << "BNO085 firmware version: " << mFirmwareVersion << std::endl;
    
    // Configure sensor
    if (!ConfigureSensor()) {
        std::cerr << "Failed to configure BNO085." << std::endl;
        CloseInterface();
        return false;
    }
    
    // Update calibration status
    if (!UpdateCalibrationStatus()) {
        std::cerr << "Failed to get calibration status." << std::endl;
        // Not critical, continue
    }
    
    std::cout << "BNO085 initialized successfully." << std::endl;
    mIsConnected = true;
    return true;
}

bool BNO085Interface::StartAcquisition()
{
    // Check if already running
    if (mRunning) {
        std::cerr << "BNO085 acquisition already running." << std::endl;
        return false;
    }
    
    // Check if initialized
    if (mDeviceHandle < 0) {
        std::cerr << "BNO085 not initialized." << std::endl;
        return false;
    }
    
    // Start acquisition thread
    mRunning = true;
    mAcquisitionThread = std::thread(&BNO085Interface::AcquisitionThreadFunc, this);
    
    std::cout << "BNO085 acquisition started." << std::endl;
    return true;
}

void BNO085Interface::StopAcquisition()
{
    // Check if running
    if (!mRunning) {
        return;
    }
    
    // Stop acquisition thread
    mRunning = false;
    
    // Notify any waiting threads
    mDataCondition.notify_all();
    
    // Join acquisition thread
    if (mAcquisitionThread.joinable()) {
        mAcquisitionThread.join();
    }
    
    std::cout << "BNO085 acquisition stopped." << std::endl;
}

std::vector<IMU::Point> BNO085Interface::GetMeasurements(size_t max_samples)
{
    std::vector<IMU::Point> measurements;
    
    // Lock data mutex
    std::unique_lock<std::mutex> lock(mDataMutex);
    
    // Determine how many samples to retrieve
    size_t num_samples = max_samples > 0 ? std::min(max_samples, mMeasurementQueue.size()) : mMeasurementQueue.size();
    
    // Reserve space for measurements
    measurements.reserve(num_samples);
    
    // Get measurements from queue
    for (size_t i = 0; i < num_samples; ++i) {
        measurements.push_back(mMeasurementQueue.front());
        mMeasurementQueue.pop();
    }
    
    return measurements;
}

std::vector<IMU::Point> BNO085Interface::GetMeasurementsInTimeRange(double start_time, double end_time)
{
    std::vector<IMU::Point> measurements;
    std::vector<IMU::Point> temp_measurements;
    
    // Lock data mutex
    std::unique_lock<std::mutex> lock(mDataMutex);
    
    // Get all measurements
    size_t queue_size = mMeasurementQueue.size();
    temp_measurements.reserve(queue_size);
    
    for (size_t i = 0; i < queue_size; ++i) {
        temp_measurements.push_back(mMeasurementQueue.front());
        mMeasurementQueue.pop();
    }
    
    // Filter measurements by time range
    for (const auto& point : temp_measurements) {
        if (point.t >= start_time && point.t <= end_time) {
            measurements.push_back(point);
        }
        
        // If measurement is after end_time, put it back in the queue
        if (point.t > end_time) {
            mMeasurementQueue.push(point);
        }
    }
    
    return measurements;
}

Eigen::Quaternionf BNO085Interface::GetOrientation()
{
    return mLastOrientation;
}

std::vector<int> BNO085Interface::GetCalibrationStatus()
{
    // Update calibration status
    UpdateCalibrationStatus();
    return mCalibrationStatus;
}

bool BNO085Interface::SelfTest()
{
    // Check if initialized
    if (mDeviceHandle < 0) {
        std::cerr << "BNO085 not initialized." << std::endl;
        return false;
    }
    
    // Perform self-test (implementation depends on specific BNO085 commands)
    // This is a simplified implementation
    
    // Check if sensor is responding
    if (!IsConnected()) {
        return false;
    }
    
    // Check sensor status
    if (GetStatus() != 0) {
        return false;
    }
    
    return true;
}

bool BNO085Interface::Reset()
{
    // Check if initialized
    if (mDeviceHandle < 0) {
        std::cerr << "BNO085 not initialized." << std::endl;
        return false;
    }
    
    // Reset the sensor (implementation depends on specific BNO085 commands)
    // This is a simplified implementation
    
    // For I2C interface, send reset command
    if (mConfig.interface_type == Interface::I2C) {
        // Send reset command (specific to BNO085)
        uint8_t reset_cmd[] = {0x1F}; // Example reset command
        
        if (write(mDeviceHandle, reset_cmd, sizeof(reset_cmd)) != sizeof(reset_cmd)) {
            std::cerr << "Failed to send reset command." << std::endl;
            return false;
        }
    }
    // For SPI interface
    else if (mConfig.interface_type == Interface::SPI) {
        // Send reset command (specific to BNO085)
        uint8_t reset_cmd[] = {0x1F}; // Example reset command
        
        struct spi_ioc_transfer tr = {0};
        tr.tx_buf = (unsigned long)reset_cmd;
        tr.len = sizeof(reset_cmd);
        
        if (ioctl(mDeviceHandle, SPI_IOC_MESSAGE(1), &tr) < 0) {
            std::cerr << "Failed to send reset command." << std::endl;
            return false;
        }
    }
    // For UART interface
    else if (mConfig.interface_type == Interface::UART) {
        // Send reset command (specific to BNO085)
        uint8_t reset_cmd[] = {0x1F}; // Example reset command
        
        if (write(mDeviceHandle, reset_cmd, sizeof(reset_cmd)) != sizeof(reset_cmd)) {
            std::cerr << "Failed to send reset command." << std::endl;
            return false;
        }
    }
    
    // Wait for sensor to reset
    std::this_thread::sleep_for(std::chrono::milliseconds(500));
    
    return true;
}

bool BNO085Interface::SetOperationMode(OperationMode mode)
{
    // Check if initialized
    if (mDeviceHandle < 0) {
        std::cerr << "BNO085 not initialized." << std::endl;
        return false;
    }
    
    // Set operation mode (implementation depends on specific BNO085 commands)
    // This is a simplified implementation
    
    // Update config
    mConfig.mode = mode;
    
    // Reconfigure sensor
    return ConfigureSensor();
}

bool BNO085Interface::SetSampleRate(float rate_hz)
{
    // Check if initialized
    if (mDeviceHandle < 0) {
        std::cerr << "BNO085 not initialized." << std::endl;
        return false;
    }
    
    // Set sample rate (implementation depends on specific BNO085 commands)
    // This is a simplified implementation
    
    // Update config
    mConfig.sample_rate_hz = rate_hz;
    
    // Reconfigure sensor
    return ConfigureSensor();
}

IMU::Calib BNO085Interface::GetCalibration() const
{
    return mCalibration;
}

void BNO085Interface::SetCalibration(const IMU::Calib& calib)
{
    mCalibration = calib;
}

IMU::Bias BNO085Interface::GetCurrentBias() const
{
    return mCurrentBias;
}

void BNO085Interface::SetBias(const IMU::Bias& bias)
{
    mCurrentBias = bias;
}

Sophus::SE3<float> BNO085Interface::GetImuToCameraTransform() const
{
    return mT_bc;
}

void BNO085Interface::SetImuToCameraTransform(const Sophus::SE3<float>& T_bc)
{
    mT_bc = T_bc;
}

bool BNO085Interface::IsConnected() const
{
    // Check if initialized
    if (mDeviceHandle < 0) {
        return false;
    }
    
    // Check if sensor is responding (implementation depends on specific BNO085 commands)
    // This is a simplified implementation
    
    // For I2C interface, try to read product ID
    if (mConfig.interface_type == Interface::I2C) {
        // Send product ID request
        uint8_t req_cmd[] = {SHTP_REPORT_PRODUCT_ID_REQ};
        
        if (write(mDeviceHandle, req_cmd, sizeof(req_cmd)) != sizeof(req_cmd)) {
            return false;
        }
        
        // Read response
        uint8_t resp[10] = {0};
        if (read(mDeviceHandle, resp, sizeof(resp)) <= 0) {
            return false;
        }
        
        // Check product ID
        return (resp[0] == SHTP_REPORT_PRODUCT_ID_RESP && resp[1] == BNO085_PRODUCT_ID);
    }
    
    // For other interfaces, similar implementation would be needed
    
    return mIsConnected;
}

float BNO085Interface::GetTemperature() const
{
    return mTemperature;
}

int BNO085Interface::GetStatus() const
{
    return mSensorStatus;
}

std::string BNO085Interface::GetFirmwareVersion() const
{
    // Check if initialized
    if (mDeviceHandle < 0) {
        return "Unknown";
    }
    
    // If we already have the firmware version, return it
    if (!mFirmwareVersion.empty()) {
        return mFirmwareVersion;
    }
    
    // Get firmware version (implementation depends on specific BNO085 commands)
    // This is a simplified implementation
    
    // For I2C interface, try to read firmware version
    if (mConfig.interface_type == Interface::I2C) {
        // Send firmware version request
        uint8_t req_cmd[] = {0xF9, 0x00}; // Example firmware version request
        
        if (write(mDeviceHandle, req_cmd, sizeof(req_cmd)) != sizeof(req_cmd)) {
            return "Unknown";
        }
        
        // Read response
        uint8_t resp[20] = {0};
        if (read(mDeviceHandle, resp, sizeof(resp)) <= 0) {
            return "Unknown";
        }
        
        // Parse firmware version
        char version[16] = {0};
        snprintf(version, sizeof(version), "%d.%d.%d", resp[2], resp[3], resp[4]);
        return std::string(version);
    }
    
    // For other interfaces, similar implementation would be needed
    
    return "Unknown";
}

//------------------------------------------------------------------------------
// Private Methods
//------------------------------------------------------------------------------

bool BNO085Interface::OpenInterface()
{
    // Open the appropriate interface based on configuration
    
    // I2C interface
    if (mConfig.interface_type == Interface::I2C) {
        // Open I2C device
        mDeviceHandle = open(mConfig.device_path.c_str(), O_RDWR);
        if (mDeviceHandle < 0) {
            std::cerr << "Failed to open I2C device: " << mConfig.device_path << std::endl;
            return false;
        }
        
        // Set I2C slave address
        if (ioctl(mDeviceHandle, I2C_SLAVE, mConfig.address) < 0) {
            std::cerr << "Failed to set I2C slave address." << std::endl;
            close(mDeviceHandle);
            mDeviceHandle = -1;
            return false;
        }
    }
    // SPI interface
    else if (mConfig.interface_type == Interface::SPI) {
        // Open SPI device
        mDeviceHandle = open(mConfig.device_path.c_str(), O_RDWR);
        if (mDeviceHandle < 0) {
            std::cerr << "Failed to open SPI device: " << mConfig.device_path << std::endl;
            return false;
        }
        
        // Set SPI mode
        uint8_t mode = SPI_MODE_0;
        if (ioctl(mDeviceHandle, SPI_IOC_WR_MODE, &mode) < 0) {
            std::cerr << "Failed to set SPI mode." << std::endl;
            close(mDeviceHandle);
            mDeviceHandle = -1;
            return false;
        }
        
        // Set SPI bits per word
        uint8_t bits = 8;
        if (ioctl(mDeviceHandle, SPI_IOC_WR_BITS_PER_WORD, &bits) < 0) {
            std::cerr << "Failed to set SPI bits per word." << std::endl;
            close(mDeviceHandle);
            mDeviceHandle = -1;
            return false;
        }
        
        // Set SPI max speed
        uint32_t speed = 1000000; // 1 MHz
        if (ioctl(mDeviceHandle, SPI_IOC_WR_MAX_SPEED_HZ, &speed) < 0) {
            std::cerr << "Failed to set SPI max speed." << std::endl;
            close(mDeviceHandle);
            mDeviceHandle = -1;
            return false;
        }
    }
    // UART interface
    else if (mConfig.interface_type == Interface::UART) {
        // Open UART device
        mDeviceHandle = open(mConfig.device_path.c_str(), O_RDWR | O_NOCTTY);
        if (mDeviceHandle < 0) {
            std::cerr << "Failed to open UART device: " << mConfig.device_path << std::endl;
            return false;
        }
        
        // Configure UART
        struct termios tty;
        memset(&tty, 0, sizeof(tty));
        if (tcgetattr(mDeviceHandle, &tty) != 0) {
            std::cerr << "Failed to get UART attributes." << std::endl;
            close(mDeviceHandle);
            mDeviceHandle = -1;
            return false;
        }
        
        // Set baudrate
        cfsetospeed(&tty, mConfig.uart_baudrate);
        cfsetispeed(&tty, mConfig.uart_baudrate);
        
        // 8N1 mode (8 bits, no parity, 1 stop bit)
        tty.c_cflag &= ~PARENB;
        tty.c_cflag &= ~CSTOPB;
        tty.c_cflag &= ~CSIZE;
        tty.c_cflag |= CS8;
        
        // No flow control
        tty.c_cflag &= ~CRTSCTS;
        
        // Set local mode and enable receiver
        tty.c_cflag |= (CLOCAL | CREAD);
        
        // Raw input
        tty.c_lflag &= ~(ICANON | ECHO | ECHOE | ISIG);
        
        // Raw output
        tty.c_oflag &= ~OPOST;
        
        // No special handling of bytes
        tty.c_iflag &= ~(IXON | IXOFF | IXANY);
        tty.c_iflag &= ~(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL);
        
        // Set attributes
        if (tcsetattr(mDeviceHandle, TCSANOW, &tty) != 0) {
            std::cerr << "Failed to set UART attributes." << std::endl;
            close(mDeviceHandle);
            mDeviceHandle = -1;
            return false;
        }
    }
    else {
        std::cerr << "Unsupported interface type." << std::endl;
        return false;
    }
    
    return true;
}

void BNO085Interface::CloseInterface()
{
    // Close the interface
    if (mDeviceHandle >= 0) {
        close(mDeviceHandle);
        mDeviceHandle = -1;
    }
}

bool BNO085Interface::ConfigureSensor()
{
    // Configure the sensor based on the current configuration
    // This is a simplified implementation
    
    // For I2C interface
    if (mConfig.interface_type == Interface::I2C) {
        // Set operation mode
        uint8_t mode_cmd[4] = {0};
        mode_cmd[0] = SHTP_REPORT_SET_FEATURE_CMD;
        mode_cmd[1] = 0x00; // Feature report ID (depends on mode)
        
        // Set appropriate feature report ID based on mode
        switch (mConfig.mode) {
            case OperationMode::IMU:
                mode_cmd[1] = SENSOR_REPORTID_GYROSCOPE;
                break;
            case OperationMode::NDOF:
                mode_cmd[1] = SENSOR_REPORTID_ROTATION_VECTOR;
                break;
            case OperationMode::AR_VR_STABILIZED:
                mode_cmd[1] = SENSOR_REPORTID_ARVR_STABILIZED_ROTATION_VECTOR;
                break;
            default:
                mode_cmd[1] = SENSOR_REPORTID_ROTATION_VECTOR;
                break;
        }
        
        // Set sample rate (bytes 2-3)
        uint16_t interval_us = static_cast<uint16_t>(1000000.0f / mConfig.sample_rate_hz);
        mode_cmd[2] = interval_us & 0xFF;
        mode_cmd[3] = (interval_us >> 8) & 0xFF;
        
        if (write(mDeviceHandle, mode_cmd, sizeof(mode_cmd)) != sizeof(mode_cmd)) {
            std::cerr << "Failed to set operation mode." << std::endl;
            return false;
        }
        
        // Enable accelerometer if needed
        if (mConfig.mode == OperationMode::IMU || mConfig.mode == OperationMode::NDOF) {
            uint8_t accel_cmd[4] = {0};
            accel_cmd[0] = SHTP_REPORT_SET_FEATURE_CMD;
            accel_cmd[1] = SENSOR_REPORTID_ACCELEROMETER;
            
            // Set sample rate (bytes 2-3)
            accel_cmd[2] = interval_us & 0xFF;
            accel_cmd[3] = (interval_us >> 8) & 0xFF;
            
            if (write(mDeviceHandle, accel_cmd, sizeof(accel_cmd)) != sizeof(accel_cmd)) {
                std::cerr << "Failed to enable accelerometer." << std::endl;
                return false;
            }
        }
        
        // Enable gyroscope if needed
        if (mConfig.mode == OperationMode::IMU || mConfig.mode == OperationMode::NDOF) {
            uint8_t gyro_cmd[4] = {0};
            gyro_cmd[0] = SHTP_REPORT_SET_FEATURE_CMD;
            gyro_cmd[1] = SENSOR_REPORTID_GYROSCOPE;
            
            // Set sample rate (bytes 2-3)
            gyro_cmd[2] = interval_us & 0xFF;
            gyro_cmd[3] = (interval_us >> 8) & 0xFF;
            
            if (write(mDeviceHandle, gyro_cmd, sizeof(gyro_cmd)) != sizeof(gyro_cmd)) {
                std::cerr << "Failed to enable gyroscope." << std::endl;
                return false;
            }
        }
        
        // Enable magnetometer if needed
        if (mConfig.use_magnetometer && (mConfig.mode == OperationMode::NDOF)) {
            uint8_t mag_cmd[4] = {0};
            mag_cmd[0] = SHTP_REPORT_SET_FEATURE_CMD;
            mag_cmd[1] = SENSOR_REPORTID_MAGNETIC_FIELD;
            
            // Set sample rate (bytes 2-3)
            mag_cmd[2] = interval_us & 0xFF;
            mag_cmd[3] = (interval_us >> 8) & 0xFF;
            
            if (write(mDeviceHandle, mag_cmd, sizeof(mag_cmd)) != sizeof(mag_cmd)) {
                std::cerr << "Failed to enable magnetometer." << std::endl;
                return false;
            }
        }
    }
    // For other interfaces, similar implementation would be needed
    
    return true;
}

bool BNO085Interface::ReadRawData()
{
    // Read raw data from the sensor
    // This is a simplified implementation
    
    // For I2C interface
    if (mConfig.interface_type == Interface::I2C) {
        // Read data based on the current mode
        uint8_t data[32] = {0};
        
        // Read data
        if (read(mDeviceHandle, data, sizeof(data)) <= 0) {
            return false;
        }
        
        // Process data based on report ID
        uint8_t report_id = data[0];
        
        // Update sensor status
        mSensorStatus = data[1];
        
        // Update temperature
        mTemperature = static_cast<float>(data[2]);
        
        // Process data based on report ID
        switch (report_id) {
            case SENSOR_REPORTID_ACCELEROMETER:
                // Process accelerometer data
                // Example: Extract accelerometer data from bytes 3-8
                float accel_x = *reinterpret_cast<float*>(&data[3]);
                float accel_y = *reinterpret_cast<float*>(&data[7]);
                float accel_z = *reinterpret_cast<float*>(&data[11]);
                
                // Process gyroscope data (if available)
                float gyro_x = 0.0f;
                float gyro_y = 0.0f;
                float gyro_z = 0.0f;
                
                // Get timestamp
                double timestamp = std::chrono::duration_cast<std::chrono::nanoseconds>(
                    std::chrono::high_resolution_clock::now().time_since_epoch()
                ).count() / 1e9;
                
                // Create IMU point
                IMU::Point point = ConvertToImuPoint(
                    accel_x, accel_y, accel_z,
                    gyro_x, gyro_y, gyro_z,
                    timestamp
                );
                
                // Apply calibration and bias correction
                point = ApplyCalibrationAndBias(point);
                
                // Add to queue
                std::lock_guard<std::mutex> lock(mDataMutex);
                mMeasurementQueue.push(point);
                
                // Limit queue size
                if (mMeasurementQueue.size() > mMaxQueueSize) {
                    mMeasurementQueue.pop();
                }
                
                // Notify waiting threads
                mDataCondition.notify_all();
                break;
                
            case SENSOR_REPORTID_GYROSCOPE:
                // Process gyroscope data
                // Similar to accelerometer processing
                break;
                
            case SENSOR_REPORTID_ROTATION_VECTOR:
            case SENSOR_REPORTID_GAME_ROTATION_VECTOR:
            case SENSOR_REPORTID_ARVR_STABILIZED_ROTATION_VECTOR:
                // Process rotation vector data
                // Example: Extract quaternion from bytes 3-18
                float qw = *reinterpret_cast<float*>(&data[3]);
                float qx = *reinterpret_cast<float*>(&data[7]);
                float qy = *reinterpret_cast<float*>(&data[11]);
                float qz = *reinterpret_cast<float*>(&data[15]);
                
                // Update orientation
                mLastOrientation = Eigen::Quaternionf(qw, qx, qy, qz);
                break;
                
            default:
                // Unknown report ID
                break;
        }
    }
    // For other interfaces, similar implementation would be needed
    
    return true;
}

void BNO085Interface::ProcessRawData()
{
    // This method would process raw data into IMU measurements
    // In the simplified implementation, this is done directly in ReadRawData()
}

void BNO085Interface::AcquisitionThreadFunc()
{
    // Set thread name for debugging
    #ifdef __linux__
        pthread_setname_np(pthread_self(), "BNO085-Acq");
    #endif
    
    // Calculate sleep time based on sample rate
    auto sleep_time = std::chrono::microseconds(static_cast<int>(1000000.0f / mConfig.sample_rate_hz));
    
    while (mRunning) {
        // Read and process data
        if (!ReadRawData()) {
            std::cerr << "Failed to read data from BNO085." << std::endl;
            
            // Check if sensor is still connected
            if (!IsConnected()) {
                std::cerr << "BNO085 disconnected." << std::endl;
                mIsConnected = false;
                break;
            }
        }
        
        // Update calibration status periodically (every 100 samples)
        static int sample_count = 0;
        if (++sample_count >= 100) {
            UpdateCalibrationStatus();
            sample_count = 0;
        }
        
        // Sleep until next sample
        std::this_thread::sleep_for(sleep_time);
    }
}

bool BNO085Interface::UpdateCalibrationStatus()
{
    // Update calibration status
    // This is a simplified implementation
    
    // For I2C interface
    if (mConfig.interface_type == Interface::I2C) {
        // Send calibration status request
        uint8_t req_cmd[] = {0xF2, 0x00}; // Example calibration status request
        
        if (write(mDeviceHandle, req_cmd, sizeof(req_cmd)) != sizeof(req_cmd)) {
            return false;
        }
        
        // Read response
        uint8_t resp[8] = {0};
        if (read(mDeviceHandle, resp, sizeof(resp)) <= 0) {
            return false;
        }
        
        // Update calibration status
        mCalibrationStatus[0] = resp[2]; // Accelerometer
        mCalibrationStatus[1] = resp[3]; // Gyroscope
        mCalibrationStatus[2] = resp[4]; // Magnetometer
        mCalibrationStatus[3] = resp[5]; // System
    }
    // For other interfaces, similar implementation would be needed
    
    return true;
}

IMU::Point BNO085Interface::ConvertToImuPoint(
    float accel_x, float accel_y, float accel_z,
    float gyro_x, float gyro_y, float gyro_z,
    double timestamp)
{
    // Convert raw sensor data to IMU::Point
    // Apply any necessary coordinate system transformations
    
    // Create IMU point
    IMU::Point point(
        accel_x, accel_y, accel_z,
        gyro_x, gyro_y, gyro_z,
        timestamp
    );
    
    return point;
}

IMU::Point BNO085Interface::ApplyCalibrationAndBias(const IMU::Point& raw_point)
{
    // Apply calibration and bias correction to raw measurements
    
    // Create corrected point
    IMU::Point corrected_point(
        raw_point.a.x() - mCurrentBias.bax,
        raw_point.a.y() - mCurrentBias.bay,
        raw_point.a.z() - mCurrentBias.baz,
        raw_point.w.x() - mCurrentBias.bwx,
        raw_point.w.y() - mCurrentBias.bwy,
        raw_point.w.z() - mCurrentBias.bwz,
        raw_point.t
    );
    
    return corrected_point;
}

} // namespace ORB_SLAM3
