
//! Split reduplicate functions in this share module

pub mod v1;
pub mod v2;

use ash;
use ash::vk;
use ash::version::{ V1_0, InstanceV1_0, EntryV1_0, DeviceV1_0 };
use ash::vk::types::uint32_t;
use winit;
use tobj;

type EntryV1 = ash::Entry<V1_0>;

use std::ptr;
use std::ffi::CString;
use std::path::Path;

use ::utility::{
    debug,
    structures::*,
    constants::*,
    platforms
};

pub fn create_instance(entry: &ash::Entry<V1_0>, window_title: &str, is_enable_debug: bool, required_validation_layers: &Vec<&str>) -> ash::Instance<V1_0> {

    if is_enable_debug && debug::check_validation_layer_support(entry, required_validation_layers) == false {
        panic!("Validation layers requested, but not available!");
    }

    let app_name = CString::new(window_title).unwrap();
    let engine_name = CString::new("Vulkan Engine").unwrap();
    let app_info = vk::ApplicationInfo {
        p_application_name  : app_name.as_ptr(),
        s_type              : vk::StructureType::ApplicationInfo,
        p_next              : ptr::null(),
        application_version : APPLICATION_VERSION,
        p_engine_name       : engine_name.as_ptr(),
        engine_version      : ENGINE_VERSION,
        api_version         : API_VERSION,
    };

    // VK_EXT debug report has been requested here.
    let extension_names = platforms::required_extension_names();

    let requred_validation_layer_raw_names: Vec<CString> = required_validation_layers.iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();
    let layer_names: Vec<*const i8> = requred_validation_layer_raw_names.iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let create_info = vk::InstanceCreateInfo {
        s_type                     : vk::StructureType::InstanceCreateInfo,
        p_next                     : ptr::null(),
        flags                      : vk::InstanceCreateFlags::empty(),
        p_application_info         : &app_info,
        pp_enabled_layer_names     : if is_enable_debug { layer_names.as_ptr() } else { ptr::null() },
        enabled_layer_count        : if is_enable_debug { layer_names.len() } else { 0 } as u32,
        pp_enabled_extension_names : extension_names.as_ptr(),
        enabled_extension_count    : extension_names.len() as u32,
    };

    let instance: ash::Instance<V1_0> = unsafe {
        entry.create_instance(&create_info, None)
            .expect("Failed to create instance!")
    };

    instance
}

