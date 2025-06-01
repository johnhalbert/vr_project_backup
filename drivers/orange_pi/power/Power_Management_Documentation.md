# Power Management System for Orange Pi CM5 VR Headset

## Overview

The Power Management System for the Orange Pi CM5 VR headset provides comprehensive power and thermal management capabilities optimized for virtual reality applications. The system is designed to balance performance and power consumption, ensuring consistent VR experiences while maximizing battery life.

## Architecture

The Power Management System follows a layered architecture:

1. **Kernel Driver Layer**: Low-level hardware access and control
2. **User-Space Service Layer**: Policy decisions and system integration
3. **Library API Layer**: Application integration
4. **Command-Line Interface**: User and script interaction

### Components

- **Kernel Driver (`orangepi_vr_power.ko`)**: Core driver that interfaces with hardware
- **User-Space Service (`vr-power-mgr`)**: Background service that monitors and manages power states
- **Library API (`libvrpower`)**: C/C++ API for applications to interact with the power system
- **Command-Line Interface (`vrpower`)**: Tool for manual control and monitoring

## Features

### Power Profiles

The system provides three predefined power profiles:

1. **High Performance**: Maximizes performance for demanding VR applications
   - CPU: 1.8-2.4 GHz, performance governor
   - GPU: 800-1000 MHz
   - NPU: 800-1000 MHz
   - Display: 90Hz refresh rate, full brightness
   - Sensor sampling: 1000Hz

2. **Balanced**: Default profile that balances performance and power consumption
   - CPU: 1.2-2.0 GHz, schedutil governor
   - GPU: 600-800 MHz
   - NPU: 600-800 MHz
   - Display: 90Hz refresh rate, 80% brightness
   - Sensor sampling: 500Hz

3. **Power Save**: Maximizes battery life for less demanding applications
   - CPU: 0.6-1.5 GHz, powersave governor
   - GPU: 400-600 MHz
   - NPU: 400-600 MHz
   - Display: 60Hz refresh rate, 60% brightness
   - Sensor sampling: 200Hz

### Battery Management

- Real-time monitoring of battery status (capacity, voltage, current, temperature)
- Multiple charging modes with temperature awareness
- Automatic profile switching based on battery level
- Overcharge protection
- USB-PD support
- Accurate time-to-empty and time-to-full estimates

### Thermal Management

- Temperature monitoring for CPU, GPU, NPU, battery, and ambient
- Multi-level thermal throttling:
  - **Normal**: No throttling
  - **Warning**: Slight frequency reduction
  - **Critical**: Significant throttling and brightness reduction
  - **Emergency**: Maximum throttling and switch to power save mode
- Emergency shutdown for critical temperatures
- Thermal zone-specific policies

### VR-Specific Optimizations

- Performance guarantees for consistent frame rates
- Prioritization of tracking and rendering processes
- Graceful performance degradation as battery depletes
- Optimized CPU core allocation for VR workloads

## Installation

### Prerequisites

- Orange Pi CM5 with VR headset hardware
- Linux kernel 5.10 or later with PREEMPT_RT patches
- Device tree with proper power management nodes

### Kernel Driver Installation

```bash
cd /path/to/drivers/orange_pi/power
make
sudo insmod orangepi_vr_power.ko
```

### User-Space Components Installation

```bash
cd /path/to/drivers/orange_pi/power
make install
```

This will install:
- The user-space service (`vr-power-mgr`)
- The library (`libvrpower.so`)
- The command-line tool (`vrpower`)

## Usage

### Command-Line Interface

The `vrpower` command-line tool provides a simple interface for controlling and monitoring the power management system:

```bash
# Get current power profile
vrpower profile get

# Set power profile
vrpower profile set balanced

# Show battery status
vrpower battery

# Show thermal status
vrpower thermal

# Monitor power status in real-time
vrpower monitor
```

### Library API

Applications can use the `libvrpower` library to interact with the power management system:

