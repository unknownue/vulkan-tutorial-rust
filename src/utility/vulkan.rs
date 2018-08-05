
use ash;
use ash::vk;
use ash::version::{ V1_0, InstanceV1_0, EntryV1_0, DeviceV1_0 };
use winit;

type EntryV1 = ash::Entry<V1_0>;

use std::ptr;
use std::ffi::CString;

use super::debug;

pub struct DeviceExtension {
    pub names: [&'static str; 1],
//    pub raw_names: [*const i8; 1],
}

impl DeviceExtension {
    pub fn get_raw_names(&self) -> Vec<*const i8> {
        self.names.iter()
            .map(|name| super::tools::vk_to_raw_string(*name).as_ptr())
            .collect()
    }
}

pub struct SurfaceStuff {
    pub surface_loader: ash::extensions::Surface,
    pub surface: vk::SurfaceKHR,

    pub screen_width: u32,
    pub screen_height: u32,
}
pub struct SwapChainStuff {
    pub swapchain_loader: ash::extensions::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
}

pub struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub struct QueueFamilyIndices {
    pub graphics_family: i32,
    pub present_family:  i32,
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

    let instance: ash::Instance<V1_0> = unsafe {
        entry.create_instance(&create_info, None)
            .expect("Failed to create instance!")
    };

    instance
}


pub fn create_surface(entry: &EntryV1, instance: &ash::Instance<V1_0>, window: &winit::Window, screen_width: u32, screen_height: u32) -> SurfaceStuff {

    let surface = unsafe {
        super::create_surface(entry, instance, window)
            .expect("Failed to create surface.")
    };
    let surface_loader = ash::extensions::Surface::new(entry, instance)
        .expect("Unable to load the Surface extension");

    SurfaceStuff {
        surface_loader,
        surface,
        screen_width,
        screen_height,
    }
}

pub fn pick_physical_device(instance: &ash::Instance<V1_0>, surface_stuff: &SurfaceStuff, required_device_extensions: &DeviceExtension) -> vk::PhysicalDevice {

    let physical_devices = instance.enumerate_physical_devices()
        .expect("Physical device error");

    let result = physical_devices.iter().find(|physical_device| {
        let swapchain_support = query_swapchain_support(physical_device, surface_stuff);
        is_physical_device_suitable(instance, physical_device, surface_stuff, &swapchain_support, required_device_extensions)
    });

    match result {
        | Some(p_physical_device) => *p_physical_device,
        | None => panic!("Failed to find a suitable GPU!"),
    }
}

pub fn is_physical_device_suitable(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice, surface_stuff: &SurfaceStuff, swapchain_support: &SwapChainSupportDetail, required_device_extensions: &DeviceExtension) -> bool {

    let _device_features = instance.get_physical_device_features(physical_device.clone());

    let indices = find_queue_family(instance, physical_device, surface_stuff);

    let is_queue_family_supported = indices.is_complete();
    let is_device_extension_supported = check_device_extension_support(instance, physical_device, required_device_extensions);
    let is_swapchain_supported = !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty();

    return is_queue_family_supported && is_device_extension_supported && is_swapchain_supported;
}

pub fn create_logical_device(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice, validation: &super::debug::ValidationInfo, device_extensions: &DeviceExtension, surface_stuff: &SurfaceStuff)
    -> (ash::Device<V1_0>, QueueFamilyIndices) {

    let indices = find_queue_family(instance, physical_device, surface_stuff);

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

    // or replace 'device_extensions.get_raw_names()' with '[ash::extension::Swapchain::name().as_ptr()]'
    let enable_extension_names = device_extensions.get_raw_names();

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

pub fn find_queue_family(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice, surface_stuff: &SurfaceStuff) -> QueueFamilyIndices {

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

pub fn check_device_extension_support(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice, device_extensions: &DeviceExtension) -> bool {

    let available_extensions = instance.enumerate_device_extension_properties(physical_device.clone())
        .expect("Failed to get device extension properties.");

    let mut available_extension_names = vec![];

    for extension in available_extensions.iter() {
        let extension_name = super::tools::vk_to_string(&extension.extension_name);

        available_extension_names.push(extension_name);
    }

    use std::collections::HashSet;
    let mut required_extensions = HashSet::new();
    for extension in device_extensions.names.iter() {
        required_extensions.insert(extension.to_string());
    }

    for extension_name in available_extension_names.iter() {
        required_extensions.remove(extension_name);
    }

    return required_extensions.is_empty()
}

pub fn query_swapchain_support(physical_device: &vk::PhysicalDevice, surface_stuff: &SurfaceStuff) -> SwapChainSupportDetail {

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

pub fn create_swapchain(instance: &ash::Instance<V1_0>, device: &ash::Device<V1_0>, physical_device: &vk::PhysicalDevice, surface_stuff: &SurfaceStuff, queue_family: &QueueFamilyIndices) -> SwapChainStuff {

    let swapchain_support = query_swapchain_support(physical_device, surface_stuff);

    let surface_format = choose_swapchain_format(&swapchain_support.formats);
    let present_mode = choose_swapchain_present_mode(&swapchain_support.present_modes);
    let extent = choose_swapchain_extent(&swapchain_support.capabilities, surface_stuff.screen_width, surface_stuff.screen_height);

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

pub fn choose_swapchain_format(available_formats: &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
    use ash::vk::types::*;

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

pub fn choose_swapchain_present_mode(available_present_modes: &Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
    use ash::vk::types::*;

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

pub fn choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR, window_width: u32, window_height: u32) -> vk::Extent2D {
    use ash::vk::types::*;

    if capabilities.current_extent.width != uint32_t::max_value() {

        capabilities.current_extent
    } else {
        use num::clamp;

        vk::Extent2D {
            width: clamp(window_width, capabilities.min_image_extent.width, capabilities.max_image_extent.width),
            height: clamp(window_height, capabilities.min_image_extent.height, capabilities.max_image_extent.height)
        }
    }
}

pub fn create_image_view(device: &ash::Device<V1_0>, surface_format: &vk::Format, images: &Vec<vk::Image>) ->Vec<vk::ImageView> {

    let mut swapchain_imageviews = vec![];

    for image in images.iter() {

        let imageview_create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::ImageViewCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            view_type: vk::ImageViewType::Type2d,
            format: surface_format.clone(),
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::Identity,
                g: vk::ComponentSwizzle::Identity,
                b: vk::ComponentSwizzle::Identity,
                a: vk::ComponentSwizzle::Identity,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::IMAGE_ASPECT_COLOR_BIT,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            image: image.clone(),
        };

        let imageview = unsafe {
            device.create_image_view(&imageview_create_info, None)
                .expect("Failed to create Image View!")
        };
        swapchain_imageviews.push(imageview);
    }

    swapchain_imageviews
}