pub fn create_surface(entry: &EntryV1, instance: &ash::Instance<V1_0>, window: &winit::Window, screen_width: u32, screen_height: u32) -> SurfaceStuff {

    let surface = unsafe {
        platforms::create_surface(entry, instance, window)
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
        .expect("Failed to enumerate Physical Devices!");

    let result = physical_devices.iter().find(|physical_device| {
        let swapchain_support = query_swapchain_support(**physical_device, surface_stuff);
        let is_suitable = is_physical_device_suitable(instance, **physical_device, surface_stuff, &swapchain_support, required_device_extensions);

        // if is_suitable {
        //     let device_properties = instance.get_physical_device_properties(**physical_device);
        //     let device_name = super::tools::vk_to_string(&device_properties.device_name);
        //     println!("Using GPU: {}", device_name);
        // }

        is_suitable
    });

    match result {
        | Some(p_physical_device) => *p_physical_device,
        | None => panic!("Failed to find a suitable GPU!"),
    }
}

pub fn is_physical_device_suitable(instance: &ash::Instance<V1_0>, physical_device: vk::PhysicalDevice, surface_stuff: &SurfaceStuff, swapchain_support: &SwapChainSupportDetail, required_device_extensions: &DeviceExtension) -> bool {

    let device_features = instance.get_physical_device_features(physical_device);

    let indices = find_queue_family(instance, physical_device, surface_stuff);

    let is_queue_family_supported = indices.is_complete();
    let is_device_extension_supported = check_device_extension_support(instance, physical_device, required_device_extensions);
    let is_swapchain_supported = !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty();
    let is_support_sampler_anisotropy = device_features.sampler_anisotropy == 1;

    return is_queue_family_supported && is_device_extension_supported && is_swapchain_supported && is_support_sampler_anisotropy;
}

pub fn create_logical_device(instance: &ash::Instance<V1_0>, physical_device: vk::PhysicalDevice, validation: &super::debug::ValidationInfo, device_extensions: &DeviceExtension, surface_stuff: &SurfaceStuff)
                             -> (ash::Device<V1_0>, QueueFamilyIndices) {

    let indices = find_queue_family(instance, physical_device, surface_stuff);

    use std::collections::HashSet;
    let mut unique_queue_families = HashSet::new();
    unique_queue_families.insert(indices.graphics_family as u32);
    unique_queue_families.insert(indices.present_family as u32);

    let queue_priorities = [1.0_f32];
    let mut queue_create_infos = vec![];
    for &queue_family in unique_queue_families.iter() {
        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type             : vk::StructureType::DeviceQueueCreateInfo,
            p_next             : ptr::null(),
            flags              : vk::DeviceQueueCreateFlags::empty(),
            queue_family_index : queue_family,
            p_queue_priorities : queue_priorities.as_ptr(),
            queue_count        : queue_priorities.len() as u32,
        };
        queue_create_infos.push(queue_create_info);
    }

    let physical_device_features = vk::PhysicalDeviceFeatures {
        sampler_anisotropy: vk::VK_TRUE, // enable anisotropy device feature from Chapter-24.
        ..Default::default()
    };

    let enable_layer_names = validation.get_layers_names();

    let enable_extension_names = device_extensions.get_extensions_raw_names();

    let device_create_info = vk::DeviceCreateInfo {
        s_type                     : vk::StructureType::DeviceCreateInfo,
        p_next                     : ptr::null(),
        flags                      : vk::DeviceCreateFlags::empty(),
        queue_create_info_count    : queue_create_infos.len() as u32,
        p_queue_create_infos       : queue_create_infos.as_ptr(),
        enabled_layer_count        : if validation.is_enable { enable_layer_names.len() } else { 0 } as u32,
        pp_enabled_layer_names     : if validation.is_enable { enable_layer_names.as_ptr() } else { ptr::null() },
        enabled_extension_count    : enable_extension_names.len() as u32,
        pp_enabled_extension_names : enable_extension_names.as_ptr(),
        p_enabled_features         : &physical_device_features,
    };

    let device: ash::Device<V1_0> = unsafe {
        instance.create_device(physical_device, &device_create_info, None)
            .expect("Failed to create logical Device!")
    };

    (device, indices)
}

