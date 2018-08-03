
use ash;
use ash::vk;
use ash::version::V1_0;
use ash::version::EntryV1_0;

use std::ptr;
use std::ffi::CString;

use super::debug;

pub fn create_instance(entry: &ash::Entry<V1_0>, window_title: &str, is_enable_debug: bool, required_validation_layers: &Vec<&str>) -> ash::Instance<V1_0> {

    if is_enable_debug && debug::check_validation_layer_support(entry, required_validation_layers) == false {
        panic!("Validation layers requested, but not available!");
    }

    let app_name = CString::new(window_title).unwrap();
    let engine_name = CString::new("Vulkan Engine").unwrap();
    let app_info = vk::ApplicationInfo {
        p_application_name: app_name.as_ptr(),
        s_type: vk::StructureType::ApplicationInfo,
        p_next: ptr::null(),
        application_version: vk_make_version!(1, 0, 0),
        p_engine_name: engine_name.as_ptr(),
        engine_version: vk_make_version!(1, 0, 0),
        api_version: vk_make_version!(1, 0, 36),
    };

    // VK_EXT debug report has been requested here.
    let extension_names = super::required_extension_names();

    let requred_validation_layer_raw_names: Vec<CString> = required_validation_layers.iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();
    let layer_names: Vec<*const i8> = requred_validation_layer_raw_names.iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::InstanceCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        p_application_info: &app_info,
        pp_enabled_layer_names: if is_enable_debug { layer_names.as_ptr() } else { ptr::null() },
        enabled_layer_count: if is_enable_debug { layer_names.len() } else { 0 } as u32,
        pp_enabled_extension_names: extension_names.as_ptr(),
        enabled_extension_count: extension_names.len() as u32,
    };

    let instance: ash::Instance<V1_0> = unsafe { entry.create_instance(&create_info, None)
        .expect("Failed to create instance!")
    };

    instance
}
