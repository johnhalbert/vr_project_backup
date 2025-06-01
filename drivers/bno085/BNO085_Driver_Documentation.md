# BNO085 IMU Driver Documentation

## Overview

The BNO085 IMU driver is a Linux kernel driver for the Bosch BNO085 9-axis IMU (Inertial Measurement Unit) with built-in sensor fusion. This driver is specifically optimized for VR applications, providing high-rate sampling, low-latency interrupt handling, and specialized AR/VR operation modes.

## Architecture

The driver follows a modular architecture with the following components:

1. **Core Driver**: Implements the main functionality and IIO interface
2. **Transport Layers**: Separate modules for I2C, SPI, and UART communication
3. **IIO Integration**: Exposes sensor data through the Linux Industrial I/O subsystem
4. **VR-Specific Optimizations**: Special modes and features for VR applications

### File Structure

```
drivers/bno085/
├── bno085_core.h       # Core driver header
├── bno085_core.c       # Core driver implementation
├── bno085_i2c.c        # I2C transport layer
├── bno085_spi.c        # SPI transport layer
├── bno085_uart.c       # UART transport layer
├── Makefile            # Build configuration
└── Kconfig             # Kernel configuration options
```

## Features

### Sensor Capabilities

- **Accelerometer**: 3-axis acceleration measurement
- **Gyroscope**: 3-axis angular velocity measurement
- **Magnetometer**: 3-axis magnetic field measurement
- **Quaternion**: Absolute orientation using quaternion representation
- **Temperature**: Internal temperature sensor

### VR-Specific Features

- **High-Rate Sampling**: Support for up to 1000Hz sampling rate
- **Low-Latency Interrupts**: Optimized interrupt handling (<500μs latency)
- **AR/VR Operation Modes**:
  - `BNO085_MODE_AR_VR_STABILIZED`: Optimized for stable head tracking
  - `BNO085_MODE_AR_VR_PREDICTIVE`: Includes motion prediction for lower perceived latency

### Performance Characteristics

- **Sampling Rate**: Configurable from 1Hz to 1000Hz
- **Interrupt Latency**: <500μs from sensor event to data available
- **Mode Switching**: <60ms to change operation modes
- **Data Acquisition**: <500μs to read complete sensor data set

## Integration with Linux Kernel

### IIO Subsystem

The driver integrates with the Linux Industrial I/O (IIO) subsystem, providing:

- Standard IIO channels for all sensors
- Triggered buffer support for efficient data acquisition
- Sysfs interface for configuration and data access
- Debugfs interface for advanced debugging

### Device Tree Support

Example device tree entry for I2C:

```
bno085@28 {
    compatible = "bosch,bno085";
    reg = <0x28>;
    interrupt-parent = <&gpio2>;
    interrupts = <5 IRQ_TYPE_EDGE_RISING>;
};
```

Example device tree entry for SPI:

```
bno085@0 {
    compatible = "bosch,bno085";
    reg = <0>;
    spi-max-frequency = <1000000>;
    interrupt-parent = <&gpio2>;
    interrupts = <5 IRQ_TYPE_EDGE_RISING>;
};
```

## Usage

### Kernel Configuration

Enable the driver in the kernel configuration:

```
Device Drivers --->
    Industrial I/O support --->
        Inertial measurement units --->
            [*] Bosch BNO085 9-axis IMU
            [*]   BNO085 I2C interface support
            [*]   BNO085 SPI interface support
            [*]   BNO085 VR-specific optimizations
```

### Sysfs Interface

The driver exposes the following sysfs attributes:

- `/sys/bus/iio/devices/iio:deviceX/in_accel_[x|y|z]_raw`: Raw accelerometer data
- `/sys/bus/iio/devices/iio:deviceX/in_anglvel_[x|y|z]_raw`: Raw gyroscope data
- `/sys/bus/iio/devices/iio:deviceX/in_magn_[x|y|z]_raw`: Raw magnetometer data
- `/sys/bus/iio/devices/iio:deviceX/in_rot_[w|x|y|z]_raw`: Raw quaternion data
- `/sys/bus/iio/devices/iio:deviceX/in_temp_raw`: Raw temperature data
- `/sys/bus/iio/devices/iio:deviceX/mode`: Operation mode
- `/sys/bus/iio/devices/iio:deviceX/calibration_status`: Calibration status
- `/sys/bus/iio/devices/iio:deviceX/sampling_frequency`: Sampling frequency in Hz

### Example Code

Reading sensor data from userspace:

```c
#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdint.h>

int main() {
    int fd;
    int16_t data;
    
    // Read accelerometer X axis
    fd = open("/sys/bus/iio/devices/iio:device0/in_accel_x_raw", O_RDONLY);
    if (fd < 0) {
        perror("Failed to open accelerometer");
        return -1;
    }
    
    read(fd, &data, sizeof(data));
    printf("Accelerometer X: %d\n", data);
    close(fd);
    
    // Set VR mode
    fd = open("/sys/bus/iio/devices/iio:device0/mode", O_WRONLY);
    if (fd < 0) {
        perror("Failed to open mode");
        return -1;
    }
    
    write(fd, "7", 1); // 7 = BNO085_MODE_AR_VR_STABILIZED
    close(fd);
    
    return 0;
}
```

## Testing Framework

A comprehensive testing framework has been developed for the BNO085 driver, including:

### Unit Tests

Unit tests validate individual components of the driver:
- Core initialization and configuration
- Mode setting and feature control
- Data reading and processing
- Error handling and recovery

### Integration Tests

Integration tests validate the interaction between components:
- I2C and SPI transport layers
- IIO subsystem integration
- Interrupt handling and buffer management

### Simulation Tests

Simulation tests use synthetic data to validate behavior in specific scenarios:
- Stationary operation
- Rotation and translation
- VR-specific motion patterns (head turning, nodding)
- Motion sequences

### Performance Tests

Performance tests measure critical metrics for VR applications:
- Initialization time
- Data acquisition latency
- Mode switching time
- High-rate sampling capability

## VR Integration

The BNO085 driver is designed to integrate seamlessly with the VR SLAM system:

1. **Direct Integration with TPUFeatureExtractor**: The driver provides IMU data to the TPUFeatureExtractor for visual-inertial fusion.
2. **Zero-Copy Buffer Sharing**: When possible, the driver uses zero-copy techniques to minimize latency.
3. **VR Motion Model Support**: The driver's AR/VR modes are optimized for the VR motion model.
4. **Multi-Camera Synchronization**: The driver supports synchronization with the multi-camera rig.

## Future Enhancements

Potential future enhancements for the BNO085 driver include:

1. **Dynamic Power Management**: Adaptive power management based on motion activity
2. **Advanced Calibration**: Improved calibration procedures for VR environments
3. **Motion Intent Detection**: Specialized algorithms for detecting user intent from motion patterns
4. **Extended VR Modes**: Additional operation modes for specific VR scenarios

## Conclusion

The BNO085 IMU driver provides a high-performance, VR-optimized interface to the Bosch BNO085 sensor. Its modular architecture, comprehensive feature set, and specialized VR optimizations make it an ideal component for the VR headset SLAM system.
