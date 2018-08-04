
extern crate vulkan_tutorial_rust;
use vulkan_tutorial_rust::utility; // the mod define some fixed functions that have been learned before.
use vulkan_tutorial_rust::utility::debug::ValidationInfo;

extern crate winit;
extern crate ash;

use winit::{ Event, EventsLoop, WindowEvent, ControlFlow, VirtualKeyCode };
use ash::vk;
use ash::version::{ V1_0, InstanceV1_0 };

type EntryV1 = ash::Entry<V1_0>;

// Constants
const WINDOW_TITLE: &'static str = "03.Physical Device Selection";
const WINDOW_WIDTH:  u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: [
        "VK_LAYER_LUNARG_standard_validation",
    ],
};

struct QueueFamilyIndices {
    graphics_family: i32,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family >= 0
    }
}

struct VulkanApp {
    // winit stuff
    events_loop: EventsLoop,
    _window: winit::Window,

    // vulkan stuff
    _entry: EntryV1,
    instance: ash::Instance<V1_0>,
    debug_report_loader: ash::extensions::DebugReport,
    debug_callback: vk::DebugReportCallbackEXT,
    _physical_device: vk::PhysicalDevice,
}

impl VulkanApp {

    pub fn new() -> VulkanApp {

        // init window stuff
        let events_loop = EventsLoop::new();
        let window = utility::window::init_window(&events_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        // init vulkan stuff
        let entry = EntryV1::new().unwrap();
        let instance = utility::vulkan::create_instance(&entry, WINDOW_TITLE, VALIDATION.is_enable, &VALIDATION.required_validation_layers.to_vec());
        let (debug_report_loader, debug_callback) = utility::debug::setup_debug_callback( VALIDATION.is_enable, &entry, &instance);
        let physical_device = VulkanApp::pick_physical_device(&instance);

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp {
            events_loop,
            _window: window,

            _entry: entry,
            instance,
            debug_report_loader,
            debug_callback,
            _physical_device: physical_device,
        }
    }

    fn pick_physical_device(instance: &ash::Instance<V1_0>) -> vk::PhysicalDevice {
        let physical_devices = instance.enumerate_physical_devices()
            .expect("Physical device error");

        println!("{} devices (GPU) found with vulkan support.", physical_devices.len());

        let mut result = None;
        for physical_device in physical_devices.iter() {
            if VulkanApp::is_physical_device_suitable(instance, physical_device) {
                if result.is_none() {
                    result = Some(*physical_device)
                }
            }
        }

        match result {
            | None => panic!("Failed to find a suitable GPU!"),
            | Some(physical_device) => physical_device,
        }
    }

    fn is_physical_device_suitable(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice) -> bool {

        let device_properties = instance.get_physical_device_properties(physical_device.clone());
        let device_features = instance.get_physical_device_features(physical_device.clone());

        use vk::PhysicalDeviceType::*;
        let device_type = match device_properties.device_type {
            | Cpu => "Cpu",
            | IntegratedGpu => "Integrated GPU",
            | DiscreteGpu => "Discrete GPU",
            | VirtualGpu => "Virtual GPU",
            | Other => "Unknown",
        };

        let device_name = utility::tools::convert_string(&device_properties.device_name);
        println!("Device Name: {}, id: {}, type: {}", device_name, device_properties.device_id, device_type);

        // there are plenty of features
        println!("Geometry Shader support: {}", if device_features.geometry_shader == 1 { "Support" } else { "Unsupport" });

        let indices = VulkanApp::find_queue_family(instance, physical_device);

        return indices.is_complete();
    }

    fn find_queue_family(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice) -> QueueFamilyIndices {

        let queue_families = instance.get_physical_device_queue_family_properties(physical_device.clone());

        let mut queue_family_indices = QueueFamilyIndices {
            graphics_family: -1,
        };

        let mut index = 0;
        for queue_family in queue_families.iter() {
            use ash::vk::types::{ QueueFlags, QUEUE_GRAPHICS_BIT };
            if queue_family.queue_count > 0 && queue_family.queue_flags.subset(QueueFlags::from(QUEUE_GRAPHICS_BIT)) {
                queue_family_indices.graphics_family = index;
            }

            if queue_family_indices.is_complete() {
                break
            }

            index += 1;
        }

        queue_family_indices
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




// Fix content -------------------------------------------------------------------------------
impl VulkanApp {

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

fn main() {

    let mut vulkan_app = VulkanApp::new();
    vulkan_app.main_loop();
}
// -------------------------------------------------------------------------------------------