```c
#include <libvrpower.h>

// Initialize the library
vr_power_init();

// Set power profile
vr_power_set_profile(VR_POWER_PROFILE_HIGH_PERFORMANCE);

// Get battery status
vr_battery_status_info_t battery;
vr_power_get_battery_status(&battery);
printf("Battery: %d%%\n", battery.capacity);

// Register callback for battery status changes
vr_power_register_battery_callback(battery_callback);

// Clean up
vr_power_cleanup();
```

### System Service

The `vr-power-mgr` service runs in the background and manages power states automatically. It can be controlled using:

```bash
# Start the service
sudo systemctl start vr-power-mgr

# Stop the service
sudo systemctl stop vr-power-mgr

# Enable at boot
sudo systemctl enable vr-power-mgr
```

## Configuration

### System Configuration

The system configuration file is located at `/etc/vr-power/config.conf`:

```ini
[General]
DefaultProfile = balanced

[Battery]
LowWarningLevel = 15
CriticalLevel = 5

[Thermal]
CPUWarningTemp = 70
CPUCriticalTemp = 80
CPUEmergencyTemp = 90
```

### Kernel Module Parameters

The kernel module accepts the following parameters:

```bash
# Load with custom parameters
sudo insmod orangepi_vr_power.ko initial_profile=1 debug=1
```

Available parameters:
- `initial_profile`: Initial power profile (0=high, 1=balanced, 2=power_save)
- `debug`: Enable debug output (0=disabled, 1=enabled)

## Integration with VR Applications

VR applications should use the `libvrpower` library to:

1. Request appropriate power profiles based on application needs
2. Monitor battery status and warn users about low battery
3. Adapt rendering quality based on thermal conditions
4. Register for callbacks to handle power events

Example integration:

```c
void init_power_management() {
    vr_power_init();
    
    // Register callbacks
    vr_power_register_battery_callback(on_battery_changed);
    vr_power_register_thermal_callback(on_thermal_changed);
    
    // Request high performance for startup
    vr_power_set_profile(VR_POWER_PROFILE_HIGH_PERFORMANCE);
}

void on_battery_changed(const vr_battery_status_info_t *status) {
    if (status->capacity <= 15) {
        // Show low battery warning to user
        show_low_battery_warning();
        
        // Reduce quality to save power
        reduce_rendering_quality();
    }
    
    if (status->capacity <= 5) {
        // Show critical battery warning
        show_critical_battery_warning();
    }
}

void on_thermal_changed(const vr_thermal_status_info_t *status) {
    // Check if any zone is in critical state
    for (int i = 0; i < VR_THERMAL_ZONE_COUNT; i++) {
        if (status->status[i] >= VR_THERMAL_STATUS_CRITICAL) {
            // Reduce quality to manage heat
            reduce_rendering_quality();
            break;
        }
    }
}
```

## Troubleshooting

### Common Issues

1. **Driver not loading**
   - Check kernel version compatibility
   - Verify device tree configuration
   - Check dmesg for errors

2. **Power profiles not working**
   - Verify driver is loaded (`lsmod | grep orangepi_vr_power`)
   - Check permissions on sysfs entries
   - Verify hardware support

3. **Battery status incorrect**
   - Calibrate battery using `vrpower battery calibrate`
   - Check battery connections
   - Verify fuel gauge configuration

### Logs

- Kernel driver logs: `dmesg | grep "orangepi-vr-power"`
- Service logs: `journalctl -u vr-power-mgr`

## Performance Considerations

- The power management system adds minimal overhead (<1% CPU usage)
- Memory footprint is approximately 2MB (kernel + userspace)
- Battery life impact of monitoring is negligible (<0.1% per hour)

## Security Considerations

- The kernel driver requires root access for installation
- The sysfs interface requires root access for writing
- The user-space service runs with reduced privileges
- The library API is accessible to all users

## Future Enhancements

- Dynamic power profiles based on application workload
- Machine learning-based thermal prediction
- Integration with cloud-based power optimization
- Support for external battery packs
- Advanced power scheduling for background tasks

## License

This software is licensed under the GNU General Public License v2.0.

## Contact

For issues and feature requests, please contact the VR Headset Project team.
