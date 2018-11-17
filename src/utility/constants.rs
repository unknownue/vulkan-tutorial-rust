
use ash;
use utility::debug::ValidationInfo;
use utility::structures::DeviceExtension;

use std::os::raw::c_char;

pub const APPLICATION_VERSION: u32 = vk_make_version!(1, 0, 0);
pub const ENGINE_VERSION:      u32 = vk_make_version!(1, 0, 0);
pub const API_VERSION:         u32 = vk_make_version!(1, 0, 85);

pub const WINDOW_WIDTH:  u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;
pub const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: [
        "VK_LAYER_LUNARG_standard_validation",
    ],
};
pub const DEVICE_EXTENSIONS: DeviceExtension = DeviceExtension {
    names: ["VK_KHR_swapchain"],
};
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
pub const IS_PAINT_FPS_COUNTER: bool = false;

impl DeviceExtension {

    pub fn get_extensions_raw_names(&self) -> [*const c_char; 1] {
        [ // currently just enable the Swapchain extension.
            ash::extensions::Swapchain::name().as_ptr(),
        ]
    }
}