pub fn find_queue_family(instance: &ash::Instance<V1_0>, physical_device: vk::PhysicalDevice, surface_stuff: &SurfaceStuff) -> QueueFamilyIndices {

    let queue_families = instance.get_physical_device_queue_family_properties(physical_device);

    let mut queue_family_indices = QueueFamilyIndices::new();

    let mut index = 0;
    for queue_family in queue_families.iter() {

        if queue_family.queue_count > 0 && queue_family.queue_flags.subset(vk::QUEUE_GRAPHICS_BIT) {
            queue_family_indices.graphics_family = index;
        }

        let is_present_support = surface_stuff.surface_loader.get_physical_device_surface_support_khr(physical_device, index as u32, surface_stuff.surface);
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

pub fn check_device_extension_support(instance: &ash::Instance<V1_0>, physical_device: vk::PhysicalDevice, device_extensions: &DeviceExtension) -> bool {

    let available_extensions = instance.enumerate_device_extension_properties(physical_device)
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

pub fn query_swapchain_support(physical_device: vk::PhysicalDevice, surface_stuff: &SurfaceStuff) -> SwapChainSupportDetail {

    let capabilities = surface_stuff.surface_loader.get_physical_device_surface_capabilities_khr(physical_device, surface_stuff.surface)
        .expect("Failed to query for surface capabilities.");
    let formats = surface_stuff.surface_loader.get_physical_device_surface_formats_khr(physical_device, surface_stuff.surface)
        .expect("Failed to query for surface formats.");
    let present_modes = surface_stuff.surface_loader.get_physical_device_surface_present_modes_khr(physical_device, surface_stuff.surface)
        .expect("Failed to query for surface present mode.");

    SwapChainSupportDetail {
        capabilities,
        formats,
        present_modes,
    }
}

pub fn create_swapchain(instance: &ash::Instance<V1_0>, device: &ash::Device<V1_0>, physical_device: vk::PhysicalDevice, window: &winit::Window, surface_stuff: &SurfaceStuff, queue_family: &QueueFamilyIndices) -> SwapChainStuff {

    let swapchain_support = query_swapchain_support(physical_device, surface_stuff);

    let surface_format = choose_swapchain_format(&swapchain_support.formats);
    let present_mode = choose_swapchain_present_mode(&swapchain_support.present_modes);
    let extent = choose_swapchain_extent(&swapchain_support.capabilities, window);

    use std::cmp::min;
    let image_count = min(swapchain_support.capabilities.min_image_count + 1, swapchain_support.capabilities.max_image_count);

    let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
        if queue_family.graphics_family != queue_family.present_family {
            (vk::SharingMode::Concurrent, 2, vec![queue_family.graphics_family as u32, queue_family.present_family as u32])
        } else {
            (vk::SharingMode::Exclusive, 0, vec![])
        };

    let swapchain_create_info = vk::SwapchainCreateInfoKHR {
        s_type                   : vk::StructureType::SwapchainCreateInfoKhr,
        p_next                   : ptr::null(),
        flags                    : vk::SwapchainCreateFlagsKHR::empty(),
        surface                  : surface_stuff.surface,
        min_image_count          : image_count,
        image_color_space        : surface_format.color_space,
        image_format             : surface_format.format,
        image_extent             : extent,
        image_usage              : vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
        image_sharing_mode,
        p_queue_family_indices   : queue_family_indices.as_ptr(),
        queue_family_index_count,
        pre_transform            : swapchain_support.capabilities.current_transform,
        composite_alpha          : vk::COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
        present_mode,
        clipped                  : vk::VK_TRUE,
        old_swapchain            : vk::SwapchainKHR::null(),
        image_array_layers       : 1,
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
            format      : Format::B8g8r8a8Unorm,
            color_space : ColorSpaceKHR::SrgbNonlinear
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

    let mut best_mode = vk::PresentModeKHR::Fifo;

    for &available_present_mode in available_present_modes.iter() {
        if available_present_mode == vk::PresentModeKHR::Mailbox {
            return available_present_mode
        } else if available_present_mode == vk::PresentModeKHR::Immediate {
            best_mode = available_present_mode;
        }
    }

    best_mode
}

pub fn choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR, window: &winit::Window) -> vk::Extent2D {

    if capabilities.current_extent.width != uint32_t::max_value() {

        capabilities.current_extent
    } else {
        use num::clamp;

        let window_size = window.get_inner_size()
            .expect("Failed to get the size of Window");
        println!("\t\tInner Window Size: ({}, {})", window_size.width, window_size.height);

        vk::Extent2D {
            width:  clamp(window_size.width as u32, capabilities.min_image_extent.width, capabilities.max_image_extent.width),
            height: clamp(window_size.height as u32, capabilities.min_image_extent.height, capabilities.max_image_extent.height)
        }
    }
}

pub fn create_shader_module(device: &ash::Device<V1_0>, code: Vec<u8>) -> vk::ShaderModule {

    let shader_module_create_info = vk::ShaderModuleCreateInfo {
        s_type    : vk::StructureType::ShaderModuleCreateInfo,
        p_next    : ptr::null(),
        flags     : vk::ShaderModuleCreateFlags::empty(),
        code_size : code.len(),
        p_code    : code.as_ptr() as *const u32,
    };

    unsafe {
        device.create_shader_module(&shader_module_create_info, None)
            .expect("Failed to create Shader Module!")
    }
}

pub fn create_buffer(device: &ash::Device<V1_0>, size: vk::DeviceSize, usage: vk::BufferUsageFlags, required_memory_properties: vk::MemoryPropertyFlags, device_memory_properties: &vk::PhysicalDeviceMemoryProperties)
                     -> (vk::Buffer, vk::DeviceMemory) {

    let buffer_create_info = vk::BufferCreateInfo {
        s_type                   : vk::StructureType::BufferCreateInfo,
        p_next                   : ptr::null(),
        flags                    : vk::BufferCreateFlags::empty(),
        size,
        usage,
        sharing_mode             : vk::SharingMode::Exclusive,
        queue_family_index_count : 0,
        p_queue_family_indices   : ptr::null(),
    };

    let buffer = unsafe {
        device.create_buffer(&buffer_create_info, None)
            .expect("Failed to create Vertex Buffer")
    };

    let mem_requirements = device.get_buffer_memory_requirements(buffer);
    let memory_type = find_memory_type(mem_requirements.memory_type_bits, required_memory_properties, device_memory_properties);

    let allocate_info = vk::MemoryAllocateInfo {
        s_type            : vk::StructureType::MemoryAllocateInfo,
        p_next            : ptr::null(),
        allocation_size   : mem_requirements.size,
        memory_type_index : memory_type,
    };

    let buffer_memory = unsafe {
        device.allocate_memory(&allocate_info, None)
            .expect("Failed to allocate vertex buffer memory!")
    };

    unsafe {
        device.bind_buffer_memory(buffer, buffer_memory, 0)
            .expect("Failed to bind Buffer");
    }

    (buffer, buffer_memory)
}

pub fn copy_buffer(device: &ash::Device<V1_0>, submit_queue: vk::Queue, command_pool: vk::CommandPool, src_buffer: vk::Buffer, dst_buffer: vk::Buffer, size: vk::DeviceSize) {

    let command_buffer = begin_single_time_command(device, command_pool);

    let copy_regions = [
        vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        },
    ];

    unsafe {
        device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &copy_regions);
    }

    end_single_time_command(device, command_pool, submit_queue, command_buffer);
}

pub fn begin_single_time_command(device: &ash::Device<V1_0>, command_pool: vk::CommandPool) -> vk::CommandBuffer {

    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
        s_type               : vk::StructureType::CommandBufferAllocateInfo,
        p_next               : ptr::null(),
        command_buffer_count : 1,
        command_pool,
        level                : vk::CommandBufferLevel::Primary,
    };

    let command_buffer = unsafe {
        device.allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    }[0];

    let command_buffer_begin_info  = vk::CommandBufferBeginInfo {
        s_type             : vk::StructureType::CommandBufferBeginInfo,
        p_next             : ptr::null(),
        p_inheritance_info : ptr::null(),
        flags              : vk::COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
    };

    unsafe {
        device.begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("Failed to begin recording Command Buffer at beginning!");
    }

    command_buffer
}

