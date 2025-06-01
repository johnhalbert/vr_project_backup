# BNO085Interface Documentation

## Overview

The `BNO085Interface` class provides a comprehensive interface for integrating the BNO085 IMU sensor with the ORB-SLAM3 visual-inertial SLAM system. The BNO085 is a high-performance 9-DOF sensor with built-in sensor fusion capabilities, making it ideal for VR/AR applications that require precise orientation tracking.

## Key Features

- **Multiple Communication Interfaces**: Supports I2C, SPI, and UART communication protocols
- **Flexible Operation Modes**: Supports various operation modes including raw IMU, NDOF fusion, and AR/VR-optimized modes
- **High-Performance Data Acquisition**: Efficient thread-based acquisition with configurable sample rates
- **Seamless ORB-SLAM3 Integration**: Direct conversion to ORB-SLAM3's IMU data structures
- **Advanced Calibration**: Continuous calibration monitoring and bias estimation
- **Comprehensive Error Handling**: Robust error detection and reporting

## Architecture

The interface follows a producer-consumer architecture:

1. **Acquisition Thread**: Continuously reads data from the BNO085 sensor and adds measurements to a queue
2. **Main Thread**: Retrieves measurements from the queue and provides them to the SLAM system
3. **Calibration System**: Monitors and updates sensor calibration status

This architecture allows for efficient parallel processing and minimizes latency by decoupling acquisition from processing.

## Usage

### Initialization

```cpp
// Create configuration
BNO085Interface::Config config;
config.interface_type = BNO085Interface::Interface::I2C;
config.device_path = "/dev/i2c-1";
config.address = 0x4A;
config.mode = BNO085Interface::OperationMode::NDOF;
config.sample_rate_hz = 100.0f;
config.use_magnetometer = true;
config.use_sensor_fusion = true;
config.enable_calibration = true;

// Set IMU to camera transform (if known)
Eigen::Matrix4f T_bc = Eigen::Matrix4f::Identity();
T_bc.block<3,3>(0,0) = rotation_matrix;
T_bc.block<3,1>(0,3) = translation_vector;
config.T_bc = T_bc;

// Create interface
BNO085Interface imu(config);

// Initialize
if (!imu.Initialize()) {
    std::cerr << "Failed to initialize BNO085." << std::endl;
    return;
}

// Start acquisition
if (!imu.StartAcquisition()) {
    std::cerr << "Failed to start acquisition." << std::endl;
    return;
}
```

### Getting IMU Measurements

```cpp
// Get all available measurements
std::vector<IMU::Point> measurements = imu.GetMeasurements();

// Process measurements
for (const auto& point : measurements) {
    // Access accelerometer data
    Eigen::Vector3f accel = point.a;
    
    // Access gyroscope data
    Eigen::Vector3f gyro = point.w;
    
    // Access timestamp
    double timestamp = point.t;
    
    // Use measurements in SLAM system
    // ...
}
```

### Getting Measurements in a Time Range

```cpp
// Get measurements between two timestamps
double start_time = 1621234567.0;
double end_time = 1621234568.0;
std::vector<IMU::Point> measurements = imu.GetMeasurementsInTimeRange(start_time, end_time);
```

### Getting Orientation

```cpp
// Get current orientation as quaternion
Eigen::Quaternionf orientation = imu.GetOrientation();

// Convert to rotation matrix if needed
Eigen::Matrix3f rotation_matrix = orientation.toRotationMatrix();
```

### Checking Calibration Status

```cpp
// Get calibration status
std::vector<int> calibration_status = imu.GetCalibrationStatus();

// Check individual sensor calibration
int accel_calibration = calibration_status[0]; // 0-3 (0=uncalibrated, 3=fully calibrated)
int gyro_calibration = calibration_status[1];
int mag_calibration = calibration_status[2];
int system_calibration = calibration_status[3];

// Print calibration status
std::cout << "Calibration status: "
          << "Accel=" << accel_calibration << ", "
          << "Gyro=" << gyro_calibration << ", "
          << "Mag=" << mag_calibration << ", "
          << "System=" << system_calibration << std::endl;
```

