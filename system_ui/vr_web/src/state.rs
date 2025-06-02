use std::sync::Mutex;
use vr_core_api::VRCoreAPI;

pub struct AppState {
    pub core_api: Mutex<VRCoreAPI>,
}
