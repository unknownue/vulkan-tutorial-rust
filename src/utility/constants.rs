

use ash;
use ash::vk::{
    uint32_t,
    VK_KHR_SWAPCHAIN_EXTENSION_NAME,
};
use utility::debug::ValidationInfo;
use utility::structures::DeviceExtension;

use std::os::raw::c_char;

pub const APPLICATION_VERSION: uint32_t = vk_make_version!(1, 0, 0);
pub const ENGINE_VERSION:      uint32_t = vk_make_version!(1, 0, 0);
pub const API_VERSION:         uint32_t = vk_make_version!(1, 0, 82);

pub const WINDOW_WIDTH:  u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;
pub const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: [
        "VK_LAYER_LUNARG_standard_validation",
    ],
};
pub const DEVICE_EXTENSIONS: DeviceExtension = DeviceExtension {
    names: [VK_KHR_SWAPCHAIN_EXTENSION_NAME],
};
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
pub const IS_PAINT_FPS_COUNTER: bool = true;

impl DeviceExtension {

    pub fn get_extensions_raw_names(&self) -> [*const c_char; 1] {
        [ // currently just enable the Swapchain extension.
            ash::extensions::Swapchain::name().as_ptr(),
        ]
    }
}

