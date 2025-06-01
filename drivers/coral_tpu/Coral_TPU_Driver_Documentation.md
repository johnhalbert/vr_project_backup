# Coral TPU Driver Integration Documentation

## Overview

This document provides comprehensive documentation for the Coral TPU driver integration developed for the VR SLAM system. The integration enables efficient, low-latency neural network inference on the Edge TPU hardware accelerator, with a focus on feature extraction for visual SLAM applications.

## Key Features

### 1. Zero-Copy Buffer Management

The driver implements a zero-copy buffer management system that minimizes memory operations:

- **Direct DMA Buffer Sharing**: Enables direct sharing of buffers between camera, CPU, and TPU
- **Buffer Import/Export**: Supports importing external DMA buffers for zero-copy operations
- **Buffer Pooling**: Pre-allocates and reuses buffers to avoid allocation overhead
- **Memory Mapping**: Provides efficient CPU access to TPU buffers when needed

```c++
struct tpu_buffer {
    void* host_ptr;                 /* Host virtual address */
    uint64_t device_addr;           /* Device physical address */
    int fd;                         /* DMA buffer file descriptor */
    size_t size;                    /* Buffer size in bytes */
    uint32_t flags;                 /* Buffer flags */
    bool in_use;                    /* Whether buffer is currently in use */
};
```

### 2. Optimized Inference Scheduling

The inference scheduler prioritizes and manages inference tasks for optimal performance:

- **Priority-Based Scheduling**: Processes high-priority tasks first
- **Asynchronous Execution**: Supports non-blocking inference operations
- **Task Batching**: Combines compatible tasks for improved throughput
- **Callback Mechanism**: Notifies completion through user-defined callbacks

```c++
enum tpu_task_priority {
    TPU_PRIORITY_LOW,               /* Background tasks */
    TPU_PRIORITY_NORMAL,            /* Normal inference tasks */
    TPU_PRIORITY_HIGH,              /* Time-sensitive tasks */
    TPU_PRIORITY_CRITICAL           /* Critical real-time tasks */
};

struct tpu_inference_task {
    uint32_t task_id;               /* Unique task identifier */
    tpu_task_priority priority;     /* Task priority */
    tpu_buffer* input_buffer;       /* Input buffer */
    tpu_buffer* output_buffer;      /* Output buffer */
    uint32_t model_id;              /* Model identifier */
    std::function<void(uint32_t)> callback; /* Completion callback */
    uint64_t submit_time;           /* Task submission timestamp */
    uint64_t start_time;            /* Task start timestamp */
    uint64_t complete_time;         /* Task completion timestamp */
};
```

### 3. Efficient Model Management

The model manager handles loading, unloading, and caching of neural network models:

- **Model Caching**: Caches frequently used models to avoid reloading
- **Versioning**: Tracks model versions and updates
- **Memory Optimization**: Efficiently manages TPU memory for multiple models
- **Model Sharing**: Allows multiple tasks to share the same model

```c++
struct tpu_model {
    uint32_t model_id;              /* Unique model identifier */
    std::string model_path;         /* Path to model file */
    size_t model_size;              /* Model size in bytes */
    void* model_data;               /* Model data in memory */
    uint64_t last_used;             /* Last usage timestamp */
    uint32_t reference_count;       /* Number of active references */
    bool loaded;                    /* Whether model is loaded on TPU */
};
```

### 4. Comprehensive Performance Monitoring

The performance monitoring system tracks various metrics to ensure optimal performance:

- **Latency Tracking**: Measures inference and buffer transfer latency
- **Throughput Monitoring**: Tracks inference throughput and data transfer rates
- **Utilization Metrics**: Monitors TPU and memory utilization
- **Thermal Monitoring**: Tracks TPU temperature and thermal status
- **Power Monitoring**: Monitors power consumption and state

```c++
struct tpu_performance_metrics {
    /* Latency metrics */
    uint32_t avg_inference_latency_us;  /* Average inference latency (us) */
    uint32_t min_inference_latency_us;  /* Minimum inference latency (us) */
    uint32_t max_inference_latency_us;  /* Maximum inference latency (us) */
    uint32_t avg_buffer_transfer_us;    /* Average buffer transfer time (us) */
    
    /* Throughput metrics */
    uint32_t inferences_per_second;     /* Inferences per second */
    uint32_t bytes_per_second;          /* Data throughput (bytes/s) */
    
    /* Utilization metrics */
    uint8_t tpu_utilization_percent;    /* TPU utilization (0-100%) */
    uint8_t memory_utilization_percent; /* TPU memory utilization (0-100%) */
    
    /* Thermal metrics */
    uint8_t temperature_celsius;        /* TPU temperature (°C) */
    
    /* Power metrics */
    uint8_t power_state;                /* Current power state */
    uint32_t power_consumption_mw;      /* Estimated power consumption (mW) */
    
    /* Error metrics */
    uint32_t error_count;               /* Number of errors */
    uint32_t recovery_count;            /* Number of recoveries */
    
    /* Timestamp */
    uint64_t timestamp;                 /* Timestamp (microseconds) */
};
```

