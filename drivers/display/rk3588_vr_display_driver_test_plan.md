# RK3588 Display Driver Test Plan

## Overview

This document outlines the testing strategy for the RK3588 display driver for the Linux kernel, with specific focus on VR-optimized features. The test plan covers unit testing, integration testing, simulation testing, and performance benchmarking to ensure the driver meets all requirements for VR display applications.

## Test Categories

### 1. Unit Tests

Unit tests will verify individual components of the driver in isolation:

#### Core Driver Tests
- **Initialization Test**: Verify proper initialization sequence and error handling
- **Mode Configuration Test**: Verify configuration of different display modes
- **Register Access Test**: Verify read/write operations to display controller registers
- **Property Interface Test**: Verify DRM property implementation
- **CRTC Configuration Test**: Verify CRTC setup for dual displays
- **Power Management Test**: Verify power state transitions

#### Display Controller Tests
- **VOPDC Configuration Test**: Verify VOPDC controller configuration
- **Layer Management Test**: Verify layer setup and configuration
- **Scaling Test**: Verify hardware scaling operations
- **Color Space Test**: Verify color space conversion
- **Blending Test**: Verify alpha blending operations

#### DRM/KMS Subsystem Tests
- **Device Registration Test**: Verify proper registration with DRM subsystem
- **Mode Setting Test**: Verify mode setting operations
- **Page Flip Test**: Verify atomic page flip operations
- **Property Handling Test**: Verify DRM property handling
- **Event Handling Test**: Verify DRM event generation and handling

### 2. Integration Tests

Integration tests will verify the driver's interaction with the kernel and hardware:

- **Kernel Integration Test**: Verify proper loading and initialization in the kernel
- **Device Tree Test**: Verify parsing of device tree bindings
- **DRM Framework Test**: Verify integration with the DRM subsystem
- **GPU Integration Test**: Verify integration with GPU driver
- **Multi-Display Test**: Verify operation with dual displays
- **Compositor Integration Test**: Verify integration with compositors
- **Buffer Sharing Test**: Verify DMA buffer sharing with other subsystems

### 3. Simulation Tests

Simulation tests will use virtual hardware to test driver behavior:

- **Virtual Display Test**: Use virtual display devices to test driver functionality
- **Register Simulation Test**: Simulate register access to display controller
- **Timing Simulation Test**: Simulate display timing scenarios
- **Error Condition Test**: Simulate various error conditions and verify recovery
- **Frame Generation Test**: Generate synthetic frames for testing
- **Vsync Simulation**: Simulate vsync timing and synchronization

### 4. Performance Tests

Performance tests will measure critical metrics for VR applications:

- **Latency Test**: Measure motion-to-photon latency
- **Vsync Accuracy Test**: Measure vsync timing accuracy
- **Frame Timing Test**: Measure frame timing stability
- **Synchronization Test**: Measure synchronization between displays
- **CPU Utilization Test**: Measure CPU overhead during display operations
- **Memory Bandwidth Test**: Measure memory bandwidth usage
- **Power Consumption Test**: Estimate power usage in different operation modes

## Test Implementation

### Unit Test Framework

Unit tests will be implemented using the Linux Kernel Test Framework (KTF):

```c
#include <ktf.h>
#include "rockchip_drm_vr.h"

KTF_INIT();

TEST(rk3588_vr_init_test)
{
    struct drm_device *dev;
    struct rockchip_vr_private *private;
    int ret;
    
    // Mock DRM device
    dev = mock_drm_device_create();
    
    // Test initialization
    ret = rockchip_vr_init(dev);
    EXPECT_INT_EQ(ret, 0);
    
    // Verify device state
    private = dev->dev_private;
    EXPECT_NOT_NULL(private);
    EXPECT_INT_EQ(private->vr_mode_enabled, 0);
    
    mock_drm_device_destroy(dev);
}

// Additional test cases...
```

### Integration Test Approach

Integration tests will use the Linux Test Project (LTP) framework:

```c
#include "test.h"
#include <fcntl.h>
#include <unistd.h>
#include <xf86drm.h>
#include <xf86drmMode.h>

void setup(void)
{
    // Load driver module
    system("modprobe rockchip_drm");
    
    // Wait for device initialization
    usleep(500000);
}

void cleanup(void)
{
    // Unload driver module
    system("modprobe -r rockchip_drm");
}

void test_device_probe(void)
{
    // Check if device was properly probed
    int ret = access("/dev/dri/card0", F_OK);
    TEST_ASSERT(ret == 0, "DRM device node not created");
    
    // Open device
    int fd = open("/dev/dri/card0", O_RDWR);
    TEST_ASSERT(fd >= 0, "Failed to open DRM device");
    
    // Get device resources
    drmModeResPtr res = drmModeGetResources(fd);
    TEST_ASSERT(res != NULL, "Failed to get DRM resources");
    
    // Verify device resources
    TEST_ASSERT(res->count_crtcs >= 2, "Insufficient CRTCs for dual display");
    TEST_ASSERT(res->count_connectors >= 2, "Insufficient connectors for dual display");
    
    drmModeFreeResources(res);
    close(fd);
}

// Additional test cases...
```

