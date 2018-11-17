
extern crate vulkan_tutorial_rust;
use vulkan_tutorial_rust::{
    utility,
    utility::debug::ValidationInfo,
    utility::constants::*,
};

extern crate winit;
extern crate ash;

use winit::{ Event, EventsLoop, WindowEvent, ControlFlow, VirtualKeyCode };
use ash::vk;
use ash::version::InstanceV1_0;
use ash::version::EntryV1_0;
use std::ptr;
use std::ffi::{ CStr, CString };
use std::os::raw::{ c_void, c_char };

// Constants
const WINDOW_TITLE: &'static str = "02.Validation Layers";
const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: [
        "VK_LAYER_LUNARG_standard_validation",
    ],
};

unsafe extern "system" fn vulkan_debug_callback(
    _: vk::DebugReportFlagsEXT,
    _: vk::DebugReportObjectTypeEXT,
    _: u64,
    _: usize,
    _: i32,
    _: *const c_char,
    p_message: *const c_char,
    _: *mut c_void,
) -> u32 {
    println!("{:?}", CStr::from_ptr(p_message));
    vk::FALSE
}


struct VulkanApp {
    // winit stuff
    events_loop         : EventsLoop,
    _window             : winit::Window,

    // vulkan stuff
    _entry              : ash::Entry,
    instance            : ash::Instance,
    debug_report_loader : ash::extensions::DebugReport,
    debug_callback      : vk::DebugReportCallbackEXT,
}

impl VulkanApp {

    pub fn new() -> VulkanApp {

        // init window stuff
        let events_loop = EventsLoop::new();
        let window = VulkanApp::init_window(&events_loop);

        // init vulkan stuff
        let entry = ash::Entry::new().unwrap();
        let instance = VulkanApp::create_instance(&entry);
        let (debug_report_loader, debug_callback) = VulkanApp::setup_debug_callback(&entry, &instance);

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp {
            events_loop,
            _window: window,

            _entry: entry,
            instance,
            debug_report_loader,
            debug_callback,
        }
    }

    fn init_window(events_loop: &EventsLoop) -> winit::Window {

        winit::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_dimensions((WINDOW_WIDTH, WINDOW_HEIGHT).into())
            .build(events_loop)
            .expect("Failed to create window.")
    }

    fn create_instance(entry: &ash::Entry) -> ash::Instance {

        if VALIDATION.is_enable && VulkanApp::check_validation_layer_support(entry) == false {
            panic!("Validation layers requested, but not available!");
        }

        let app_name    = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new("Vulkan Engine").unwrap();
        let app_info = vk::ApplicationInfo {
            p_application_name  : app_name.as_ptr(),
            s_type              : vk::StructureType::APPLICATION_INFO,
            p_next              : ptr::null(),
            application_version : APPLICATION_VERSION,
            p_engine_name       : engine_name.as_ptr(),
            engine_version      : ENGINE_VERSION,
            api_version         : API_VERSION,
        };

        // VK_EXT debug report has been requested here.
        let extension_names = utility::platforms::required_extension_names();

        let requred_validation_layer_raw_names: Vec<CString> = VALIDATION.required_validation_layers.iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        let enable_layer_names: Vec<*const i8> = requred_validation_layer_raw_names.iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let create_info = vk::InstanceCreateInfo {
            s_type                     : vk::StructureType::INSTANCE_CREATE_INFO,
            p_next                     : ptr::null(),
            flags                      : vk::InstanceCreateFlags::empty(),
            p_application_info         : &app_info,
            pp_enabled_layer_names     : if VALIDATION.is_enable { enable_layer_names.as_ptr() } else { ptr::null() },
            enabled_layer_count        : if VALIDATION.is_enable { enable_layer_names.len() } else { 0 } as u32,
            pp_enabled_extension_names : extension_names.as_ptr(),
            enabled_extension_count    : extension_names.len() as u32,
        };

        let instance: ash::Instance = unsafe {
            entry.create_instance(&create_info, None)
                .expect("Failed to create Instance!")
        };

        instance
    }

    fn check_validation_layer_support(entry: &ash::Entry) -> bool {
        // if support validation layer, then return true

        let layer_properties = entry.enumerate_instance_layer_properties()
            .expect("Failed to enumerate Instance Layers Properties!");

        if layer_properties.len() <= 0 {
            eprintln!("No available layers.");
            return false
        } else {

            println!("Instance Available Layers: ");
            for layer in layer_properties.iter() {
                let layer_name = utility::tools::vk_to_string(&layer.layer_name);
                println!("\t{}", layer_name);
            }
        }

        for required_layer_name in VALIDATION.required_validation_layers.iter() {
            let mut is_layer_found = false;

            for layer_property in layer_properties.iter() {

                let test_layer_name = utility::tools::vk_to_string(&layer_property.layer_name);
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

    fn setup_debug_callback(entry: &ash::Entry, instance: &ash::Instance)
        -> (ash::extensions::DebugReport, vk::DebugReportCallbackEXT) {

        let debug_report_loader = ash::extensions::DebugReport::new(entry, instance);

        if VALIDATION.is_enable == false {
            (debug_report_loader, ash::vk::DebugReportCallbackEXT::null())
        } else {

            let debug_create_info = vk::DebugReportCallbackCreateInfoEXT {
                s_type : vk::StructureType::DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT,
                p_next : ptr::null(),
                flags  : vk::DebugReportFlagsEXT::ERROR
                    | vk::DebugReportFlagsEXT::INFORMATION
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

    pub fn main_loop(&mut self) {

        self.events_loop.run_forever(|event| {

            match event {
                // handling keyboard event
                | Event::WindowEvent { event, .. } => match event {
                    | WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            ControlFlow::Break
                        } else {
                            ControlFlow::Continue
                        }
                    }
                    | WindowEvent::CloseRequested => ControlFlow::Break,
                    | _ => ControlFlow::Continue,
                },
                | _ => ControlFlow::Continue,
            }
        });
    }
}

impl Drop for VulkanApp {

    fn drop(&mut self) {

        unsafe {

            if VALIDATION.is_enable {
                self.debug_report_loader.destroy_debug_report_callback_ext(self.debug_callback, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

fn main() {

    let mut vulkan_app = VulkanApp::new();
    vulkan_app.main_loop();
}
