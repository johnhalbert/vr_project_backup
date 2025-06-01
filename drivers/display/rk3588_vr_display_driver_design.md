# Display Driver (DRM/KMS for RK3588) Design Document

## Overview

This document outlines the design for the RK3588 display driver for the Linux kernel, targeting integration with the Direct Rendering Manager (DRM) and Kernel Mode Setting (KMS) subsystems. The driver will support dual display output for VR headsets with specific optimizations for low latency and synchronization.

## Driver Architecture

The driver will extend the existing Rockchip DRM/KMS driver with VR-specific enhancements:

```
drivers/gpu/drm/rockchip/
├── rockchip_drm_vr.c      # VR-specific extensions
├── rockchip_drm_vr.h      # VR extension header
├── rockchip_vr_display.c  # VR display management
├── rockchip_vr_sync.c     # Display synchronization
├── rockchip_vr_lowlat.c   # Low latency optimizations
├── rockchip_vr_dts.h      # Device tree bindings
├── Kconfig                # Kernel configuration options
└── Makefile               # Build system integration
```

## Key Components

### 1. VR Extensions (rockchip_drm_vr.c/h)

The VR extensions handle:
- Dual display configuration
- Display synchronization
- Low persistence mode
- Direct mode rendering
- Distortion correction

### 2. VR Display Management (rockchip_vr_display.c)

Manages VR-specific display features:
- Display mode configuration
- Resolution and refresh rate management
- Color space and gamma correction
- Display power management
- EDID handling for VR displays

### 3. Display Synchronization (rockchip_vr_sync.c)

Implements precise synchronization between displays:
- Vsync alignment between left and right displays
- Frame timing control
- Tear-free rendering
- Scanout synchronization

### 4. Low Latency Optimizations (rockchip_vr_lowlat.c)

Implements optimizations for minimal motion-to-photon latency:
- Direct scanout paths
- Minimal buffer copies
- Efficient page flipping
- Predictive scanout

## DRM/KMS Integration

The driver will integrate with the Linux DRM/KMS subsystem:

1. **DRM Device**:
   - Primary and secondary CRTC for dual displays
   - Planes for composition
   - Encoders and connectors for display output

2. **KMS Properties**:
   - VR mode enable/disable
   - Low persistence control
   - Synchronization options
   - Latency reduction settings

3. **DRM Events**:
   - Vsync events
   - Pageflip completion
   - Mode change events
   - Error events

## VR-Specific Features

Special features for VR applications:

1. **Dual Display Synchronization**:
   - Precise timing alignment between left and right displays
   - Synchronized page flips
   - Matched refresh rates and timings

2. **Low Persistence Mode**:
   - Reduced pixel illumination time
   - Strobed backlight control
   - Reduced motion blur

3. **Direct Mode Rendering**:
   - Direct GPU-to-display path
   - Minimal compositor intervention
   - Dedicated display path for VR

4. **Distortion Correction**:
   - Hardware-accelerated lens distortion correction
   - Chromatic aberration correction
   - Mesh-based distortion

## RK3588 Display Hardware

The RK3588 SoC includes advanced display capabilities:

1. **Display Controller**:
   - VOPDC (Video Output Processor Display Controller)
   - Multiple video layers
   - Hardware scaling and color space conversion
   - HDR processing

2. **Display Interfaces**:
   - HDMI 2.1 (up to 8K@60Hz)
   - DisplayPort 1.4 (up to 8K@30Hz)
   - MIPI DSI (dual-channel)
   - eDP (Embedded DisplayPort)

3. **Hardware Composition**:
   - Multiple hardware layers
   - Alpha blending
   - Z-order control
   - Hardware cursor

## Display Configuration for VR

The driver will support a dual-display configuration for VR:

```
┌─────────────────────────────────────────────────────────────┐
│                      RK3588 SoC                             │
│                                                             │
│  ┌───────────┐     ┌───────────┐     ┌───────────────────┐  │
│  │           │     │ VOPDC     │     │ MIPI DSI 0        │  │
│  │           │     │ (Primary) │────▶│ (Left Display)    │  │
│  │           │     └───────────┘     └───────────────────┘  │
│  │   GPU     │                                              │
│  │           │     ┌───────────┐     ┌───────────────────┐  │
│  │           │     │ VOPDC     │     │ MIPI DSI 1        │  │
│  │           │────▶│ (Secondary)│────▶│ (Right Display)   │  │
│  └───────────┘     └───────────┘     └───────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Synchronization Mechanisms

The driver will implement multiple synchronization mechanisms:

1. **Hardware Synchronization**:
   - Shared PLL clock source
   - Synchronized VOPDC timing
   - Hardware genlock

2. **Software Synchronization**:
   - Atomic modesetting for synchronized updates
   - Coordinated vblank intervals
   - Frame sequence tracking

3. **GPU Synchronization**:
   - Synchronized GPU submission
   - Fence synchronization
   - Explicit synchronization primitives

## Performance Optimizations

1. **Scanout Optimization**:
   - Direct scanout from GPU buffers
   - Minimal composition overhead
   - Efficient memory bandwidth usage

2. **Vsync Timing**:
   - Precise vsync prediction
   - Adaptive vsync for latency reduction
   - Frame pacing control

3. **Buffer Management**:
   - Triple buffering for tear-free rendering
   - Buffer age tracking
   - Efficient buffer reuse

4. **Power Management**:
   - Dynamic refresh rate adjustment
   - Panel self-refresh for static content
   - Selective component power gating

## Device Tree Binding Example

```
&vopdc {
    status = "okay";
    rockchip,vr-mode = <1>;
    rockchip,low-persistence = <1>;
    rockchip,sync-displays = <1>;
};

