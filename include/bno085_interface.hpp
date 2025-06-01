#ifndef BNO085_INTERFACE_HPP
#define BNO085_INTERFACE_HPP

#include <vector>
#include <string>
#include <thread>
#include <mutex>
#include <atomic>
#include <queue>
#include <condition_variable>
#include <Eigen/Core>
#include <Eigen/Geometry>
#include "ImuTypes.h"

namespace ORB_SLAM3
{

/**
 * @brief Interface class for the BNO085 IMU sensor integration with ORB-SLAM3
 * 
 * This class provides a high-level interface for integrating the BNO085 IMU sensor
 * with the ORB-SLAM3 visual-inertial SLAM system. The BNO085 is a 9-DOF sensor
 * with built-in sensor fusion capabilities, providing high-quality orientation data.
 * 
 * Key features of the BNO085:
 * - 3-axis accelerometer, gyroscope, and magnetometer
 * - Built-in ARM Cortex M0 processor with CEVA SH-2 firmware
 * - On-chip sensor fusion at up to 1kHz
 * - Multiple output modes including raw data and fused orientation
 * - I2C, SPI, and UART interfaces
 * - Predictive head tracking for VR/AR applications
 * 
 * This interface handles:
 * 1. Communication with the BNO085 sensor
 * 2. Data acquisition and synchronization
 * 3. Conversion to ORB-SLAM3 IMU data structures
 * 4. Calibration and bias estimation
 * 5. Integration with the visual-inertial tracking system
 */
class BNO085Interface
{
public:
    /**
     * @brief Enumeration of available BNO085 operation modes
     */
    enum class OperationMode {
        CONFIG,             ///< Configuration mode, sensor is idle
        IMU,                ///< Raw IMU mode (accel + gyro)
        NDOF,               ///< Nine Degrees of Freedom fusion mode (accel + gyro + mag)
        NDOF_FMC_OFF,       ///< NDOF mode with Fast Magnetic Calibration disabled
        GYRO_ONLY,          ///< Gyroscope only mode
        ACCEL_ONLY,         ///< Accelerometer only mode
        MAG_ONLY,           ///< Magnetometer only mode
        AR_VR_STABILIZED,   ///< Special mode optimized for AR/VR with stabilization
        AR_VR_PREDICTIVE    ///< Special mode with predictive tracking for AR/VR
    };

    /**
     * @brief Enumeration of available communication interfaces
     */
    enum class Interface {
        I2C,
        SPI,
        UART
    };

    /**
     * @brief Configuration structure for BNO085Interface
     */
    struct Config {
        Interface interface_type = Interface::I2C;    ///< Communication interface type
        std::string device_path = "/dev/i2c-1";       ///< Device path (e.g., I2C bus)
        int address = 0x4A;                           ///< I2C address (default: 0x4A)
        int spi_cs_pin = 0;                           ///< SPI chip select pin (if using SPI)
        int uart_baudrate = 115200;                   ///< UART baudrate (if using UART)
        OperationMode mode = OperationMode::NDOF;     ///< Sensor operation mode
        float sample_rate_hz = 100.0f;                ///< Desired sample rate in Hz
        bool use_magnetometer = true;                 ///< Whether to use magnetometer data
        bool use_sensor_fusion = true;                ///< Whether to use on-chip sensor fusion
        bool enable_calibration = true;               ///< Whether to enable continuous calibration
        
        // IMU noise parameters (used for ORB-SLAM3 integration)
        float gyro_noise = 1.7e-4f;                   ///< Gyroscope noise (rad/s/sqrt(Hz))
        float accel_noise = 2.0e-3f;                  ///< Accelerometer noise (m/s^2/sqrt(Hz))
        float gyro_walk = 1.9e-5f;                    ///< Gyroscope random walk (rad/s^2/sqrt(Hz))
        float accel_walk = 3.0e-3f;                   ///< Accelerometer random walk (m/s^3/sqrt(Hz))
        
        // Transformation from IMU to camera frame (Tbc)
        Eigen::Matrix4f T_bc = Eigen::Matrix4f::Identity(); ///< Transform from body (IMU) to camera
    };

    /**
     * @brief Constructor with configuration
     * @param config Configuration structure
     */
    BNO085Interface(const Config& config);
    
    /**
     * @brief Destructor
     */
    ~BNO085Interface();
    
    /**
     * @brief Initialize the BNO085 sensor and start data acquisition
     * @return True if initialization was successful, false otherwise
     */
    bool Initialize();
    
    /**
     * @brief Start data acquisition in a separate thread
     * @return True if acquisition started successfully, false otherwise
     */
    bool StartAcquisition();
    
    /**
     * @brief Stop data acquisition
     */
    void StopAcquisition();
    
    /**
     * @brief Get the latest IMU measurements
     * @param max_samples Maximum number of samples to retrieve (0 for all available)
     * @return Vector of IMU::Point measurements
     */
    std::vector<IMU::Point> GetMeasurements(size_t max_samples = 0);
    
    /**
     * @brief Get the latest IMU measurements within a time range
     * @param start_time Start time in seconds
     * @param end_time End time in seconds
     * @return Vector of IMU::Point measurements
     */
    std::vector<IMU::Point> GetMeasurementsInTimeRange(double start_time, double end_time);
    
