# ZeroCopyFrameProvider Documentation

## Overview

The `ZeroCopyFrameProvider` is a high-performance interface for acquiring camera frames and passing them directly to the TPU for feature extraction without unnecessary memory copies. It is designed to work with the V4L2 camera driver and the EdgeTPU hardware to minimize latency in the SLAM pipeline.

Key features:
1. Direct DMA buffer sharing between camera and TPU
2. Multi-camera synchronization
3. Frame timestamping and synchronization with IMU data
4. Efficient buffer management to avoid memory copies
5. Support for different camera configurations and formats

## Integration with ORB-SLAM3

The `ZeroCopyFrameProvider` is designed to be integrated with ORB-SLAM3 and the `TPUFeatureExtractor` to create a high-performance SLAM system for VR headsets. It replaces the standard OpenCV camera capture methods with a more efficient approach that minimizes memory copies and CPU overhead.

## API Reference

### Classes and Structures

#### `ZeroCopyFrameProvider`

Main class for zero-copy frame acquisition.

#### `ZeroCopyFrameProvider::CameraConfig`

Structure for camera configuration.

| Field | Type | Description |
|-------|------|-------------|
| `device_path` | `std::string` | Camera device path (e.g., `/dev/video0`) |
| `width` | `int` | Frame width in pixels |
| `height` | `int` | Frame height in pixels |
| `fps` | `int` | Frames per second |
| `pixel_format` | `std::string` | Pixel format (e.g., "GREY", "YUYV", "MJPG") |
| `zero_copy_enabled` | `bool` | Whether zero-copy is enabled for this camera |
| `buffer_count` | `int` | Number of buffers to allocate |
| `fx`, `fy` | `float` | Focal length in pixels |
| `cx`, `cy` | `float` | Principal point in pixels |
| `distortion_coeffs` | `std::vector<float>` | Distortion coefficients |
| `T_ref_cam` | `cv::Mat` | Transform from reference camera to this camera |

#### `ZeroCopyFrameProvider::FrameMetadata`

Structure for frame metadata.

| Field | Type | Description |
|-------|------|-------------|
| `frame_id` | `uint64_t` | Unique frame identifier |
| `timestamp` | `double` | Timestamp in seconds |
| `camera_id` | `int` | Camera identifier |
| `width` | `int` | Frame width in pixels |
| `height` | `int` | Frame height in pixels |
| `pixel_format` | `std::string` | Pixel format |
| `buffer_ptr` | `void*` | Pointer to the frame buffer |
| `buffer_size` | `size_t` | Size of the buffer in bytes |
| `dma_fd` | `int` | DMA file descriptor for zero-copy |
| `is_keyframe` | `bool` | Whether this frame is a keyframe |

### Constructor

```cpp
ZeroCopyFrameProvider(const std::vector<CameraConfig>& configs);
```

Creates a new `ZeroCopyFrameProvider` with the specified camera configurations.

### Initialization and Control

```cpp
bool Initialize();
```

Initializes the frame provider, opening cameras and allocating buffers.

```cpp
bool StartAcquisition();
```

Starts frame acquisition from all cameras.

```cpp
void StopAcquisition();
```

Stops frame acquisition from all cameras.

### Frame Acquisition

```cpp
bool GetNextFrame(int camera_id, FrameMetadata& metadata, int timeout_ms = -1);
```

Gets the next frame from the specified camera. The `timeout_ms` parameter specifies the timeout in milliseconds (0 for non-blocking, negative for infinite).

```cpp
bool GetNextSynchronizedFrames(std::vector<FrameMetadata>& metadata_vec, 
                              float max_time_diff_ms = 10.0f,
                              int timeout_ms = -1);
```

Gets the next synchronized frames from all cameras. The `max_time_diff_ms` parameter specifies the maximum time difference between frames in milliseconds.

```cpp
void ReleaseFrame(const FrameMetadata& metadata);
```

Releases a frame buffer, allowing it to be reused.

### Frame Processing

```cpp
cv::Mat GetMatForFrame(const FrameMetadata& metadata);
```

Gets a `cv::Mat` wrapper for a frame buffer. Note that this should be avoided for zero-copy operations, as it may introduce memory copies.

```cpp
int GetDmaFdForFrame(const FrameMetadata& metadata);
```