pub fn end_single_time_command(device: &ash::Device<V1_0>, command_pool: vk::CommandPool, submit_queue: vk::Queue, command_buffer: vk::CommandBuffer) {

    unsafe {
        device.end_command_buffer(command_buffer)
            .expect("Failed to record Command Buffer at Ending!");
    }

    let buffers_to_submit = [
        command_buffer,
    ];

    let sumbit_infos = [
        vk::SubmitInfo {
            s_type                 : vk::StructureType::SubmitInfo,
            p_next                 : ptr::null(),
            wait_semaphore_count   : 0,
            p_wait_semaphores      : ptr::null(),
            p_wait_dst_stage_mask  : ptr::null(),
            command_buffer_count   : 1,
            p_command_buffers      : buffers_to_submit.as_ptr(),
            signal_semaphore_count : 0,
            p_signal_semaphores    : ptr::null(),
        },
    ];

    unsafe {
        device.queue_submit(submit_queue, &sumbit_infos, vk::Fence::null())
            .expect("Failed to Queue Submit!");
        device.queue_wait_idle(submit_queue)
            .expect("Failed to wait Queue idle!");
        device.free_command_buffers(command_pool, &buffers_to_submit);
    }
}

pub fn find_memory_type(type_filter: uint32_t, required_properties: vk::MemoryPropertyFlags, mem_properties: &vk::PhysicalDeviceMemoryProperties) -> uint32_t {

    for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
        if (type_filter & (1 << i)) > 0 && memory_type.property_flags.subset(required_properties) {
            return i as uint32_t
        }
    }

    panic!("Failed to find suitable memory type!")
}