    /**
     * @brief Get the latest orientation estimate from the BNO085's internal fusion
     * @return Quaternion representing the orientation (w, x, y, z)
     */
    Eigen::Quaternionf GetOrientation();
    
    /**
     * @brief Get the latest calibration status
     * @return Calibration status as a vector (accel, gyro, mag, system) with values 0-3 (0=uncalibrated, 3=fully calibrated)
     */
    std::vector<int> GetCalibrationStatus();
    
    /**
     * @brief Perform a self-test of the sensor
     * @return True if self-test passed, false otherwise
     */
    bool SelfTest();
    
    /**
     * @brief Reset the sensor
     * @return True if reset was successful, false otherwise
     */
    bool Reset();
    
    /**
     * @brief Set the operation mode
     * @param mode New operation mode
     * @return True if mode was set successfully, false otherwise
     */
    bool SetOperationMode(OperationMode mode);
    
    /**
     * @brief Set the sample rate
     * @param rate_hz Sample rate in Hz
     * @return True if sample rate was set successfully, false otherwise
     */
    bool SetSampleRate(float rate_hz);
    
    /**
     * @brief Get the current IMU calibration
     * @return IMU::Calib object with current calibration parameters
     */
    IMU::Calib GetCalibration() const;
    
    /**
     * @brief Set the IMU calibration
     * @param calib IMU::Calib object with calibration parameters
     */
    void SetCalibration(const IMU::Calib& calib);
    
    /**
     * @brief Get the current IMU bias estimate
     * @return IMU::Bias object with current bias estimates
     */
    IMU::Bias GetCurrentBias() const;
    
    /**
     * @brief Set the IMU bias
     * @param bias IMU::Bias object with bias values
     */
    void SetBias(const IMU::Bias& bias);
    
    /**
     * @brief Get the transformation from IMU to camera frame
     * @return Sophus::SE3f object representing the transformation
     */
    Sophus::SE3<float> GetImuToCameraTransform() const;
    
    /**
     * @brief Set the transformation from IMU to camera frame
     * @param T_bc Sophus::SE3f object representing the transformation
     */
    void SetImuToCameraTransform(const Sophus::SE3<float>& T_bc);
    
    /**
     * @brief Check if the sensor is currently connected and responding
     * @return True if sensor is connected, false otherwise
     */
    bool IsConnected() const;
    
    /**
     * @brief Get the current sensor temperature
     * @return Temperature in degrees Celsius
     */
    float GetTemperature() const;
    
    /**
     * @brief Get the current sensor status
     * @return Status code (0 = normal, non-zero = error)
     */
    int GetStatus() const;
    
    /**
     * @brief Get the sensor firmware version
     * @return Firmware version string
     */
    std::string GetFirmwareVersion() const;

private:
    // Configuration
    Config mConfig;
    
    // Communication interface
    int mDeviceHandle;
    
    // Thread management
    std::thread mAcquisitionThread;
    std::atomic<bool> mRunning;
    std::mutex mDataMutex;
    std::condition_variable mDataCondition;
    
    // Data storage
    std::queue<IMU::Point> mMeasurementQueue;
    size_t mMaxQueueSize;
    
    // Calibration and state
    IMU::Calib mCalibration;
    IMU::Bias mCurrentBias;
    Sophus::SE3<float> mT_bc;  // Transform from body (IMU) to camera
    
    // Sensor state
    std::atomic<bool> mIsConnected;
    std::atomic<int> mSensorStatus;
    std::atomic<float> mTemperature;
    std::string mFirmwareVersion;
    std::vector<int> mCalibrationStatus;
    Eigen::Quaternionf mLastOrientation;
    
    // Private methods
    
    /**
     * @brief Open the communication interface to the sensor
     * @return True if successful, false otherwise
     */
    bool OpenInterface();
    
    /**
     * @brief Close the communication interface
     */
    void CloseInterface();
    
    /**
     * @brief Configure the sensor with the current settings
     * @return True if successful, false otherwise
     */
    bool ConfigureSensor();
    
    /**
     * @brief Read raw data from the sensor
     * @return True if successful, false otherwise
     */
    bool ReadRawData();
    
    /**
     * @brief Process raw data into IMU measurements
     */
    void ProcessRawData();
    
    /**
     * @brief Main acquisition thread function
     */
    void AcquisitionThreadFunc();
    
    /**
     * @brief Update the sensor calibration status
     * @return True if successful, false otherwise
     */
    bool UpdateCalibrationStatus();
    
    /**
     * @brief Convert raw sensor data to ORB-SLAM3 IMU::Point
     * @param accel_x Accelerometer X value
     * @param accel_y Accelerometer Y value
     * @param accel_z Accelerometer Z value
     * @param gyro_x Gyroscope X value
     * @param gyro_y Gyroscope Y value
     * @param gyro_z Gyroscope Z value
     * @param timestamp Timestamp in seconds
     * @return IMU::Point object
     */
    IMU::Point ConvertToImuPoint(
        float accel_x, float accel_y, float accel_z,
        float gyro_x, float gyro_y, float gyro_z,
        double timestamp);
    
    /**
     * @brief Apply calibration and bias correction to raw measurements
     * @param raw_point Raw IMU measurement
     * @return Corrected IMU measurement
     */
    IMU::Point ApplyCalibrationAndBias(const IMU::Point& raw_point);
};

} // namespace ORB_SLAM3

#endif // BNO085_INTERFACE_HPP
