# Power Management System Architecture for Orange Pi CM5 VR Headset

## 1. Overview

This document outlines the architecture for the Power Management System for the Orange Pi CM5 VR headset. The system is designed to provide comprehensive power management capabilities including battery monitoring, charging control, thermal management, and power profiles for different VR scenarios.

## 2. System Architecture

The Power Management System follows a layered architecture with both kernel-space and user-space components:

```
+-------------------------------------------+
|              VR Applications              |
+-------------------------------------------+
                    |
+-------------------------------------------+
|         Power Management Service          |
|  (User-space daemon for policy decisions) |
+-------------------------------------------+
                    |
+-------------------------------------------+
|        Power Management Interface         |
|     (sysfs, ioctl, and library APIs)      |
+-------------------------------------------+
                    |
+-------------------------------------------+
|        Power Management Kernel Driver     |
|  (Core driver for hardware interaction)   |
+-------------------------------------------+
                    |
+-------------------------------------------+
|        Hardware Abstraction Layer         |
| (Platform-specific hardware interfaces)   |
+-------------------------------------------+
                    |
+-------------------------------------------+
|           Orange Pi CM5 Hardware          |
| (Battery, PMIC, Thermal sensors, etc.)    |
+-------------------------------------------+
```

## 3. Component Descriptions

### 3.1 Kernel-Space Components

#### 3.1.1 Power Management Kernel Driver (`orangepi_vr_power.ko`)

The core kernel driver that interfaces with the hardware and provides the foundation for power management:

- **Battery Management Module**: Interfaces with battery fuel gauge and charging hardware
- **Thermal Management Module**: Monitors temperatures and implements thermal throttling
- **DVFS Module**: Controls CPU, GPU, and NPU frequencies and voltages
- **Power Profile Module**: Implements power profiles and transitions between them
- **Sysfs Interface**: Exposes power management controls and status to user-space

#### 3.1.2 Hardware Abstraction Layer

Platform-specific code that interfaces with the Orange Pi CM5 hardware:

- **Battery HAL**: Interfaces with battery management IC
- **PMIC HAL**: Interfaces with power management IC for voltage regulation
- **Thermal HAL**: Interfaces with temperature sensors
- **Clock and Voltage HAL**: Interfaces with RK3588S clock and voltage control

### 3.2 User-Space Components

#### 3.2.1 Power Management Service (`vr-power-mgr`)

A daemon that implements power management policies and coordinates system-wide power management:

- **Policy Manager**: Implements power management policies
- **Profile Manager**: Manages power profiles and transitions
- **Thermal Manager**: Implements thermal management policies
- **Battery Manager**: Monitors battery status and implements charging policies
- **Event Manager**: Handles power-related events and notifications

#### 3.2.2 Power Management Library (`libvrpower`)

A library that provides a high-level API for applications to interact with the power management system:

- **Profile API**: Allows applications to request power profiles
- **Battery API**: Provides battery status information
- **Thermal API**: Provides thermal status information
- **Event API**: Allows applications to register for power-related events

#### 3.2.3 Power Management CLI (`vrpower`)

A command-line tool for configuring and monitoring the power management system:

- **Configuration Commands**: Set power profiles, charging policies, etc.
- **Monitoring Commands**: Monitor battery, thermal, and power status
- **Diagnostic Commands**: Run diagnostics and tests

## 4. Key Interfaces

### 4.1 Kernel-Space Interfaces

#### 4.1.1 Sysfs Interface

The primary interface between kernel and user space:

```
/sys/class/power_supply/vr_battery/
  - capacity          # Battery capacity (0-100%)
  - voltage_now       # Current battery voltage (μV)
  - current_now       # Current battery current (μA)
  - temp              # Battery temperature (0.1°C)
  - status            # Charging status (Charging, Discharging, Full)

/sys/class/power_supply/vr_charger/
  - online            # Charger connected (0/1)
  - type              # Charger type (USB, AC, Wireless)
  - current_max       # Maximum charging current (μA)

/sys/class/thermal/thermal_zone*/
  - temp              # Temperature of thermal zone (mC)
  - type              # Type of thermal zone (cpu, gpu, battery, etc.)
  - mode              # Thermal zone mode (enabled/disabled)

/sys/devices/platform/vr_power/
  - power_profile     # Current power profile
  - available_profiles # Available power profiles
  - thermal_status    # Current thermal status
  - battery_status    # Current battery status
```

#### 4.1.2 IOCTL Interface

For more complex interactions between kernel and user space:

```c
// Power profile control
#define VR_POWER_IOCTL_SET_PROFILE    _IOW('V', 1, struct vr_power_profile)
#define VR_POWER_IOCTL_GET_PROFILE    _IOR('V', 2, struct vr_power_profile)

// Thermal control
#define VR_POWER_IOCTL_SET_THERMAL    _IOW('V', 3, struct vr_thermal_config)
#define VR_POWER_IOCTL_GET_THERMAL    _IOR('V', 4, struct vr_thermal_config)

// Battery control
#define VR_POWER_IOCTL_SET_BATTERY    _IOW('V', 5, struct vr_battery_config)
#define VR_POWER_IOCTL_GET_BATTERY    _IOR('V', 6, struct vr_battery_config)
```

### 4.2 User-Space Interfaces

#### 4.2.1 Library API

The primary interface for applications:

```c
// Power profile API
int vr_power_set_profile(vr_power_profile_t profile);
vr_power_profile_t vr_power_get_profile(void);
int vr_power_register_profile_callback(vr_power_profile_callback_t callback);

// Battery API
int vr_power_get_battery_status(vr_battery_status_t *status);
int vr_power_register_battery_callback(vr_battery_callback_t callback);

// Thermal API
int vr_power_get_thermal_status(vr_thermal_status_t *status);
int vr_power_register_thermal_callback(vr_thermal_callback_t callback);
```