### 5. Advanced Power Management

The power management system optimizes power consumption based on workload:

- **Multiple Power States**: Supports different power states for various scenarios
- **Dynamic Scaling**: Adjusts performance based on workload
- **Idle Detection**: Transitions to low-power states during idle periods
- **Thermal Coordination**: Coordinates power management with thermal status

```c++
enum tpu_power_state {
    TPU_POWER_OFF,                  /* TPU powered off */
    TPU_POWER_LOW,                  /* Low-power state */
    TPU_POWER_NORMAL,               /* Normal operating state */
    TPU_POWER_HIGH                  /* High-performance state */
};

struct tpu_power_config {
    tpu_power_state default_state;  /* Default power state */
    bool dynamic_scaling;           /* Enable dynamic power scaling */
    uint32_t idle_timeout_ms;       /* Idle timeout before power down (ms) */
    uint8_t performance_target;     /* Performance target (0-100%) */
};
```

### 6. Robust Error Handling

The error handling system detects and recovers from errors:

- **Error Detection**: Detects hardware and software errors
- **Error Classification**: Classifies errors by type and severity
- **Recovery Strategies**: Implements recovery for different error types
- **Error Reporting**: Provides detailed error information

```c++
enum tpu_error_type {
    TPU_ERROR_NONE,                 /* No error */
    TPU_ERROR_HARDWARE,             /* Hardware error */
    TPU_ERROR_COMMUNICATION,        /* Communication error */
    TPU_ERROR_TIMEOUT,              /* Operation timeout */
    TPU_ERROR_INVALID_MODEL,        /* Invalid model */
    TPU_ERROR_OUT_OF_MEMORY,        /* Out of memory */
    TPU_ERROR_THERMAL,              /* Thermal error */
    TPU_ERROR_UNKNOWN               /* Unknown error */
};

struct tpu_error_info {
    tpu_error_type type;            /* Error type */
    uint32_t code;                  /* Error code */
    std::string message;            /* Error message */
    uint64_t timestamp;             /* Error timestamp */
    bool recovered;                 /* Whether error was recovered */
};
```

## Integration with VR SLAM System

The Coral TPU driver integration is designed to work seamlessly with the VR SLAM system:

### Integration with TPUFeatureExtractor

```c++
// Example integration with TPUFeatureExtractor
class TPUFeatureExtractor {
public:
    TPUFeatureExtractor(std::shared_ptr<EdgeTpuDriver> driver) 
        : driver_(driver) {
        // Load the feature extraction model
        model_id_ = driver_->LoadModel(model_path_);
        
        // Create buffer pools for input and output
        input_pool_ = driver_->CreateBufferPool(input_size_, pool_size_);
        output_pool_ = driver_->CreateBufferPool(output_size_, pool_size_);
    }
    
    ~TPUFeatureExtractor() {
        // Clean up resources
        driver_->UnloadModel(model_id_);
        driver_->DestroyBufferPool(input_pool_);
        driver_->DestroyBufferPool(output_pool_);
    }
    
    void ExtractFeaturesAsync(const cv::Mat& image, 
                             std::function<void(const std::vector<Feature>&)> callback) {
        // Get buffers from pools
        tpu_buffer* input_buffer = driver_->GetBufferFromPool(input_pool_);
        tpu_buffer* output_buffer = driver_->GetBufferFromPool(output_pool_);
        
        // Copy image data to input buffer
        // In a real implementation, this would be zero-copy from camera
        memcpy(input_buffer->host_ptr, image.data, image.total() * image.elemSize());
        
        // Create inference task
        tpu_inference_task* task = driver_->CreateInferenceTask(
            model_id_, input_buffer, output_buffer, TPU_PRIORITY_HIGH);
        
        // Set callback
        task->callback = [this, input_buffer, output_buffer, callback](uint32_t task_id) {
            // Process output data
            std::vector<Feature> features = ProcessOutputBuffer(output_buffer);
            
            // Return buffers to pools
            driver_->ReturnBufferToPool(input_pool_, input_buffer);
            driver_->ReturnBufferToPool(output_pool_, output_buffer);
            
            // Call user callback with results
            callback(features);
        };
        
        // Schedule inference
        driver_->ScheduleInference(task);
    }
    
private:
    std::shared_ptr<EdgeTpuDriver> driver_;
    uint32_t model_id_;
    tpu_buffer_pool* input_pool_;
    tpu_buffer_pool* output_pool_;
    std::string model_path_ = "/path/to/feature_extraction_model.tflite";
    size_t input_size_ = 300 * 300 * 3;  // Example input size
    size_t output_size_ = 1000 * 128;    // Example output size
    size_t pool_size_ = 5;               // Number of buffers in pool
    
    std::vector<Feature> ProcessOutputBuffer(tpu_buffer* buffer) {
        // Process the output buffer to extract features
        // This is application-specific
        std::vector<Feature> features;
        // ... processing code ...
        return features;
    }
};
```

