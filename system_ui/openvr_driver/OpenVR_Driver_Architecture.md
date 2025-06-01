# OpenVR Driver Architecture for VR Headset

## Overview

This document outlines the architecture for the OpenVR driver component of the VR headset project. The driver enables the headset to integrate with SteamVR, providing tracking data from our SLAM system to SteamVR applications and handling input from controllers.

## Design Principles

- **Rust-First Approach**: Core implementation in Rust with minimal C++ for OpenVR interface
- **Clean Separation of Concerns**: Clear boundaries between components
- **Seamless Integration**: Tight integration with existing Core API
- **Extensibility**: Designed to evolve as requirements change
- **Performance**: Optimized for low-latency VR experiences

## Architecture Components

### 1. Driver Interface Layer (C++)

This thin C++ layer implements the required OpenVR interfaces and serves as the bridge between OpenVR and our Rust implementation:

```
┌─────────────────────────────────────────────────────┐
│              Driver Interface Layer (C++)           │
├─────────────┬─────────────┬─────────────┬──────────┤
│ HmdDriver   │ ServerDevice│   Factory   │ Native   │
│   Factory   │  Provider   │  Functions  │ Bindings │
└─────────┬───┴─────────────┴─────────────┴──────────┘
          │
          ▼
┌─────────────────────────────────────────────────────┐
│                 FFI Boundary Layer                  │
└─────────────────────────┬───────────────────────────┘
                          │
                          ▼
```

#### Components:
- **HmdDriverFactory**: Entry point for OpenVR to load our driver
- **ServerDeviceProvider**: Implements `IServerTrackedDeviceProvider` interface
- **Factory Functions**: C++ functions exposed to OpenVR
- **Native Bindings**: C++ code that calls into our Rust implementation

### 2. Rust Driver Core

The main implementation of the driver functionality in Rust:

```
┌─────────────────────────────────────────────────────┐
│                 Rust Driver Core                    │
├─────────────┬─────────────┬─────────────┬──────────┤
│  Device     │   Tracking  │   Input     │ Settings │
│  Manager    │   Provider  │   Handler   │ Manager  │
└─────────────┴─────────────┴─────────────┴──────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────┐
│                 Core API Integration                │
└─────────────────────────────────────────────────────┘
```

#### Components:
- **Device Manager**: Manages device lifecycle and registration
- **Tracking Provider**: Processes and formats tracking data
- **Input Handler**: Processes controller input and haptic feedback
- **Settings Manager**: Manages driver configuration

### 3. Device Implementations

Specific implementations for each device type:

```
┌─────────────────────────────────────────────────────┐
│                Device Implementations               │
├─────────────┬─────────────┬─────────────┬──────────┤
│    HMD      │ Controllers │  Trackers   │ Tracking │
│   Device    │             │             │References│
└─────────────┴─────────────┴─────────────┴──────────┘
```

#### Components:
- **HMD Device**: Implements headset-specific functionality
- **Controllers**: Implements controller-specific functionality
- **Trackers**: Implements tracker-specific functionality
- **Tracking References**: Implements base station emulation

### 4. Core API Integration

Integration with the existing Core API:

```
┌─────────────────────────────────────────────────────┐
│                 Core API Integration                │
├─────────────┬─────────────┬─────────────┬──────────┤
│  Hardware   │ Configuration│   SLAM     │   IPC    │
│   Access    │  Access     │  Integration│ Channels │
└─────────────┴─────────────┴─────────────┴──────────┘
```

#### Components:
- **Hardware Access**: Interfaces with Core API hardware components
- **Configuration Access**: Uses Core API configuration system
- **SLAM Integration**: Integrates with SLAM tracking data
- **IPC Channels**: Communication channels with Core API

## Detailed Component Design

### 1. Driver Interface Layer (C++)

#### HmdDriverFactory

```cpp
// Entry point for OpenVR
extern "C" __declspec(dllexport) void* HmdDriverFactory(const char* pInterfaceName, int* pReturnCode)
{
    if (0 == strcmp(IServerTrackedDeviceProvider_Version, pInterfaceName))
    {
        return vr_driver_get_server_provider();
    }
    
    if (pReturnCode)
        *pReturnCode = VRInitError_Init_InterfaceNotFound;
    
    return nullptr;
}
```

