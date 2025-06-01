# System UI Architecture for Orange Pi CM5 VR Headset

## Overview

This document outlines the architecture for the System UI component of the Orange Pi CM5 VR headset project. The System UI provides configuration, monitoring, and diagnostic capabilities through multiple interfaces, all built on a common Rust-based API layer.

## Core Design Principles

- **Multi-tiered UI**: Organized in layers from simple to advanced settings
- **Multiple interfaces**: In-VR menu, web interface, and CLI
- **Rust-based implementation**: For performance, safety, and reliability
- **Extensibility**: Designed to evolve as requirements change
- **SteamVR integration**: Optimized for use with SteamVR via OpenVR driver

## Architecture Components

### 1. Core API Layer

The foundation of the System UI is a Rust-based API layer that provides:

- Hardware access and control
- Configuration management (using TOML)
- System monitoring and diagnostics
- Security and authentication
- IPC mechanisms for inter-process communication

```
┌─────────────────────────────────────────────────────┐
│                   Core API Layer                    │
├─────────────┬─────────────┬─────────────┬──────────┤
│  Hardware   │ Configuration│   System    │ Security │
│   Access    │  Management  │  Monitoring │          │
└─────────────┴─────────────┴─────────────┴──────────┘
```

### 2. Interface Implementations

Built on top of the Core API Layer are three interface implementations:

#### 2.1 In-VR Menu System
- 3D UI components for use within VR
- Controller-based navigation
- Quick settings panel for common adjustments
- Full settings interface for detailed configuration
- Diagnostic visualization tools

#### 2.2 Web Interface
- Rust-based web server
- Responsive web UI
- WebSocket for real-time updates
- Configuration editor
- Diagnostic dashboards

#### 2.3 CLI Interface
- Command-line parser
- Configuration commands
- Monitoring utilities
- Diagnostic tools
- Scripting capabilities

```
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│    In-VR Menu   │ │  Web Interface  │ │  CLI Interface  │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘
         │                   │                   │
         v                   v                   v
┌─────────────────────────────────────────────────────────┐
│                     Core API Layer                       │
└─────────────────────────────────────────────────────────┘
```

### 3. SteamVR Integration

The System UI includes specific components for SteamVR integration:

- OpenVR driver for headset
- IMU data formatting for OpenVR
- SLAM tracking data integration with OpenVR
- SteamVR device configuration utilities
- Controller emulation configuration

```
┌─────────────────────────────────────────┐
│           SteamVR Integration           │
├─────────────┬─────────────┬─────────────┤
│  OpenVR     │ Tracking    │ Controller  │
│  Driver     │ Integration │ Emulation   │
└─────────┬───┴─────────────┴─────────────┘
          │
          v
┌─────────────────────────────────────────┐
│             Core API Layer              │
└─────────────────────────────────────────┘
```

## Configuration Categories

The System UI manages configuration in several categories:

### 1. Hardware Configuration
- Display settings (brightness, refresh rate, persistence)
- Audio settings (volume, EQ, spatial audio)
- Tracking settings (SLAM parameters, IMU calibration)
- Power settings (profiles, thermal thresholds)
- Input settings (controller mapping, button configuration)

### 2. SteamVR Configuration
- OpenVR driver settings
- Tracking data format settings
- Controller emulation settings
- Room setup configuration
- Application compatibility settings

### 3. Network Configuration
- WiFi settings
- Steam Link configuration
- QoS settings
- Connection management
- Performance monitoring

### 4. System Management
- Update management
- Diagnostic tools
- Backup/restore functionality
- Factory reset options
- User profile management

## UI Structure (Multi-Tiered Approach)

The UI is organized in a multi-tiered structure:

### Tier 1: Quick Access
Common settings users will frequently adjust:
- Display brightness and refresh rate
- Audio volume and microphone settings
- Tracking recalibration
- Battery status and power profile switching
- Steam Link connection status and quick troubleshooting

### Tier 2: Standard Configuration
Regular settings that may need occasional adjustment:
- Display color calibration and persistence settings
- Audio spatial settings and EQ
- Tracking sensitivity and prediction settings
- Network configuration for Steam Link
- Performance profiles (balanced, high performance, power saving)

### Tier 3: Advanced Configuration
Detailed settings for power users:
- Detailed hardware diagnostics and logs
- Low-level driver configuration
- SLAM and IMU data visualization and tuning
- Steam VR driver parameters and compatibility options
- Custom controller mapping and button configuration

### Tier 4: Developer/Maintenance
Settings primarily for development and maintenance:
- Firmware update and rollback options
- System logs and diagnostics export
- Factory reset and configuration backup/restore
- Development mode toggles and debug options
- Performance benchmarking and stress testing

## Security Considerations

The System UI implements several security features:

- **Authentication**: Local authentication for web interface, SSH key-based authentication for CLI
- **Authorization**: Role-based access control (admin vs. user)
- **Data Protection**: Encryption of sensitive configuration data
- **System Integrity**: Validation of all configuration changes, rollback capability

## IPC Mechanisms

Inter-process communication is handled through:

- **Driver-to-UI Communication**: Unix domain sockets
- **System Service Communication**: D-Bus with Rust wrapper
- **Web Server to UI Communication**: WebSockets for real-time updates, HTTP for configuration changes

## Implementation Plan

1. Develop Core API Layer
2. Implement basic CLI interface for testing
3. Develop web server and minimal web UI
4. Implement in-VR menu system
5. Integrate SteamVR components
6. Implement security features
7. Comprehensive testing and documentation

## Open Questions

- UI framework selection for web interface (pending further investigation)
- Specific SteamVR controller emulation requirements
- Performance targets for UI responsiveness in VR

## Next Steps

1. Implement Core API Layer with hardware access and configuration management
2. Develop basic CLI interface for initial testing and validation
3. Begin implementation of web server component