#### 4.2.2 D-Bus Interface

For inter-process communication:

```
org.orangepi.VRPower
  Methods:
    SetPowerProfile(string profile)
    GetPowerProfile() -> string
    GetBatteryStatus() -> dict
    GetThermalStatus() -> dict
  Signals:
    PowerProfileChanged(string profile)
    BatteryStatusChanged(dict status)
    ThermalStatusChanged(dict status)
```

## 5. Data Structures

### 5.1 Power Profiles

```c
enum vr_power_profile_type {
    VR_POWER_PROFILE_HIGH_PERFORMANCE,
    VR_POWER_PROFILE_BALANCED,
    VR_POWER_PROFILE_POWER_SAVE,
    VR_POWER_PROFILE_CUSTOM
};

struct vr_power_profile {
    enum vr_power_profile_type type;
    
    // CPU settings
    unsigned int cpu_freq_min;
    unsigned int cpu_freq_max;
    char cpu_governor[32];
    
    // GPU settings
    unsigned int gpu_freq_min;
    unsigned int gpu_freq_max;
    
    // NPU settings
    unsigned int npu_freq_min;
    unsigned int npu_freq_max;
    
    // Display settings
    unsigned int display_brightness;
    unsigned int display_refresh_rate;
    
    // Misc settings
    bool wifi_power_save;
    unsigned int sensor_rate;
};
```

### 5.2 Battery Status

```c
enum vr_battery_status_type {
    VR_BATTERY_STATUS_CHARGING,
    VR_BATTERY_STATUS_DISCHARGING,
    VR_BATTERY_STATUS_FULL,
    VR_BATTERY_STATUS_UNKNOWN
};

enum vr_charger_type {
    VR_CHARGER_TYPE_NONE,
    VR_CHARGER_TYPE_USB,
    VR_CHARGER_TYPE_AC,
    VR_CHARGER_TYPE_WIRELESS
};

struct vr_battery_status {
    enum vr_battery_status_type status;
    enum vr_charger_type charger_type;
    
    unsigned int capacity;        // 0-100%
    unsigned int voltage;         // mV
    int current;                  // mA (positive = charging, negative = discharging)
    int temperature;              // 0.1°C
    
    unsigned int time_to_empty;   // minutes
    unsigned int time_to_full;    // minutes
};
```

### 5.3 Thermal Status

```c
enum vr_thermal_zone {
    VR_THERMAL_ZONE_CPU,
    VR_THERMAL_ZONE_GPU,
    VR_THERMAL_ZONE_NPU,
    VR_THERMAL_ZONE_BATTERY,
    VR_THERMAL_ZONE_AMBIENT,
    VR_THERMAL_ZONE_COUNT
};

enum vr_thermal_status {
    VR_THERMAL_STATUS_NORMAL,
    VR_THERMAL_STATUS_WARNING,
    VR_THERMAL_STATUS_CRITICAL,
    VR_THERMAL_STATUS_EMERGENCY
};

struct vr_thermal_config {
    int trip_points[VR_THERMAL_ZONE_COUNT][3];  // Warning, Critical, Emergency
    int hysteresis[VR_THERMAL_ZONE_COUNT];      // Hysteresis for trip points
};

struct vr_thermal_status {
    enum vr_thermal_status status[VR_THERMAL_ZONE_COUNT];
    int temperature[VR_THERMAL_ZONE_COUNT];     // 0.1°C
};
```

## 6. Workflows

### 6.1 Power Profile Switching

1. Application requests a power profile change via the library API
2. Library sends request to Power Management Service via D-Bus
3. Service validates the request and updates the profile
4. Service writes the new profile to sysfs
5. Kernel driver applies the profile settings
6. Service notifies all registered applications of the profile change

### 6.2 Thermal Management

1. Kernel driver monitors temperatures via thermal sensors
2. When a temperature exceeds a trip point, the driver takes immediate action
3. Driver notifies user space via sysfs
4. Service reads the thermal status and updates its state
5. Service may adjust power profile based on thermal policy
6. Service notifies applications of thermal status change

### 6.3 Battery Management

1. Kernel driver monitors battery status via fuel gauge
2. Driver updates sysfs with battery information
3. Service reads battery status and updates its state
4. Service may adjust charging parameters based on battery policy
5. Service notifies applications of battery status change

## 7. Implementation Strategy

### 7.1 Phase 1: Kernel Driver Implementation

1. Implement basic kernel driver structure
2. Implement battery monitoring and charging control
3. Implement thermal monitoring
4. Implement DVFS control
5. Implement sysfs interface

### 7.2 Phase 2: User-Space Implementation

1. Implement Power Management Service
2. Implement library API
3. Implement CLI tool
4. Implement D-Bus interface

### 7.3 Phase 3: Integration and Testing

1. Integrate with VR applications
2. Test all power management features
3. Optimize performance and power consumption
4. Validate against requirements

## 8. Testing Strategy

### 8.1 Unit Testing

1. Test each kernel driver module independently
2. Test each user-space component independently
3. Validate all interfaces

### 8.2 Integration Testing

1. Test kernel and user-space integration
2. Test power profile switching
3. Test thermal management
4. Test battery management

### 8.3 Performance Testing

1. Measure power consumption under different profiles
2. Measure thermal performance under load
3. Validate battery life estimates

## 9. Security Considerations

1. Implement proper permission checks for power management operations
2. Validate all inputs to prevent malicious manipulation
3. Protect sensitive power management settings

## 10. Future Enhancements

1. AI-based power optimization
2. Cloud-based power profile optimization
3. Advanced battery health management
4. External cooling accessories support