Gets the DMA file descriptor for a frame buffer, which can be used for zero-copy operations.

### Configuration and Status

```cpp
CameraConfig GetCameraConfig(int camera_id) const;
```

Gets the camera configuration for the specified camera.

```cpp
bool SetCameraConfig(int camera_id, const CameraConfig& config);
```

Sets a new camera configuration for the specified camera.

```cpp
bool IsCameraConnected(int camera_id) const;
```

Checks if a camera is connected.

```cpp
size_t GetCameraCount() const;
```

Gets the number of cameras.

```cpp
float GetCurrentFrameRate(int camera_id) const;
```

Gets the current frame rate for the specified camera.

### Callbacks and Zero-Copy

```cpp
void RegisterFrameCallback(std::function<void(const FrameMetadata&)> callback);
```

Registers a callback function that will be called when a new frame is available.

```cpp
bool EnableZeroCopy(int camera_id, bool enable);
```

Enables or disables zero-copy mode for the specified camera.

```cpp
bool IsZeroCopySupported(int camera_id) const;
```

Checks if zero-copy is supported for the specified camera.

### Error Handling

```cpp
std::string GetLastErrorMessage() const;
```

Gets the latest error message.

## Usage Examples

### Basic Usage

```cpp
// Create camera configuration
ZeroCopyFrameProvider::CameraConfig config;
config.device_path = "/dev/video0";
config.width = 640;
config.height = 480;
config.fps = 30;
config.pixel_format = "YUYV";
config.zero_copy_enabled = true;
config.buffer_count = 4;

// Set camera intrinsics
config.fx = 500.0f;
config.fy = 500.0f;
config.cx = 320.0f;
config.cy = 240.0f;
config.distortion_coeffs = {0.0f, 0.0f, 0.0f, 0.0f, 0.0f};

// Set camera extrinsics (identity transform for single camera)
config.T_ref_cam = cv::Mat::eye(4, 4, CV_32F);

// Create frame provider with single camera
std::vector<ZeroCopyFrameProvider::CameraConfig> configs = {config};
ZeroCopyFrameProvider provider(configs);

// Initialize provider
if (!provider.Initialize()) {
    std::cerr << "Failed to initialize frame provider: " << provider.GetLastErrorMessage() << std::endl;
    return 1;
}

// Start acquisition
if (!provider.StartAcquisition()) {
    std::cerr << "Failed to start acquisition: " << provider.GetLastErrorMessage() << std::endl;
    return 1;
}

// Acquisition loop
while (true) {
    // Get next frame
    ZeroCopyFrameProvider::FrameMetadata metadata;
    if (!provider.GetNextFrame(0, metadata, 100)) {
        std::cerr << "Failed to get frame: " << provider.GetLastErrorMessage() << std::endl;
        continue;
    }
    
    // Process frame
    // ...
    
    // Release frame
    provider.ReleaseFrame(metadata);
}

// Stop acquisition
provider.StopAcquisition();
```

### Multi-Camera Synchronization

```cpp
// Create camera configurations
std::vector<ZeroCopyFrameProvider::CameraConfig> configs;

// Camera 0 (reference camera)
ZeroCopyFrameProvider::CameraConfig config0;
config0.device_path = "/dev/video0";
config0.width = 640;
config0.height = 480;
config0.fps = 30;
config0.pixel_format = "YUYV";
config0.zero_copy_enabled = true;
config0.buffer_count = 4;
config0.T_ref_cam = cv::Mat::eye(4, 4, CV_32F);
configs.push_back(config0);

// Camera 1
ZeroCopyFrameProvider::CameraConfig config1;
config1.device_path = "/dev/video1";
config1.width = 640;
config1.height = 480;
config1.fps = 30;
config1.pixel_format = "YUYV";
config1.zero_copy_enabled = true;
config1.buffer_count = 4;
// Set transform from reference camera to this camera
config1.T_ref_cam = (cv::Mat_<float>(4, 4) << 
                    1.0f, 0.0f, 0.0f, 0.1f,
                    0.0f, 1.0f, 0.0f, 0.0f,
                    0.0f, 0.0f, 1.0f, 0.0f,
                    0.0f, 0.0f, 0.0f, 1.0f);
configs.push_back(config1);

// Create frame provider
ZeroCopyFrameProvider provider(configs);

// Initialize and start acquisition
provider.Initialize();
provider.StartAcquisition();

// Acquisition loop
while (true) {
    // Get synchronized frames
    std::vector<ZeroCopyFrameProvider::FrameMetadata> metadata_vec;
    if (!provider.GetNextSynchronizedFrames(metadata_vec, 10.0f, 100)) {
        std::cerr << "Failed to get synchronized frames" << std::endl;
        continue;
    }
    
    // Process frames
    for (const auto& metadata : metadata_vec) {
        // Process each frame
        // ...
        
        // Release frame
        provider.ReleaseFrame(metadata);
    }
}

// Stop acquisition
provider.StopAcquisition();
```