### Integration with ZeroCopyFrameProvider

```c++
// Example integration with ZeroCopyFrameProvider
class ZeroCopyFrameProvider {
public:
    ZeroCopyFrameProvider(std::shared_ptr<EdgeTpuDriver> driver) 
        : driver_(driver) {
        // Initialize camera and DMA buffers
        // ... initialization code ...
    }
    
    tpu_buffer* GetFrame() {
        // Get a frame from the camera as a DMA buffer
        int dma_fd = camera_->GetFrameDmaFd();
        void* dma_ptr = camera_->GetFramePtr();
        size_t size = camera_->GetFrameSize();
        
        // Import the DMA buffer for TPU use
        return driver_->ImportBuffer(dma_fd, dma_ptr, size);
    }
    
    void ReleaseFrame(tpu_buffer* buffer) {
        // Release the buffer but don't free the underlying DMA buffer
        driver_->ReleaseBuffer(buffer);
        
        // Return the frame to the camera
        camera_->ReturnFrame();
    }
    
private:
    std::shared_ptr<EdgeTpuDriver> driver_;
    std::unique_ptr<Camera> camera_;
};
```

## Performance Optimization

### Latency Optimization

The driver implements several techniques to minimize inference latency:

1. **Zero-Copy Data Path**: Eliminates unnecessary memory copies
2. **Asynchronous Processing**: Overlaps data transfer and computation
3. **Priority-Based Scheduling**: Ensures critical tasks are processed first
4. **Pre-allocated Buffers**: Avoids allocation overhead during inference
5. **Optimized Model Loading**: Caches models to avoid reloading

### Throughput Optimization

The driver maximizes inference throughput through:

1. **Task Batching**: Combines compatible tasks when possible
2. **Pipelined Processing**: Overlaps different stages of inference
3. **Parallel Execution**: Utilizes multiple TPUs when available
4. **Efficient Memory Management**: Minimizes memory-related stalls
5. **Optimized Data Transfers**: Minimizes transfer overhead

### Memory Optimization

The driver optimizes memory usage through:

1. **Buffer Pooling**: Reuses buffers to avoid allocation/deallocation
2. **Model Caching**: Keeps frequently used models in memory
3. **Memory Mapping**: Provides efficient access to shared buffers
4. **Zero-Copy Transfers**: Eliminates duplicate copies of data
5. **Memory Compression**: Uses compressed formats when beneficial

## API Reference

### Buffer Management API

```c++
// Allocate a buffer for TPU use
tpu_buffer* AllocateBuffer(size_t size);

// Release a buffer
void ReleaseBuffer(tpu_buffer* buffer);

// Import an external DMA buffer
tpu_buffer* ImportBuffer(int dma_fd, void* host_ptr, size_t size);

// Create a buffer pool
tpu_buffer_pool* CreateBufferPool(size_t buffer_size, size_t pool_size);

// Destroy a buffer pool
void DestroyBufferPool(tpu_buffer_pool* pool);

// Get a buffer from a pool
tpu_buffer* GetBufferFromPool(tpu_buffer_pool* pool);

// Return a buffer to a pool
void ReturnBufferToPool(tpu_buffer_pool* pool, tpu_buffer* buffer);
```

### Model Management API

```c++
// Load a model
uint32_t LoadModel(const std::string& model_path);

// Unload a model
void UnloadModel(uint32_t model_id);

// Check if a model is loaded
bool IsModelLoaded(uint32_t model_id);

// Get model size
size_t GetModelSize(uint32_t model_id);
```

### Inference API

```c++
// Create an inference task
tpu_inference_task* CreateInferenceTask(uint32_t model_id, 
                                      tpu_buffer* input_buffer,
                                      tpu_buffer* output_buffer,
                                      tpu_task_priority priority);

// Destroy an inference task
void DestroyInferenceTask(tpu_inference_task* task);

// Schedule an inference task
uint32_t ScheduleInference(tpu_inference_task* task);

// Cancel an inference task
void CancelInference(uint32_t task_id);
```

### Performance Monitoring API

```c++
// Get performance metrics
tpu_performance_metrics GetPerformanceMetrics();

// Reset performance metrics
void ResetPerformanceMetrics();
```

### Power Management API

```c++
// Set power state
void SetPowerState(tpu_power_state state);

// Get power state
tpu_power_state GetPowerState();

// Set power configuration
void SetPowerConfig(const tpu_power_config& config);

// Get power configuration
tpu_power_config GetPowerConfig();
```