#### ServerDeviceProvider

```cpp
class VRDriverServerProvider : public vr::IServerTrackedDeviceProvider
{
public:
    // OpenVR interface implementation
    vr::EVRInitError Init(vr::IVRDriverContext* pDriverContext) override;
    void Cleanup() override;
    const char* const* GetInterfaceVersions() override;
    void RunFrame() override;
    bool ShouldBlockStandbyMode() override;
    void EnterStandby() override;
    void LeaveStandby() override;
    
private:
    // Rust driver core handle
    void* rust_driver_handle;
};
```

### 2. Rust Driver Core

#### FFI Interface

```rust
#[no_mangle]
pub extern "C" fn vr_driver_get_server_provider() -> *mut c_void {
    // Create and return a pointer to our server provider
    Box::into_raw(Box::new(ServerProviderWrapper::new())) as *mut c_void
}

#[no_mangle]
pub extern "C" fn vr_driver_init(context: *mut c_void) -> i32 {
    // Initialize the driver with the given context
    let driver = DriverCore::new();
    driver.initialize(context)
}

#[no_mangle]
pub extern "C" fn vr_driver_run_frame() {
    // Run a single frame update
    DRIVER.with(|driver| {
        driver.borrow_mut().run_frame();
    });
}
```

#### Device Manager

```rust
pub struct DeviceManager {
    devices: Vec<Box<dyn VRDevice>>,
    driver_host: *mut c_void, // IVRServerDriverHost pointer
}

impl DeviceManager {
    pub fn new(driver_host: *mut c_void) -> Self {
        Self {
            devices: Vec::new(),
            driver_host,
        }
    }
    
    pub fn add_device(&mut self, device: Box<dyn VRDevice>) -> bool {
        // Register device with OpenVR
        let serial = device.get_serial();
        let device_class = device.get_device_class();
        
        // Call into OpenVR to register the device
        let success = unsafe {
            openvr_add_tracked_device(self.driver_host, serial.as_ptr(), device_class, device.as_ptr())
        };
        
        if success {
            self.devices.push(device);
        }
        
        success
    }
    
    pub fn update_devices(&mut self) {
        for device in &mut self.devices {
            device.update();
        }
    }
}
```

#### Tracking Provider

```rust
pub struct TrackingProvider {
    slam_interface: Arc<Mutex<dyn SLAMInterface>>,
}

impl TrackingProvider {
    pub fn new(slam_interface: Arc<Mutex<dyn SLAMInterface>>) -> Self {
        Self {
            slam_interface,
        }
    }
    
    pub fn get_hmd_pose(&self) -> Pose {
        // Get latest HMD pose from SLAM system
        let slam = self.slam_interface.lock().unwrap();
        let slam_pose = slam.get_latest_pose();
        
        // Convert to OpenVR pose format
        self.convert_to_openvr_pose(slam_pose)
    }
    
    pub fn get_controller_poses(&self) -> (Pose, Pose) {
        // Get latest controller poses
        // This would use a different mechanism, possibly from the Core API
        // For now, we'll derive them from the HMD pose
        let hmd_pose = self.get_hmd_pose();
        
        // Create left and right controller poses based on HMD
        let left_pose = self.derive_left_controller_pose(&hmd_pose);
        let right_pose = self.derive_right_controller_pose(&hmd_pose);
        
        (left_pose, right_pose)
    }
    
    fn convert_to_openvr_pose(&self, slam_pose: SLAMPose) -> Pose {
        // Convert from our SLAM coordinate system to OpenVR
        // This involves rotation and translation transformations
        // ...
    }
}
```

#### Input Handler