### Zero-Copy Integration with TPUFeatureExtractor

```cpp
// Create and initialize frame provider
ZeroCopyFrameProvider provider(configs);
provider.Initialize();

// Create TPU feature extractor
TPUFeatureExtractor extractor("models/superpoint_edgetpu.tflite", "", 1000, 1.2f, 8);

// Start acquisition
provider.StartAcquisition();

// Acquisition loop
while (true) {
    // Get next frame
    ZeroCopyFrameProvider::FrameMetadata metadata;
    if (!provider.GetNextFrame(0, metadata, 100)) {
        continue;
    }
    
    // Get DMA file descriptor for zero-copy
    int dma_fd = provider.GetDmaFdForFrame(metadata);
    
    if (dma_fd >= 0) {
        // Use DMA file descriptor for zero-copy inference
        // This would be implemented in TPUFeatureExtractor
        // extractor.RunInferenceZeroCopy(dma_fd, metadata.width, metadata.height);
    } else {
        // Fallback to regular inference
        cv::Mat frame = provider.GetMatForFrame(metadata);
        std::vector<cv::KeyPoint> keypoints;
        cv::Mat descriptors;
        extractor(frame, cv::Mat(), keypoints, descriptors);
    }
    
    // Release frame
    provider.ReleaseFrame(metadata);
}

// Stop acquisition
provider.StopAcquisition();
```

## Performance Considerations

### Zero-Copy Mode

Zero-copy mode is the most efficient way to transfer frames from the camera to the TPU, as it avoids memory copies. However, it requires hardware support and may not be available on all platforms. The `IsZeroCopySupported` method can be used to check if zero-copy is supported for a specific camera.

### Buffer Management

The `ZeroCopyFrameProvider` uses a pool of buffers to avoid allocating memory for each frame. The number of buffers can be configured using the `buffer_count` parameter in the `CameraConfig` structure. A higher buffer count can improve performance by reducing the likelihood of buffer starvation, but it also increases memory usage.

### Frame Rate

The `ZeroCopyFrameProvider` attempts to maintain the requested frame rate, but the actual frame rate may be lower due to hardware limitations or processing overhead. The `GetCurrentFrameRate` method can be used to monitor the actual frame rate.

### Multi-Camera Synchronization

When using multiple cameras, the `GetNextSynchronizedFrames` method can be used to get frames that are captured at approximately the same time. The `max_time_diff_ms` parameter specifies the maximum allowed time difference between frames. If the time difference exceeds this value, the method will discard frames until it finds a synchronized set.

## Troubleshooting

### Camera Not Found

If a camera is not found, check the device path and ensure that the camera is connected and recognized by the system. The `v4l2-ctl --list-devices` command can be used to list available cameras.

### Unsupported Format

If a pixel format is not supported, try using a different format. The `v4l2-ctl --list-formats` command can be used to list supported formats for a specific camera.

### Low Frame Rate

If the frame rate is lower than expected, check if the camera supports the requested frame rate and resolution. Reducing the resolution or frame rate may improve performance.

### Memory Usage

If memory usage is a concern, reduce the `buffer_count` parameter in the `CameraConfig` structure. However, this may reduce performance by increasing the likelihood of buffer starvation.

### CPU Usage

If CPU usage is a concern, enable zero-copy mode if supported. This reduces CPU overhead by avoiding memory copies.

## Conclusion

The `ZeroCopyFrameProvider` is a high-performance interface for acquiring camera frames and passing them directly to the TPU for feature extraction. It is designed to minimize latency and CPU overhead, making it ideal for real-time SLAM applications such as VR headsets.