### Setting Operation Mode

```cpp
// Set operation mode to AR/VR stabilized
imu.SetOperationMode(BNO085Interface::OperationMode::AR_VR_STABILIZED);

// Set operation mode to raw IMU
imu.SetOperationMode(BNO085Interface::OperationMode::IMU);
```

### Setting Sample Rate

```cpp
// Set sample rate to 200 Hz
imu.SetSampleRate(200.0f);
```

### Cleanup

```cpp
// Stop acquisition
imu.StopAcquisition();
```

## Integration with ORB-SLAM3

The `BNO085Interface` class is designed to integrate seamlessly with ORB-SLAM3's visual-inertial tracking system. The key integration points are:

### IMU Calibration

```cpp
// Get IMU calibration for ORB-SLAM3
IMU::Calib calib = imu.GetCalibration();

// Set IMU calibration from ORB-SLAM3
imu.SetCalibration(calib);
```

### IMU Bias

```cpp
// Get current IMU bias
IMU::Bias bias = imu.GetCurrentBias();

// Set IMU bias from ORB-SLAM3
imu.SetBias(bias);
```

### IMU to Camera Transform

```cpp
// Get IMU to camera transform
Sophus::SE3<float> T_bc = imu.GetImuToCameraTransform();

// Set IMU to camera transform
imu.SetImuToCameraTransform(T_bc);
```

## Performance Considerations

### Sample Rate Selection

The sample rate should be chosen based on the application requirements and the capabilities of the host system. For VR applications, a sample rate of 100-200 Hz is typically sufficient.

```cpp
// For VR applications, 100-200 Hz is recommended
imu.SetSampleRate(100.0f);

// For high-precision applications, higher rates may be used
imu.SetSampleRate(200.0f);
```

### Queue Size Management

The measurement queue size is limited to prevent memory issues. By default, the queue can hold up to 1000 measurements. If measurements are not consumed quickly enough, older measurements will be discarded.

### Thread Safety

The `BNO085Interface` class is thread-safe for concurrent calls to `GetMeasurements()` and other methods. The acquisition thread runs independently and adds measurements to the queue, which can be consumed by multiple threads.

## Error Handling

The interface provides comprehensive error handling through return values and error messages. Most methods return a boolean indicating success or failure, and error details can be obtained through the `GetStatus()` method.

```cpp
// Check if operation succeeded
if (!imu.SetOperationMode(BNO085Interface::OperationMode::NDOF)) {
    std::cerr << "Failed to set operation mode. Status: " << imu.GetStatus() << std::endl;
}
```

## Limitations and Future Work

### Current Limitations

1. **Simplified Implementation**: The current implementation includes simplified versions of some BNO085-specific commands and protocols. A production implementation would need to use the actual BNO085 protocol.

2. **Limited Error Recovery**: The current implementation provides basic error detection but limited error recovery capabilities.

3. **Fixed Coordinate System**: The current implementation assumes a fixed coordinate system. A more flexible implementation would allow for configurable coordinate system transformations.

### Future Work

1. **Complete BNO085 Protocol**: Implement the complete BNO085 protocol for more robust communication.

2. **Advanced Calibration**: Implement more advanced calibration procedures and storage.

3. **Dynamic Bias Estimation**: Implement dynamic bias estimation and compensation.

4. **Power Management**: Implement power management features for mobile applications.

## Conclusion

The `BNO085Interface` class provides a comprehensive interface for integrating the BNO085 IMU sensor with the ORB-SLAM3 visual-inertial SLAM system. It offers flexible configuration options, efficient data acquisition, and seamless integration with the SLAM system, making it ideal for VR/AR applications that require precise orientation tracking.