### Simulation Test Implementation

Simulation tests will use a custom virtual display controller:

```c
#include <linux/module.h>
#include <drm/drm_device.h>
#include <drm/drm_crtc.h>

// Virtual display controller implementation
static struct virtual_display_controller {
    uint32_t registers[1024];
    struct drm_crtc *crtcs[2];
    bool enabled[2];
    uint32_t width[2];
    uint32_t height[2];
    uint32_t refresh_rate[2];
    bool vsync_active[2];
    ktime_t last_vsync[2];
} vdc;

// Register read callback
static uint32_t virtual_display_read_reg(uint32_t offset)
{
    if (offset >= ARRAY_SIZE(vdc.registers))
        return 0;
        
    return vdc.registers[offset];
}

// Register write callback
static void virtual_display_write_reg(uint32_t offset, uint32_t value)
{
    if (offset >= ARRAY_SIZE(vdc.registers))
        return;
        
    vdc.registers[offset] = value;
    
    // Handle special registers
    if (offset == VOPDC_SYS_CTRL) {
        // Handle enable/disable
        vdc.enabled[0] = (value & VOPDC_SYS_CTRL_ENABLE0) != 0;
        vdc.enabled[1] = (value & VOPDC_SYS_CTRL_ENABLE1) != 0;
    } else if (offset == VOPDC_VR_CTRL) {
        // Handle VR mode settings
    }
}

// Vsync simulation function
static void virtual_display_vsync_timer_func(struct timer_list *t)
{
    int crtc_index = (int)(unsigned long)t->data;
    
    if (!vdc.enabled[crtc_index])
        return;
    
    // Calculate vsync interval based on refresh rate
    unsigned long interval_ns = 1000000000UL / vdc.refresh_rate[crtc_index];
    
    // Generate vsync event
    vdc.vsync_active[crtc_index] = true;
    vdc.last_vsync[crtc_index] = ktime_get();
    
    // Notify DRM subsystem
    if (vdc.crtcs[crtc_index])
        drm_crtc_handle_vblank(vdc.crtcs[crtc_index]);
    
    // Clear vsync after a short delay
    usleep_range(100, 200);
    vdc.vsync_active[crtc_index] = false;
    
    // Reschedule timer
    mod_timer(&vdc.vsync_timer[crtc_index], 
              jiffies + nsecs_to_jiffies(interval_ns));
}

// Additional simulation functions...
```

### Performance Test Implementation

Performance tests will use a combination of kernel tracing and userspace tools:

```c
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <time.h>
#include <xf86drm.h>
#include <xf86drmMode.h>
#include <gbm.h>
#include <EGL/egl.h>
#include <GLES2/gl2.h>

#define FRAMES 300

void test_vsync_accuracy(void)
{
    int fd, ret;
    drmModeResPtr res;
    drmModeCrtcPtr crtc1, crtc2;
    drmVBlank vbl1, vbl2;
    struct timespec start, end;
    uint64_t sequence1_start, sequence2_start;
    uint64_t sequence1_end, sequence2_end;
    double elapsed;
    double sync_error_sum = 0.0;
    double sync_error_max = 0.0;
    int sync_measurements = 0;
    
    // Open DRM device
    fd = open("/dev/dri/card0", O_RDWR);
    if (fd < 0) {
        perror("Failed to open DRM device");
        return;
    }
    
    // Get resources
    res = drmModeGetResources(fd);
    if (!res) {
        perror("Failed to get DRM resources");
        close(fd);
        return;
    }
    
    // Get CRTCs for both displays
    crtc1 = drmModeGetCrtc(fd, res->crtcs[0]);
    crtc2 = drmModeGetCrtc(fd, res->crtcs[1]);
    if (!crtc1 || !crtc2) {
        perror("Failed to get CRTCs");
        drmModeFreeResources(res);
        close(fd);
        return;
    }
    
    // Enable VR mode and synchronization
    // ... (code to set VR mode properties) ...
    
    // Start measurement
    clock_gettime(CLOCK_MONOTONIC, &start);
    
    // Get initial vblank counters
    memset(&vbl1, 0, sizeof(vbl1));
    vbl1.request.type = DRM_VBLANK_RELATIVE;
    vbl1.request.sequence = 0;
    ret = drmWaitVBlank(fd, &vbl1);
    sequence1_start = vbl1.reply.sequence;
    
    memset(&vbl2, 0, sizeof(vbl2));
    vbl2.request.type = DRM_VBLANK_RELATIVE | DRM_VBLANK_SECONDARY;
    vbl2.request.sequence = 0;
    ret = drmWaitVBlank(fd, &vbl2);
    sequence2_start = vbl2.reply.sequence;
    
    // Measure vsync timing for multiple frames
    for (int i = 0; i < FRAMES; i++) {
        // Wait for vblank on first display
        memset(&vbl1, 0, sizeof(vbl1));
        vbl1.request.type = DRM_VBLANK_RELATIVE;
        vbl1.request.sequence = 1;
        ret = drmWaitVBlank(fd, &vbl1);
        
        // Immediately check vblank counter on second display
        memset(&vbl2, 0, sizeof(vbl2));
        vbl2.request.type = DRM_VBLANK_RELATIVE | DRM_VBLANK_SECONDARY;
        vbl2.request.sequence = 0;
        ret = drmWaitVBlank(fd, &vbl2);
        
        // Calculate synchronization error
        int64_t sequence_diff = (int64_t)vbl1.reply.sequence - (int64_t)vbl2.reply.sequence;
        uint64_t tv_diff_us = 
            (vbl1.reply.tval_sec - vbl2.reply.tval_sec) * 1000000 +
            (vbl1.reply.tval_usec - vbl2.reply.tval_usec);
            
        double sync_error_ms = tv_diff_us / 1000.0;
        sync_error_sum += fabs(sync_error_ms);
        if (fabs(sync_error_ms) > sync_error_max)
            sync_error_max = fabs(sync_error_ms);
            
        sync_measurements++;
    }
    
    // Get final vblank counters
    memset(&vbl1, 0, sizeof(vbl1));
    vbl1.request.type = DRM_VBLANK_RELATIVE;
    vbl1.request.sequence = 0;
    ret = drmWaitVBlank(fd, &vbl1);
    sequence1_end = vbl1.reply.sequence;
    
    memset(&vbl2, 0, sizeof(vbl2));
    vbl2.request.type = DRM_VBLANK_RELATIVE | DRM_VBLANK_SECONDARY;
    vbl2.request.sequence = 0;
    ret = drmWaitVBlank(fd, &vbl2);
    sequence2_end = vbl2.reply.sequence;
    
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    // Calculate results
    elapsed = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1000000000.0;
    uint64_t frames1 = sequence1_end - sequence1_start;
    uint64_t frames2 = sequence2_end - sequence2_start;
    
    printf("Display 1: %lu frames in %.2f seconds (%.2f fps)\n", 
           frames1, elapsed, frames1 / elapsed);
    printf("Display 2: %lu frames in %.2f seconds (%.2f fps)\n", 
           frames2, elapsed, frames2 / elapsed);
    printf("Average sync error: %.3f ms\n", sync_error_sum / sync_measurements);
    printf("Maximum sync error: %.3f ms\n", sync_error_max);
    
    // Clean up
    drmModeFreeCrtc(crtc1);
    drmModeFreeCrtc(crtc2);
    drmModeFreeResources(res);
    close(fd);
}

// Additional performance tests...
```

## Test Execution Plan

### Phase 1: Unit Testing

1. Implement mock display controller interfaces
2. Develop unit tests for core driver functionality
3. Develop unit tests for DRM/KMS integration
4. Execute unit tests and fix any issues

### Phase 2: Integration Testing

1. Set up test environment with kernel build infrastructure
2. Implement integration tests for kernel interaction
3. Implement integration tests for DRM subsystem
4. Execute integration tests and fix any issues

### Phase 3: Simulation Testing

1. Develop virtual display controller implementation
2. Implement simulation tests for various scenarios
3. Execute simulation tests and fix any issues

### Phase 4: Performance Testing

1. Set up performance measurement infrastructure
2. Implement performance tests for critical metrics
3. Execute performance tests and analyze results
4. Optimize driver based on performance results

## Test Reporting

Test results will be documented in a comprehensive test report including:

1. **Test Summary**: Overview of test execution and results
2. **Unit Test Results**: Detailed results of unit tests
3. **Integration Test Results**: Detailed results of integration tests
4. **Simulation Test Results**: Detailed results of simulation tests
5. **Performance Test Results**: Detailed performance measurements
6. **Issues and Resolutions**: Any issues found and their resolutions
7. **Recommendations**: Recommendations for further improvements

## Performance Targets Validation

The following performance targets will be validated:

| Metric | Target | Test Method |
|--------|--------|------------|
| Vsync Accuracy | < 200μs between displays | Measure vsync timing differences |
| Frame Timing Jitter | < 1ms | Analyze timestamps of consecutive frames |
| Display Latency | < 5ms | Measure time from buffer submission to display |
| Refresh Rate | Stable 90Hz | Measure refresh rate over extended period |
| Motion-to-Photon Latency | < 20ms | Measure end-to-end latency with light sensor |
| Synchronization Error | < 500μs | Measure timing differences between displays |

## Conclusion

This test plan provides a comprehensive approach to validating the RK3588 display driver for VR applications. The combination of unit, integration, simulation, and performance testing will verify both functionality and performance characteristics critical for VR use cases, with particular focus on dual display synchronization, low persistence, and minimal latency.
