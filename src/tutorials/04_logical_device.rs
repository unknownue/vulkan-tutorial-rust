use vulkan_tutorial_rust::{
    utility, // the mod define some fixed functions that have been learned before.
    utility::constants::*,
    utility::debug::ValidationInfo,
    utility::share,
};

use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;
use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};

use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

// Constants
const WINDOW_TITLE: &'static str = "04.Logical Device";

struct QueueFamilyIndices {
    graphics_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
    }
}

struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_merssager: vk::DebugUtilsMessengerEXT,
    _physical_device: vk::PhysicalDevice,
    device: ash::Device, // Logical Device
    _graphics_queue: vk::Queue,
}

impl VulkanApp {
    pub fn new() -> VulkanApp {
        // init vulkan stuff
        let entry = ash::Entry::new().unwrap();
        let instance = share::create_instance(
            &entry,
            WINDOW_TITLE,
            VALIDATION.is_enable,
            &VALIDATION.required_validation_layers.to_vec(),
        );
        let (debug_utils_loader, debug_merssager) =
            utility::debug::setup_debug_utils(VALIDATION.is_enable, &entry, &instance);
        let physical_device = VulkanApp::pick_physical_device(&instance);
        let (logical_device, graphics_queue) =
            VulkanApp::create_logical_device(&instance, physical_device, &VALIDATION);

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp {
            _entry: entry,
            instance,
            debug_utils_loader,
            debug_merssager,
            _physical_device: physical_device,
            device: logical_device,
            _graphics_queue: graphics_queue,
        }
    }

    fn pick_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate Physical Devices!")
        };

        let result = physical_devices.iter().find(|physical_device| {
            VulkanApp::is_physical_device_suitable(instance, **physical_device)
        });

        match result {
            Some(p_physical_device) => *p_physical_device,
            None => panic!("Failed to find a suitable GPU!"),
        }
    }

    fn is_physical_device_suitable(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> bool {
        let _device_properties =
            unsafe { instance.get_physical_device_properties(physical_device) };
        let _device_features = unsafe { instance.get_physical_device_features(physical_device) };

        let indices = VulkanApp::find_queue_family(instance, physical_device);

        return indices.is_complete();
    }

    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        validation: &ValidationInfo,
    ) -> (ash::Device, vk::Queue) {
        let indices = VulkanApp::find_queue_family(instance, physical_device);

        let queue_priorities = [1.0_f32];
        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: indices.graphics_family.unwrap(),
            p_queue_priorities: queue_priorities.as_ptr(),
            queue_count: queue_priorities.len() as u32,
        };

        let physical_device_features = vk::PhysicalDeviceFeatures {
            ..Default::default() // default just enable no feature.
        };

        let requred_validation_layer_raw_names: Vec<CString> = validation
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        let enable_layer_names: Vec<*const c_char> = requred_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_create_info,
            enabled_layer_count: if validation.is_enable {
                enable_layer_names.len()
            } else {
                0
            } as u32,
            pp_enabled_layer_names: if validation.is_enable {
                enable_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_extension_count: 0,
            pp_enabled_extension_names: ptr::null(),
            p_enabled_features: &physical_device_features,
        };

        let device: ash::Device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .expect("Failed to create logical Device!")
        };

        let graphics_queue = unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };

        (device, graphics_queue)
    }

    fn find_queue_family(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = QueueFamilyIndices {
            graphics_family: None,
        };

        let mut index = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        queue_family_indices
    }

    fn draw_frame(&mut self) {
        // Drawing will be here
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);

            if VALIDATION.is_enable {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_merssager, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

// Fix content -------------------------------------------------------------------------------
impl VulkanApp {
    pub fn main_loop(mut self, event_loop: EventLoop<()>, window: winit::window::Window) {

         event_loop.run(move |event, _, control_flow| {

            match event {
                | Event::WindowEvent { event, .. } => {
                    match event {
                        | WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit
                        },
                        | WindowEvent::KeyboardInput { input, .. } => {
                            match input {
                                | KeyboardInput { virtual_keycode, state, .. } => {
                                    match (virtual_keycode, state) {
                                        | (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                            *control_flow = ControlFlow::Exit
                                        },
                                        | _ => {},
                                    }
                                },
                            }
                        },
                        | _ => {},
                    }
                },
                | Event::MainEventsCleared => {
                    window.request_redraw();
                },
                | Event::RedrawRequested(_window_id) => {
                    self.draw_frame();
                },
                _ => (),
            }

        })
    }
}


fn main() {
    let event_loop = EventLoop::new();
    let window = utility::window::init_window(&event_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

    let vulkan_app = VulkanApp::new();
    vulkan_app.main_loop(event_loop, window);
}
// -------------------------------------------------------------------------------------------
