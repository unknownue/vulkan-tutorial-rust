
use ash;
use ash::vk;
use ash::version::EntryV1_0;

use std::ffi::{ CStr, CString };
use std::ptr;
use std::os::raw::{ c_void, c_char };

pub unsafe extern "system" fn vulkan_debug_callback(
    _: vk::DebugReportFlagsEXT,
    _: vk::DebugReportObjectTypeEXT,
    _: u64,
    _: usize,
    _: i32,
    _: *const c_char,
    p_message: *const c_char,
    _: *mut c_void,
) -> u32 {
    println!("[Debug] -> {:?}", CStr::from_ptr(p_message));
    vk::FALSE
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

pub fn check_validation_layer_support(entry: &ash::Entry, required_validation_layers: &Vec<&str>) -> bool {
    // if support validation layer, then return true

    let layer_properties = entry.enumerate_instance_layer_properties()
        .expect("Failed to enumerate Instance Layers Properties");

    if layer_properties.len() <= 0 {
        eprintln!("No available layers.");
        return false
    }

    for required_layer_name in required_validation_layers.iter() {
        let mut is_layer_found = false;

        for layer_property in layer_properties.iter() {

            let test_layer_name = super::tools::vk_to_string(&layer_property.layer_name);
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


pub fn setup_debug_callback(is_enable_debug: bool, entry: &ash::Entry, instance: &ash::Instance)
    -> (ash::extensions::DebugReport, vk::DebugReportCallbackEXT) {

    let debug_report_loader = ash::extensions::DebugReport::new(entry, instance);

    if is_enable_debug == false {
        (debug_report_loader, ash::vk::DebugReportCallbackEXT::null())
    } else {

        let debug_create_info = vk::DebugReportCallbackCreateInfoEXT {
            s_type : vk::StructureType::DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT,
            p_next : ptr::null(),
            flags  : vk::DebugReportFlagsEXT::ERROR
                 // | vk::DebugReportFlagsEXT::INFORMATION
                 // | vk::DebugReportFlagsEXT::DEBUG
                    | vk::DebugReportFlagsEXT::WARNING
                    | vk::DebugReportFlagsEXT::PERFORMANCE_WARNING,
            pfn_callback : Some(vulkan_debug_callback),
            p_user_data  : ptr::null_mut(),
        };

        let debug_call_back = unsafe {
            debug_report_loader.create_debug_report_callback_ext(&debug_create_info, None)
                .expect("Failed to set up Debug Callback!")
        };

        (debug_report_loader, debug_call_back)
    }
}