```rust
pub struct InputHandler {
    driver_input: *mut c_void, // IVRDriverInput pointer
    device_handles: HashMap<String, u32>, // Map of device serials to their input handles
}

impl InputHandler {
    pub fn new(driver_input: *mut c_void) -> Self {
        Self {
            driver_input,
            device_handles: HashMap::new(),
        }
    }
    
    pub fn register_device(&mut self, serial: &str, device_handle: u32) {
        self.device_handles.insert(serial.to_string(), device_handle);
    }
    
    pub fn update_button(&mut self, serial: &str, button: Button, state: ButtonState) {
        if let Some(handle) = self.device_handles.get(serial) {
            unsafe {
                openvr_update_boolean_component(
                    self.driver_input,
                    button.component_handle,
                    state.pressed,
                    0.0 // Seconds from now
                );
            }
        }
    }
    
    pub fn update_axis(&mut self, serial: &str, axis: Axis, x: f32, y: f32) {
        if let Some(handle) = self.device_handles.get(serial) {
            unsafe {
                openvr_update_scalar_component(
                    self.driver_input,
                    axis.x_handle,
                    x,
                    0.0 // Seconds from now
                );
                
                if axis.has_y {
                    openvr_update_scalar_component(
                        self.driver_input,
                        axis.y_handle,
                        y,
                        0.0 // Seconds from now
                    );
                }
            }
        }
    }
    
    pub fn trigger_haptic_pulse(&mut self, serial: &str, duration_micros: u16, frequency: u16, amplitude: f32) {
        if let Some(handle) = self.device_handles.get(serial) {
            unsafe {
                openvr_trigger_haptic_pulse(
                    self.driver_input,
                    *handle,
                    0, // Which haptic
                    duration_micros,
                    frequency,
                    amplitude
                );
            }
        }
    }
}
```

### 3. Device Implementations

#### HMD Device

```rust
pub struct HMDDevice {
    serial: String,
    device_index: AtomicU32,
    properties: Arc<Mutex<PropertyContainer>>,
    pose: Arc<Mutex<Pose>>,
}

impl VRDevice for HMDDevice {
    fn get_serial(&self) -> &str {
        &self.serial
    }
    
    fn get_device_class(&self) -> DeviceClass {
        DeviceClass::HMD
    }
    
    fn update(&mut self) {
        // Update HMD pose from tracking provider
        let tracking_provider = DRIVER.with(|driver| {
            driver.borrow().get_tracking_provider()
        });
        
        let new_pose = tracking_provider.get_hmd_pose();
        *self.pose.lock().unwrap() = new_pose;
    }
    
    fn get_pose(&self) -> Pose {
        self.pose.lock().unwrap().clone()
    }
    
    // Other VRDevice methods...
}

impl HMDDevice {
    pub fn new(serial: &str) -> Self {
        Self {
            serial: serial.to_string(),
            device_index: AtomicU32::new(vr_k_invalid_device_index),
            properties: Arc::new(Mutex::new(PropertyContainer::new())),
            pose: Arc::new(Mutex::new(Pose::default())),
        }
    }
    
    // HMD-specific methods...
}
```

#### Controller Device

```rust
pub struct ControllerDevice {
    serial: String,
    device_index: AtomicU32,
    properties: Arc<Mutex<PropertyContainer>>,
    pose: Arc<Mutex<Pose>>,
    handedness: Handedness,
    button_states: HashMap<Button, ButtonState>,
    axis_states: HashMap<Axis, (f32, f32)>,
}

impl VRDevice for ControllerDevice {
    // VRDevice implementation...
}

impl ControllerDevice {
    pub fn new(serial: &str, handedness: Handedness) -> Self {
        Self {
            serial: serial.to_string(),
            device_index: AtomicU32::new(vr_k_invalid_device_index),
            properties: Arc::new(Mutex::new(PropertyContainer::new())),
            pose: Arc::new(Mutex::new(Pose::default())),
            handedness,
            button_states: HashMap::new(),
            axis_states: HashMap::new(),
        }
    }
    
    pub fn update_button(&mut self, button: Button, state: ButtonState) {
        self.button_states.insert(button, state);
        
        // Update through input handler
        let input_handler = DRIVER.with(|driver| {
            driver.borrow().get_input_handler()
        });
        
        input_handler.update_button(&self.serial, button, state);
    }
    
    pub fn update_axis(&mut self, axis: Axis, x: f32, y: f32) {
        self.axis_states.insert(axis, (x, y));
        
        // Update through input handler
        let input_handler = DRIVER.with(|driver| {
            driver.borrow().get_input_handler()
        });
        
        input_handler.update_axis(&self.serial, axis, x, y);
    }
}
```

### 4. Core API Integration

#### SLAM Integration

