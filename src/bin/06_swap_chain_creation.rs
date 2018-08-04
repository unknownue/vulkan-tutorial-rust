
extern crate vulkan_tutorial_rust;
use vulkan_tutorial_rust::{
    utility, // the mod define some fixed functions that have been learned before.
    utility::debug::ValidationInfo,
    utility::vulkan::DeviceExtension,
};

extern crate winit;
extern crate ash;
extern crate num;

use winit::{ Event, EventsLoop, WindowEvent, ControlFlow, VirtualKeyCode };
use ash::vk;
use ash::version::{ V1_0, InstanceV1_0 };
use ash::version::DeviceV1_0;

use std::ptr;
use std::collections::HashSet;

type EntryV1 = ash::Entry<V1_0>;

// Constants
const WINDOW_TITLE: &'static str = "06.Swap Chain Creation";
const WINDOW_WIDTH:  u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: [
        "VK_LAYER_LUNARG_standard_validation",
    ],
};
const DEVICE_EXTENSIONS: DeviceExtension = DeviceExtension {
    names: [vk::VK_KHR_SWAPCHAIN_EXTENSION_NAME],
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

struct SurfaceStuff {
    surface_loader: ash::extensions::Surface,
    surface: vk::SurfaceKHR,
}
struct SwapChainStuff {
    swapchain_loader: ash::extensions::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
}

struct SwapChainSupportDetail {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
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

    swapchain_loader: ash::extensions::Swapchain,
    swapchain: vk::SwapchainKHR,
    _swapchain_images: Vec<vk::Image>,
    _swapchain_format: vk::Format,
    _swapchain_extent: vk::Extent2D,
}

impl VulkanApp {

    pub fn new() -> VulkanApp {

        // init window stuff
        let events_loop = EventsLoop::new();
        let window = utility::window::init_window(&events_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        // init vulkan stuff
        let entry = EntryV1::new().unwrap();
        let instance = utility::vulkan::create_instance(&entry, WINDOW_TITLE, VALIDATION.is_enable, &VALIDATION.required_validation_layers.to_vec());
        let surface_stuff = VulkanApp::create_surface(&entry, &instance, &window);
        let (debug_report_loader, debug_callback) = utility::debug::setup_debug_callback( VALIDATION.is_enable, &entry, &instance);
        let physical_device = VulkanApp::pick_physical_device(&instance, &surface_stuff);
        let (device, family_indices) = VulkanApp::create_logical_device(&instance, &physical_device, &VALIDATION, &surface_stuff);
        let graphics_queue = unsafe { device.get_device_queue(family_indices.graphics_family as u32, 0) };
        let present_queue  = unsafe { device.get_device_queue(family_indices.present_family as u32, 0) };
        let swapchain_stuff = VulkanApp::create_swapchain(&instance, &device, &physical_device, &surface_stuff, &family_indices);

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp {
            // winit stuff
            events_loop,
            _window: window,

            // vulkan stuff
            _entry: entry,
            instance,
            surface: surface_stuff.surface,
            surface_loader: surface_stuff.surface_loader,
            debug_report_loader,
            debug_callback,

            _physical_device: physical_device,
            device,

            _graphics_queue: graphics_queue,
            _present_queue: present_queue,

            swapchain_loader: swapchain_stuff.swapchain_loader,
            swapchain: swapchain_stuff.swapchain,
            _swapchain_format: swapchain_stuff.swapchain_format,
            _swapchain_images: swapchain_stuff.swapchain_images,
            _swapchain_extent: swapchain_stuff.swapchain_extent,
        }
    }

    fn create_surface(entry: &EntryV1, instance: &ash::Instance<V1_0>, window: &winit::Window) -> SurfaceStuff {

        let surface = unsafe {
            utility::create_surface(entry, instance, window)
                .expect("Failed to create surface.")
        };
        let surface_loader = ash::extensions::Surface::new(entry, instance)
            .expect("Unable to load the Surface extension");

        SurfaceStuff {
            surface_loader,
            surface
        }
    }

    fn pick_physical_device(instance: &ash::Instance<V1_0>, surface_stuff: &SurfaceStuff) -> vk::PhysicalDevice {

        let physical_devices = instance.enumerate_physical_devices()
            .expect("Physical device error");

        let result = physical_devices.iter().find(|physical_device| {
            let swapchain_support = VulkanApp::query_swapchain_support(physical_device, surface_stuff);
            VulkanApp::is_physical_device_suitable(instance, physical_device, surface_stuff, &swapchain_support)
        });

        match result {
            | Some(p_physical_device) => *p_physical_device,
            | None => panic!("Failed to find a suitable GPU!"),
        }
    }

    fn is_physical_device_suitable(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice, surface_stuff: &SurfaceStuff, swapchain_support: &SwapChainSupportDetail) -> bool {

        let _device_features = instance.get_physical_device_features(physical_device.clone());

        let indices = VulkanApp::find_queue_family(instance, physical_device, surface_stuff);

        let is_queue_family_supported = indices.is_complete();
        let is_device_extension_supported = VulkanApp::check_device_extension_support(instance, physical_device);
        let is_swapchain_supported = !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty();

        return is_queue_family_supported && is_device_extension_supported && is_swapchain_supported;
    }

    fn create_logical_device(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice, validation: &ValidationInfo, surface_stuff: &SurfaceStuff)
        -> (ash::Device<V1_0>, QueueFamilyIndices) {

        let indices = VulkanApp::find_queue_family(instance, physical_device, surface_stuff);

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

        // or replace 'DEVICE_EXTENSIONS.get_raw_names()' with '[ash::extension::Swapchain::name().as_ptr()]'
        let enable_extension_names = DEVICE_EXTENSIONS.get_raw_names();

        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DeviceCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_layer_count: if validation.is_enable { enable_layer_names.len() } else { 0 } as u32,
            pp_enabled_layer_names: if validation.is_enable { enable_layer_names.as_ptr() } else { ptr::null() },
            enabled_extension_count: enable_extension_names.len() as u32,
            pp_enabled_extension_names: enable_extension_names.as_ptr(),
            p_enabled_features: &physical_device_features,
        };

        let device: ash::Device<V1_0> = unsafe {
            instance.create_device(physical_device.clone(), &device_create_info, None)
                .expect("Failed to create logical device!")
        };

        (device, indices)
    }

    fn find_queue_family(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice, surface_stuff: &SurfaceStuff) -> QueueFamilyIndices {

        let queue_families = instance.get_physical_device_queue_family_properties(physical_device.clone());

        let mut queue_family_indices = QueueFamilyIndices::new();

        let mut index = 0;
        for queue_family in queue_families.iter() {
            use ash::vk::types::{ QueueFlags, QUEUE_GRAPHICS_BIT };
            if queue_family.queue_count > 0 && queue_family.queue_flags.subset(QueueFlags::from(QUEUE_GRAPHICS_BIT)) {
                queue_family_indices.graphics_family = index;
            }

            let is_present_support = surface_stuff.surface_loader.get_physical_device_surface_support_khr(physical_device.clone(), index as u32, surface_stuff.surface);
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

    fn check_device_extension_support(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice) -> bool {

        let available_extensions = instance.enumerate_device_extension_properties(physical_device.clone())
            .expect("Failed to get device extension properties.");

        let mut available_extension_names = vec![];

        println!("\tAvailable Device Extensions: ");
        for extension in available_extensions.iter() {
            let extension_name = utility::tools::vk_to_string(&extension.extension_name);
            println!("\t\tName: {}, Version: {}", extension_name, extension.spec_version);

            available_extension_names.push(extension_name);
        }

        let mut required_extensions = HashSet::new();
        for extension in DEVICE_EXTENSIONS.names.iter() {
            required_extensions.insert(extension.to_string());
        }

        for extension_name in available_extension_names.iter() {
            required_extensions.remove(extension_name);
        }

        return required_extensions.is_empty()
    }

    fn query_swapchain_support(physical_device: &vk::PhysicalDevice, surface_stuff: &SurfaceStuff) -> SwapChainSupportDetail {

        let capabilities = surface_stuff.surface_loader.get_physical_device_surface_capabilities_khr(physical_device.clone(), surface_stuff.surface)
            .expect("Failed to query for surface capabilities.");
        let formats = surface_stuff.surface_loader.get_physical_device_surface_formats_khr(physical_device.clone(), surface_stuff.surface)
            .expect("Failed to query for surface formats.");
        let present_modes = surface_stuff.surface_loader.get_physical_device_surface_present_modes_khr(physical_device.clone(), surface_stuff.surface)
            .expect("Failed to query for surface present mode.");

        SwapChainSupportDetail {
            capabilities,
            formats,
            present_modes,
        }
    }

    fn create_swapchain(instance: &ash::Instance<V1_0>, device: &ash::Device<V1_0>, physical_device: &vk::PhysicalDevice, surface_stuff: &SurfaceStuff, queue_family: &QueueFamilyIndices) -> SwapChainStuff {

        let swapchain_support = VulkanApp::query_swapchain_support(physical_device, surface_stuff);

        let surface_format = VulkanApp::choose_swapchain_format(&swapchain_support.formats);
        let present_mode = VulkanApp::choose_swapchain_present_mode(&swapchain_support.present_modes);
        let extent = VulkanApp::choose_swapchain_extent(&swapchain_support.capabilities);

        use std::cmp::min;
        let image_count = min(swapchain_support.capabilities.min_image_count + 1, swapchain_support.capabilities.max_image_count);

        let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
            if queue_family.graphics_family != queue_family.present_family {
                (vk::SharingMode::Concurrent, 2, vec![queue_family.graphics_family as u32, queue_family.present_family as u32])
        } else {
                (vk::SharingMode::Exclusive, 0, vec![])
        };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SwapchainCreateInfoKhr,
            p_next: ptr::null(),
            flags: Default::default(),
            surface: surface_stuff.surface,
            min_image_count: image_count,
            image_color_space: surface_format.color_space,
            image_format: surface_format.format,
            image_extent: extent,
            image_usage: vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
            image_sharing_mode,
            p_queue_family_indices: queue_family_indices.as_ptr(),
            queue_family_index_count,
            pre_transform: swapchain_support.capabilities.current_transform,
            composite_alpha: vk::COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            present_mode,
            clipped: vk::VK_TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
            image_array_layers: 1,
        };

        let swapchain_loader = ash::extensions::Swapchain::new(instance, device)
            .expect("Unable to load Swapchain.");
        let swapchain = unsafe {
            swapchain_loader.create_swapchain_khr(&swapchain_create_info, None)
                .expect("Failed to create Swapchain!")
        };

        let swapchain_images = swapchain_loader.get_swapchain_images_khr(swapchain)
            .expect("Failed to get Swapchain Images.");

        SwapChainStuff {
            swapchain_loader,
            swapchain,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
            swapchain_images,
        }
    }

    fn choose_swapchain_format(available_formats: &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
        use vk::types::*;

        if available_formats.len() == 1 && available_formats[0].format == Format::Undefined {
            return vk::SurfaceFormatKHR {
                format: Format::B8g8r8a8Unorm,
                color_space: ColorSpaceKHR::SrgbNonlinear
            };
        }

        for available_format in available_formats {
            if available_format.format == Format::B8g8r8a8Unorm && available_format.color_space == ColorSpaceKHR::SrgbNonlinear {
                return available_format.clone()
            }
        }

        return available_formats.first().unwrap().clone()
    }

    fn choose_swapchain_present_mode(available_present_modes: &Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
        use vk::types::*;

        let mut best_mode = PresentModeKHR::Fifo;

        for available_present_mode in available_present_modes.iter() {
            if *available_present_mode == PresentModeKHR::Mailbox {
                return available_present_mode.clone()
            } else if *available_present_mode == PresentModeKHR::Immediate {
                best_mode = available_present_mode.clone();
            }
        }

        best_mode
    }

    fn choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        use vk::types::*;

        if capabilities.current_extent.width != uint32_t::max_value() {

            capabilities.current_extent
        } else {
            use num::clamp;

            vk::Extent2D {
                width: clamp(WINDOW_WIDTH, capabilities.min_image_extent.width, capabilities.max_image_extent.width),
                height: clamp(WINDOW_HEIGHT, capabilities.min_image_extent.height, capabilities.max_image_extent.height)
            }
        }
    }
}

impl Drop for VulkanApp {

    fn drop(&mut self) {

        unsafe {

            self.swapchain_loader.destroy_swapchain_khr(self.swapchain, None);
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
