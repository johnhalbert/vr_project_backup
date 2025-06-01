//! C++ implementation of the OpenVR driver interface

#include <openvr_driver.h>
#include <cstring>
#include <vector>
#include <string>
#include <memory>

// Forward declarations for the Rust FFI functions
extern "C" {
    void* vr_driver_get_server_provider();
    int vr_driver_init(void* context, void* driver_log, void* driver_host, void* driver_input, void* driver_properties, void* driver_settings);
    int vr_driver_run_frame();
    int vr_driver_cleanup();
    int vr_driver_enter_standby();
    int vr_driver_leave_standby();
    const char* const* vr_driver_get_interface_versions();
}

// Server tracked device provider implementation
class CServerTrackedDeviceProvider : public vr::IServerTrackedDeviceProvider {
public:
    CServerTrackedDeviceProvider() : m_initialized(false) {}
    
    virtual vr::EVRInitError Init(vr::IVRDriverContext* pDriverContext) override {
        // Initialize the driver context
        vr::EVRInitError init_error = vr::InitServerDriverContext(pDriverContext);
        if (init_error != vr::VRInitError_None) {
            return init_error;
        }
        
        // Initialize the Rust driver
        int result = vr_driver_init(
            pDriverContext,
            vr::VRDriverLog(),
            vr::VRServerDriverHost(),
            vr::VRDriverInput(),
            vr::VRProperties(),
            vr::VRSettings()
        );
        
        if (result != 0) {
            return static_cast<vr::EVRInitError>(result);
        }
        
        m_initialized = true;
        return vr::VRInitError_None;
    }
    
    virtual void Cleanup() override {
        if (m_initialized) {
            vr_driver_cleanup();
            m_initialized = false;
        }
    }
    
    virtual const char* const* GetInterfaceVersions() override {
        return vr_driver_get_interface_versions();
    }
    
    virtual void RunFrame() override {
        if (m_initialized) {
            vr_driver_run_frame();
        }
    }
    
    virtual bool ShouldBlockStandbyMode() override {
        return false;
    }
    
    virtual void EnterStandby() override {
        if (m_initialized) {
            vr_driver_enter_standby();
        }
    }
    
    virtual void LeaveStandby() override {
        if (m_initialized) {
            vr_driver_leave_standby();
        }
    }
    
private:
    bool m_initialized;
};

// Factory function to create the server tracked device provider
extern "C" __declspec(dllexport) void* HmdDriverFactory(const char* pInterfaceName, int* pReturnCode) {
    if (0 == strcmp(vr::IServerTrackedDeviceProvider_Version, pInterfaceName)) {
        return new CServerTrackedDeviceProvider();
    }
    
    if (pReturnCode) {
        *pReturnCode = vr::VRInitError_Init_InterfaceNotFound;
    }
    
    return nullptr;
}