```rust
pub struct SLAMIntegration {
    core_api: Arc<Mutex<dyn VRCoreAPI>>,
}

impl SLAMIntegration {
    pub fn new(core_api: Arc<Mutex<dyn VRCoreAPI>>) -> Self {
        Self {
            core_api,
        }
    }
    
    pub fn get_latest_slam_data(&self) -> SLAMData {
        let api = self.core_api.lock().unwrap();
        
        // Get latest camera frames and IMU data
        let frames = api.hardware().get_camera_frames();
        let imu_data = api.hardware().get_imu_data();
        
        // Process through SLAM pipeline
        let slam_result = api.slam().process_frames_and_imu(frames, imu_data);
        
        slam_result
    }
}
```

#### Configuration Integration

```rust
pub struct ConfigurationIntegration {
    core_api: Arc<Mutex<dyn VRCoreAPI>>,
}

impl ConfigurationIntegration {
    pub fn new(core_api: Arc<Mutex<dyn VRCoreAPI>>) -> Self {
        Self {
            core_api,
        }
    }
    
    pub fn get_driver_settings(&self) -> DriverSettings {
        let api = self.core_api.lock().unwrap();
        
        // Get driver-specific settings from Core API
        let config = api.config();
        
        DriverSettings {
            render_width: config.get_int("openvr.render_width").unwrap_or(1600),
            render_height: config.get_int("openvr.render_height").unwrap_or(1600),
            refresh_rate: config.get_float("openvr.refresh_rate").unwrap_or(90.0),
            ipd: config.get_float("openvr.ipd").unwrap_or(0.063), // 63mm default
            prediction_time_ms: config.get_float("openvr.prediction_time_ms").unwrap_or(30.0),
        }
    }
    
    pub fn save_driver_settings(&self, settings: &DriverSettings) {
        let mut api = self.core_api.lock().unwrap();
        let mut config = api.config_mut();
        
        config.set_int("openvr.render_width", settings.render_width);
        config.set_int("openvr.render_height", settings.render_height);
        config.set_float("openvr.refresh_rate", settings.refresh_rate);
        config.set_float("openvr.ipd", settings.ipd);
        config.set_float("openvr.prediction_time_ms", settings.prediction_time_ms);
        
        config.save();
    }
}
```

## Data Flow

### Initialization Flow

1. SteamVR loads our driver DLL and calls `HmdDriverFactory`
2. The C++ factory function creates and returns our `ServerDeviceProvider`
3. SteamVR calls `Init` on our provider, which initializes the Rust driver core
4. The Rust driver core initializes and creates device instances
5. Devices are registered with SteamVR through the OpenVR API

### Frame Update Flow

1. SteamVR calls `RunFrame` on our provider
2. The C++ provider calls into the Rust driver core's `run_frame` function
3. The Rust driver core:
   - Gets the latest tracking data from the SLAM system
   - Updates all device poses
   - Processes any input events
   - Sends updates to SteamVR

### Input Flow

1. Core API receives input from physical controllers
2. Input events are passed to the OpenVR driver through IPC
3. The driver updates controller state and notifies SteamVR
4. SteamVR passes input to applications

### Haptic Feedback Flow

1. SteamVR application triggers haptic feedback
2. SteamVR calls into our driver
3. Driver passes haptic commands to Core API through IPC
4. Core API triggers physical haptic actuators

## Build and Integration

### Build Process

1. Rust driver core is compiled as a static library
2. C++ interface layer is compiled and linked with the Rust library
3. The resulting DLL is packaged with the driver manifest and resources

### Integration with SteamVR

1. Driver is registered with SteamVR through the `vrpathreg` tool
2. SteamVR loads the driver on startup
3. Driver appears in SteamVR settings as a custom device

## Future Extensions

1. **Advanced Controller Emulation**: Support for more controller types and input profiles
2. **Room Setup Integration**: Custom room setup process integrated with our SLAM system
3. **Performance Optimization**: Specialized prediction and filtering for VR
4. **Multi-Device Support**: Support for multiple headsets and accessories

## Conclusion

This architecture provides a solid foundation for integrating our VR headset with SteamVR. By leveraging our existing Core API and implementing the necessary OpenVR interfaces, we can provide a seamless experience for users while maintaining the flexibility to evolve the implementation as requirements change.
