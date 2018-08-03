
extern crate vulkan_tutorial_rust;
use vulkan_tutorial_rust::utility; // the mod define some fixed functions that have been learned before.
use vulkan_tutorial_rust::utility::debug::ValidationInfo;

extern crate winit;
extern crate ash;

use winit::{ Event, EventsLoop, WindowEvent, ControlFlow, VirtualKeyCode };
use ash::vk;
use ash::version::{ V1_0, InstanceV1_0 };
use ash::version::DeviceV1_0;

use std::ptr;

type EntryV1 = ash::Entry<V1_0>;

// Constants
const WINDOW_TITLE: &'static str = "05.Window Surface";
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
    present_family:  i32,
}

impl QueueFamilyIndices {

    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family: -1,
            present_family:  -1,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family >= 0 && self.present_family >= 0
    }
}

struct SurfaceStruct {
    surface_loader: ash::extensions::Surface,
    surface: vk::SurfaceKHR,
}

struct VulkanApp {
    // winit stuff
    events_loop: EventsLoop,
    _window: winit::Window,

    // vulkan stuff
    _entry: EntryV1,
    instance: ash::Instance<V1_0>,
    surface_loader: ash::extensions::Surface,
    surface: vk::SurfaceKHR,
    debug_report_loader: ash::extensions::DebugReport,
    debug_callback: vk::DebugReportCallbackEXT,
    _physical_device: vk::PhysicalDevice,
    device: ash::Device<V1_0>,
    _graphics_queue: vk::Queue,
    _present_queue: vk::Queue,
}

impl VulkanApp {

    pub fn new() -> VulkanApp {

        // init window stuff
        let events_loop = EventsLoop::new();
        let window = utility::window::init_window(&events_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        // init vulkan stuff
        let entry = EntryV1::new().unwrap();
        let instance = utility::vulkan::create_instance(&entry, WINDOW_TITLE, VALIDATION.is_enable, &VALIDATION.required_validation_layers.to_vec());
        let surface_struct = VulkanApp::create_surface(&entry, &instance, &window);
        let (debug_report_loader, debug_callback) = utility::debug::setup_debug_callback( VALIDATION.is_enable, &entry, &instance);
        let physical_device = VulkanApp::pick_physical_device(&instance, &surface_struct);
        let (device, family_indices) = VulkanApp::create_logical_device(&instance, &physical_device, &VALIDATION, &surface_struct);
        let graphics_queue = unsafe { device.get_device_queue(family_indices.graphics_family as u32, 0) };
        let present_queue  = unsafe { device.get_device_queue(family_indices.present_family as u32, 0) };

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp {
            // winit stuff
            events_loop,
            _window: window,

            // vulkan stuff
            _entry: entry,
            instance,
            surface: surface_struct.surface,
            surface_loader: surface_struct.surface_loader,
            debug_report_loader,
            debug_callback,
            _physical_device: physical_device,
            device,
            _graphics_queue: graphics_queue,
            _present_queue: present_queue,
        }
    }

    fn create_surface(entry: &EntryV1, instance: &ash::Instance<V1_0>, window: &winit::Window) -> SurfaceStruct {

        let surface = unsafe {
            utility::create_surface(entry, instance, window)
                .expect("Failed to create surface.")
        };
        let surface_loader = ash::extensions::Surface::new(entry, instance)
            .expect("Unable to load the Surface extension");

        SurfaceStruct {
            surface_loader,
            surface
        }
    }

    fn pick_physical_device(instance: &ash::Instance<V1_0>, surface_struct: &SurfaceStruct) -> vk::PhysicalDevice {
        let physical_devices = instance.enumerate_physical_devices()
            .expect("Physical device error");

        let mut result = None;
        for physical_device in physical_devices.iter() {
            if VulkanApp::is_physical_device_suitable(instance, physical_device, surface_struct) {
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

    fn is_physical_device_suitable(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice, surface_struct: &SurfaceStruct) -> bool {

        let _device_properties = instance.get_physical_device_properties(physical_device.clone());
        let _device_features = instance.get_physical_device_features(physical_device.clone());

        let indices = VulkanApp::find_queue_family(instance, physical_device, surface_struct);

        return indices.is_complete();
    }

    fn create_logical_device(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice, validation: &ValidationInfo, surface_struct: &SurfaceStruct)
        -> (ash::Device<V1_0>, QueueFamilyIndices) {

        let indices = VulkanApp::find_queue_family(instance, physical_device, surface_struct);

        use std::collections::HashSet;
        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(indices.graphics_family as u32);
        unique_queue_families.insert(indices.present_family as u32);

        let queue_priorities = [1.0_f32];
        let mut queue_create_infos = vec![];
        for queue_family in unique_queue_families.iter() {
            let queue_create_info = vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DeviceQueueCreateInfo,
                p_next: ptr::null(),
                flags: Default::default(),
                queue_family_index: *queue_family as u32,
                p_queue_priorities: queue_priorities.as_ptr(),
                queue_count: queue_priorities.len() as u32,
            };
            queue_create_infos.push(queue_create_info);
        }

        let physical_device_features = vk::PhysicalDeviceFeatures {
            ..Default::default() // default just enable no feature.
        };

        let enable_layer_names = validation.get_layers_names();

        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DeviceCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_layer_count: if validation.is_enable { enable_layer_names.len() } else { 0 } as u32,
            pp_enabled_layer_names: if validation.is_enable { enable_layer_names.as_ptr() } else { ptr::null() },
            enabled_extension_count: 0,
            pp_enabled_extension_names: ptr::null(),
            p_enabled_features: &physical_device_features,
        };

        let device: ash::Device<V1_0> = unsafe {
            instance.create_device(physical_device.clone(), &device_create_info, None)
                .expect("Failed to create logical device!")
        };

        (device, indices)
    }

    fn find_queue_family(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice, surface_struct: &SurfaceStruct) -> QueueFamilyIndices {

        let queue_families = instance.get_physical_device_queue_family_properties(physical_device.clone());

        let mut queue_family_indices = QueueFamilyIndices::new();

        let mut index = 0;
        for queue_family in queue_families.iter() {
            use ash::vk::types::{ QueueFlags, QUEUE_GRAPHICS_BIT };
            if queue_family.queue_count > 0 && queue_family.queue_flags.subset(QueueFlags::from(QUEUE_GRAPHICS_BIT)) {
                queue_family_indices.graphics_family = index;
            }

            let is_present_support = surface_struct.surface_loader.get_physical_device_surface_support_khr(physical_device.clone(), index as u32, surface_struct.surface);
            if queue_family.queue_count > 0 && is_present_support {
                queue_family_indices.present_family = index;
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

            self.device.destroy_device(None);
            self.surface_loader.destroy_surface_khr(self.surface, None);

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