&vopdcsc {
    status = "okay";
};

&dsi0 {
    status = "okay";
    rockchip,lane-rate = <1000>;
    
    panel@0 {
        compatible = "vr,display-left";
        reg = <0>;
        backlight = <&backlight_left>;
        reset-gpios = <&gpio1 RK_PA0 GPIO_ACTIVE_LOW>;
        enable-gpios = <&gpio1 RK_PA1 GPIO_ACTIVE_HIGH>;
        
        display-timings {
            native-mode = <&timing0>;
            
            timing0: timing0 {
                clock-frequency = <150000000>;
                hactive = <1832>;
                vactive = <1920>;
                hfront-porch = <8>;
                hsync-len = <8>;
                hback-porch = <16>;
                vfront-porch = <8>;
                vsync-len = <8>;
                vback-porch = <16>;
                hsync-active = <0>;
                vsync-active = <0>;
                de-active = <0>;
                pixelclk-active = <0>;
            };
        };
    };
};

&dsi1 {
    status = "okay";
    rockchip,lane-rate = <1000>;
    
    panel@0 {
        compatible = "vr,display-right";
        reg = <0>;
        backlight = <&backlight_right>;
        reset-gpios = <&gpio1 RK_PA2 GPIO_ACTIVE_LOW>;
        enable-gpios = <&gpio1 RK_PA3 GPIO_ACTIVE_HIGH>;
        
        display-timings {
            native-mode = <&timing1>;
            
            timing1: timing1 {
                clock-frequency = <150000000>;
                hactive = <1832>;
                vactive = <1920>;
                hfront-porch = <8>;
                hsync-len = <8>;
                hback-porch = <16>;
                vfront-porch = <8>;
                vsync-len = <8>;
                vback-porch = <16>;
                hsync-active = <0>;
                vsync-active = <0>;
                de-active = <0>;
                pixelclk-active = <0>;
            };
        };
    };
};
```

## Userspace Interface

The driver will expose VR-specific controls through the DRM API:

```c
/* Enable VR mode */
struct drm_property *vr_mode_property;
vr_mode_property = drm_property_create_bool(dev, 0, "VR_MODE");
drm_object_attach_property(&crtc->base, vr_mode_property, 0);

/* Set low persistence mode */
struct drm_property *low_persistence_property;
low_persistence_property = drm_property_create_range(dev, 0, "LOW_PERSISTENCE", 0, 100);
drm_object_attach_property(&crtc->base, low_persistence_property, 0);

/* Control display synchronization */
struct drm_property *sync_displays_property;
sync_displays_property = drm_property_create_bool(dev, 0, "SYNC_DISPLAYS");
drm_object_attach_property(&crtc->base, sync_displays_property, 0);
```

## Implementation Plan

1. **Phase 1: Base Integration**
   - Extend existing Rockchip DRM driver
   - Implement dual display support
   - Basic synchronization

2. **Phase 2: VR Features**
   - Low persistence mode
   - Direct mode rendering
   - Advanced synchronization

3. **Phase 3: Performance Optimization**
   - Latency reduction
   - Distortion correction
   - Power optimization

4. **Phase 4: Testing and Validation**
   - Timing accuracy testing
   - Latency measurement
   - Power consumption analysis

## Performance Targets

- **Vsync Accuracy**: < 200μs between displays
- **Frame Timing Jitter**: < 1ms
- **Display Latency**: < 5ms from buffer submission to display
- **Refresh Rate**: Stable 90Hz operation
- **Motion-to-Photon Latency**: < 20ms end-to-end

## Conclusion

This driver design provides a comprehensive solution for integrating the RK3588 display controller into a VR headset system, with specific optimizations for dual display synchronization, low persistence, and minimal latency. The modular architecture ensures maintainability and extensibility, while the DRM/KMS integration provides a standard interface for applications to control the displays.