### Thermal Management API

```c++
// Get temperature
uint8_t GetTemperature();

// Set thermal configuration
void SetThermalConfig(const tpu_thermal_config& config);

// Get thermal configuration
tpu_thermal_config GetThermalConfig();
```

### Error Handling API

```c++
// Get last error
tpu_error_info GetLastError();

// Clear errors
void ClearErrors();
```

## Configuration

### Driver Configuration

The driver can be configured through a configuration file or API:

```c++
struct tpu_driver_config {
    // Buffer management configuration
    size_t default_buffer_size;     /* Default buffer size */
    size_t max_buffer_size;         /* Maximum buffer size */
    size_t buffer_alignment;        /* Buffer alignment */
    size_t max_buffers;             /* Maximum number of buffers */
    
    // Model management configuration
    size_t model_cache_size;        /* Model cache size */
    std::string model_cache_dir;    /* Model cache directory */
    
    // Inference configuration
    size_t max_batch_size;          /* Maximum batch size */
    size_t max_concurrent_inferences; /* Maximum concurrent inferences */
    
    // Power management configuration
    tpu_power_config power_config;  /* Power configuration */
    
    // Thermal management configuration
    tpu_thermal_config thermal_config; /* Thermal configuration */
};
```

### Sysfs Interface

The driver exposes configuration and monitoring interfaces through sysfs:

```
/sys/class/edgetpu/
├── device/
│   ├── status                  # Device status (online/offline)
│   ├── model                   # Device model information
│   └── version                 # Firmware version
├── performance/
│   ├── utilization             # TPU utilization (0-100%)
│   ├── temperature             # TPU temperature (°C)
│   ├── power_state             # Current power state
│   └── metrics                 # Detailed performance metrics (JSON)
├── config/
│   ├── power_mode              # Power mode (0-3)
│   ├── performance_target      # Performance target (0-100%)
│   └── scheduler_policy        # Scheduler policy (0-2)
└── models/
    ├── loaded                  # Currently loaded models
    ├── cache_size              # Model cache size
    └── cache_policy            # Cache policy (0-2)
```

## Testing and Validation

### Unit Tests

The driver includes a comprehensive unit test suite that validates:

- Buffer management functionality
- Model loading and unloading
- Inference scheduling and execution
- Performance monitoring
- Power and thermal management
- Error handling and recovery

### Integration Tests

Integration tests validate the driver's interaction with:

- Edge TPU hardware (when available)
- TPUFeatureExtractor component
- ZeroCopyFrameProvider component
- VR SLAM system as a whole

### Performance Tests

Performance tests measure key metrics:

- Inference latency (average, minimum, maximum)
- Buffer transfer time
- Model loading time
- Throughput (inferences per second)
- Power consumption
- Thermal behavior

## Current Validation Status

The driver has been validated through software simulation and unit testing. The following aspects have been validated:

- ✓ API design and implementation
- ✓ Buffer management logic
- ✓ Inference scheduling algorithms
- ✓ Model management
- ✓ Error handling

The following aspects require hardware validation:

- ⚠ Actual inference performance
- ⚠ DMA buffer sharing
- ⚠ Power consumption
- ⚠ Thermal behavior
- ⚠ Integration with physical Edge TPU hardware

## Known Limitations

1. **Hardware Validation**: The driver has not been validated on actual Edge TPU hardware.
2. **USB vs. PCIe**: Performance characteristics may differ between USB and PCIe Edge TPU variants.
3. **Firmware Compatibility**: The driver assumes compatibility with the latest Edge TPU firmware.
4. **Multi-TPU Support**: Support for multiple TPUs is implemented but not fully tested.
5. **Power Measurement**: Actual power consumption may vary based on hardware implementation.

## Future Enhancements

1. **Advanced Scheduling**: Implement more sophisticated scheduling algorithms based on task dependencies and priorities.
2. **Dynamic Model Optimization**: Automatically optimize models based on performance requirements and hardware capabilities.
3. **Distributed Inference**: Support distributed inference across multiple TPUs and other accelerators.
4. **Hardware-Specific Optimizations**: Implement optimizations for specific Edge TPU hardware variants.
5. **Advanced Power Management**: Implement more sophisticated power management based on workload prediction.

## Conclusion

The Coral TPU driver integration provides a comprehensive solution for efficient, low-latency neural network inference in the VR SLAM system. By implementing zero-copy buffer sharing, optimized inference scheduling, and sophisticated performance monitoring, the driver enables optimal utilization of the Edge TPU hardware accelerator for feature extraction and other inference tasks.

The driver is designed to integrate seamlessly with the existing VR SLAM components, particularly the TPUFeatureExtractor and ZeroCopyFrameProvider, creating an efficient pipeline from camera acquisition to feature extraction and tracking.