pub fn has_stencil_component(format: vk::Format) -> bool {
    format == vk::Format::D32SfloatS8Uint || format == vk::Format::D24UnormS8Uint
}

pub fn copy_buffer_to_image(device: &ash::Device<V1_0>, command_pool: vk::CommandPool, submit_queue: vk::Queue, buffer: vk::Buffer, image: vk::Image, width: uint32_t, height: uint32_t) {

    let command_buffer = begin_single_time_command(device, command_pool);

    let buffer_image_regions = [
        vk::BufferImageCopy {
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask      : vk::IMAGE_ASPECT_COLOR_BIT,
                mip_level        : 0,
                base_array_layer : 0,
                layer_count      : 1,
            },
            image_extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            buffer_offset       : 0,
            buffer_image_height : 0,
            buffer_row_length   : 0,
            image_offset        : vk::Offset3D { x: 0, y: 0, z: 0 },
        },
    ];

    unsafe {
        device.cmd_copy_buffer_to_image(command_buffer, buffer, image, vk::ImageLayout::TransferDstOptimal, &buffer_image_regions);
    }

    end_single_time_command(device, command_pool, submit_queue, command_buffer);
}

pub fn find_depth_format(instance: &ash::Instance<V1_0>, physical_device: vk::PhysicalDevice) -> vk::Format {
    find_supported_format(
        instance, physical_device,
        &[
            vk::Format::D32Sfloat,
            vk::Format::D32SfloatS8Uint,
            vk::Format::D24UnormS8Uint,
        ],
        vk::ImageTiling::Optimal,
        vk::FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT
    )
}

pub fn find_supported_format(instance: &ash::Instance<V1_0>, physical_device: vk::PhysicalDevice, candidate_formats: &[vk::Format], tiling: vk::ImageTiling, features: vk::FormatFeatureFlags) -> vk::Format {

    for &format in candidate_formats.iter() {

        let format_properties = instance.get_physical_device_format_properties(physical_device, format);
        if tiling == vk::ImageTiling::Linear && format_properties.linear_tiling_features.subset(features) {
            return format.clone()
        } else if tiling == vk::ImageTiling::Optimal && format_properties.optimal_tiling_features.subset(features) {
            return format.clone()
        }
    }

    panic!("Failed to find supported format!")
}

pub fn load_model(model_path: &Path) -> (Vec<VertexV3>, Vec<vk::uint32_t>) {

    let model_obj = tobj::load_obj(model_path)
        .expect("Failed to load model object!");

    let mut vertices = vec![];
    let mut indices  = vec![];

    let (models, _) = model_obj;
    for m in models.iter() {
        let mesh = &m.mesh;

        if mesh.texcoords.len() == 0 {
            panic!("Missing texture coordinate for the model.")
        }

        let total_vertices_count = mesh.positions.len() / 3;
        for i in 0..total_vertices_count {
            let vertex = VertexV3 {
                pos: [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                    1.0,
                ],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coord: [
                    mesh.texcoords[i * 2],
                    mesh.texcoords[i * 2 + 1],
                ],
            };
            vertices.push(vertex);
        }

        indices = mesh.indices.clone();
    }

    (vertices, indices)
}

pub fn check_mipmap_support(instance: &ash::Instance<V1_0>, physcial_device: vk::PhysicalDevice, image_format: vk::Format) {

    let format_properties = instance.get_physical_device_format_properties(physcial_device, image_format);

    let is_sample_image_filter_linear_support = format_properties.optimal_tiling_features.subset(vk::FORMAT_FEATURE_SAMPLED_IMAGE_FILTER_LINEAR_BIT);

    if is_sample_image_filter_linear_support == false {
        panic!("Texture Image format does not support linear blitting!")
    }
}
