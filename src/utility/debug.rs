
use ash;
use ash::vk;
use ash::version::V1_0;
use ash::version::EntryV1_0;

use std::ffi::{ CStr, CString };
use std::ptr;

pub unsafe extern "system" fn vulkan_debug_callback(
    _: vk::DebugReportFlagsEXT,
    _: vk::DebugReportObjectTypeEXT,
    _: vk::uint64_t,
    _: vk::size_t,
    _: vk::int32_t,
    _: *const vk::c_char,
    p_message: *const vk::c_char,
    _: *mut vk::c_void,
) -> u32 {
    println!("{:?}", CStr::from_ptr(p_message));
    vk::VK_FALSE
}


pub struct ValidationInfo {
    pub is_enable: bool,
    pub required_validation_layers: [&'static str; 1],
}

impl ValidationInfo {

    pub fn get_layers_names(&self) -> Vec<*const i8> {
        let requred_validation_layer_raw_names: Vec<CString> = self.required_validation_layers.iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        requred_validation_layer_raw_names.iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect()
    }
}

pub fn check_validation_layer_support(entry: &ash::Entry<V1_0>, required_validation_layers: &Vec<&str>) -> bool {
    // if support validation layer, then return true

    let layer_properties = entry.enumerate_instance_layer_properties()
        .expect("Failed to enumerate instance layers properties");

    if layer_properties.len() <= 0 {
        eprintln!("No available layers.");
        return false
    }

    for required_layer_name in required_validation_layers.iter() {
        let mut is_layer_found = false;

        for layer_property in layer_properties.iter() {

            let test_layer_name = super::tools::convert_string(&layer_property.layer_name);
            if (*required_layer_name) == test_layer_name {
                is_layer_found = true;
                break
            }
        }

        if is_layer_found == false {
            return false
        }
    }

    true
}


pub fn setup_debug_callback(is_enable_debug: bool, entry: &ash::Entry<V1_0>, instance: &ash::Instance<V1_0>)
    -> (ash::extensions::DebugReport, vk::DebugReportCallbackEXT) {

    let debug_report_loader = ash::extensions::DebugReport::new(entry, instance)
        .expect("Unable to load debug report");

    if is_enable_debug == false {
        (debug_report_loader, ash::vk::types::DebugReportCallbackEXT::null())
    } else {

        let debug_create_info = vk::DebugReportCallbackCreateInfoEXT {
            s_type: vk::StructureType::DebugReportCallbackCreateInfoExt,
            p_next: ptr::null(),
            flags: vk::DEBUG_REPORT_ERROR_BIT_EXT
                | vk::DEBUG_REPORT_INFORMATION_BIT_EXT
                // | vk::DEBUG_REPORT_DEBUG_BIT_EXT
                | vk::DEBUG_REPORT_WARNING_BIT_EXT
                | vk::DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT,
            pfn_callback: vulkan_debug_callback,
            p_user_data: ptr::null_mut(),
        };

        let debug_call_back = unsafe {
            debug_report_loader.create_debug_report_callback_ext(&debug_create_info, None)
                .expect("Failed to set up debug callback!")
        };

        (debug_report_loader, debug_call_back)
    }
}