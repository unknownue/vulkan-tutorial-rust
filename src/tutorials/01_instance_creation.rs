
use vulkan_tutorial_rust::utility;
use vulkan_tutorial_rust::utility::constants::*;

use winit::{ Event, EventsLoop, WindowEvent, ControlFlow, VirtualKeyCode };
use ash::vk;
use ash::version::InstanceV1_0;
use ash::version::EntryV1_0;
use std::ptr;
use std::ffi::CString;

// Constants
const WINDOW_TITLE: &'static str = "01.Instance Creation";

struct VulkanApp {
    // winit stuff
    events_loop : EventsLoop,
    _window     : winit::Window,

    instance    : ash::Instance,
}

impl VulkanApp {

    pub fn new() -> VulkanApp {

        // init window stuff
        let events_loop = EventsLoop::new();
        let window = VulkanApp::init_window(&events_loop);

        // init vulkan stuff
        let instance = VulkanApp::create_instance();

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp {
            events_loop,
            _window: window,
            instance,
        }
    }

    fn init_window(events_loop: &EventsLoop) -> winit::Window {

        winit::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_dimensions((WINDOW_WIDTH, WINDOW_HEIGHT).into())
            .build(events_loop)
            .expect("Failed to create window.")
    }

    fn create_instance() -> ash::Instance {

        let entry = ash::Entry::new().unwrap();

        let app_name    = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new("Vulkan Engine").unwrap();
        let app_info = vk::ApplicationInfo {
            s_type              : vk::StructureType::APPLICATION_INFO,
            p_next              : ptr::null(),
            p_application_name  : app_name.as_ptr(),
            application_version : APPLICATION_VERSION,
            p_engine_name       : engine_name.as_ptr(),
            engine_version      : ENGINE_VERSION,
            api_version         : API_VERSION,
        };

        let extension_names = utility::platforms::required_extension_names();

        let create_info = vk::InstanceCreateInfo {
            s_type                     : vk::StructureType::INSTANCE_CREATE_INFO,
            p_next                     : ptr::null(),
            flags                      : vk::InstanceCreateFlags::empty(),
            p_application_info         : &app_info,
            pp_enabled_layer_names     : ptr::null(),
            enabled_layer_count        : 0,
            pp_enabled_extension_names : extension_names.as_ptr(),
            enabled_extension_count    : extension_names.len() as u32,
        };

        let instance: ash::Instance = unsafe { entry.create_instance(&create_info, None)
            .expect("Failed to create instance!")
        };

        instance
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
            self.instance.destroy_instance(None);
        }
    }
}

fn main() {

    let mut vulkan_app = VulkanApp::new();
    vulkan_app.main_loop();
}
