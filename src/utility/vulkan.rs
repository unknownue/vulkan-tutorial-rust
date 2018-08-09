
use ash;
use ash::vk;
use ash::version::{ V1_0, InstanceV1_0, EntryV1_0, DeviceV1_0 };
use ash::vk::types::uint32_t;
use winit;

type EntryV1 = ash::Entry<V1_0>;

use std::ptr;
use std::ffi::CString;
use std::path::Path;

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

pub struct SyncObjects {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub inflight_fences: Vec<vk::Fence>,
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
        let is_suitable = is_physical_device_suitable(instance, physical_device, surface_stuff, &swapchain_support, required_device_extensions);

        if is_suitable {
            let device_properties = instance.get_physical_device_properties(**physical_device);
            let device_name = super::tools::vk_to_string(&device_properties.device_name);
            println!("Using GPU: {}", device_name);
        }

        is_suitable
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

pub fn create_swapchain(instance: &ash::Instance<V1_0>, device: &ash::Device<V1_0>, physical_device: &vk::PhysicalDevice, window: &winit::Window, surface_stuff: &SurfaceStuff, queue_family: &QueueFamilyIndices) -> SwapChainStuff {

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

pub fn choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR, window: &winit::Window) -> vk::Extent2D {
    use ash::vk::types::*;

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

pub fn create_shader_module(device: &ash::Device<V1_0>, code: Vec<u8>) -> vk::ShaderModule {
    let vertex_shader_info = vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::ShaderModuleCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        code_size: code.len(),
        p_code: code.as_ptr() as *const u32,
    };

    unsafe {
        device.create_shader_module(&vertex_shader_info, None)
            .expect("Failed to create Shader Module!")
    }
}


pub fn create_render_pass(device: &ash::Device<V1_0>, surface_format: &vk::Format) -> vk::RenderPass {

    let color_attachment = vk::AttachmentDescription {
        format: surface_format.clone(),
        flags: vk::AttachmentDescriptionFlags::empty(),
        samples: vk::SAMPLE_COUNT_1_BIT,
        load_op: vk::AttachmentLoadOp::Clear,
        store_op: vk::AttachmentStoreOp::Store,
        stencil_load_op: vk::AttachmentLoadOp::DontCare,
        stencil_store_op: vk::AttachmentStoreOp::DontCare,
        initial_layout: vk::ImageLayout::Undefined,
        final_layout: vk::ImageLayout::PresentSrcKhr,
    };

    let color_attachment_ref = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::ColorAttachmentOptimal,
    };

    let subpass = vk::SubpassDescription {
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_ref,
        p_depth_stencil_attachment: ptr::null(),
        flags: Default::default(),
        pipeline_bind_point: vk::PipelineBindPoint::Graphics,
        input_attachment_count: 0,
        p_input_attachments: ptr::null(),
        p_resolve_attachments: ptr::null(),
        preserve_attachment_count: 0,
        p_preserve_attachments: ptr::null(),
    };

    let render_pass_attachments = [
        color_attachment,
    ];

    let renderpass_create_info = vk::RenderPassCreateInfo {
        s_type: vk::StructureType::RenderPassCreateInfo,
        flags: Default::default(),
        p_next: ptr::null(),
        attachment_count: render_pass_attachments.len() as u32,
        p_attachments: render_pass_attachments.as_ptr(),
        subpass_count: 1,
        p_subpasses: &subpass,
        dependency_count: 0,
        p_dependencies: ptr::null(),
    };

    unsafe {
        device.create_render_pass(&renderpass_create_info, None)
            .expect("Failed to create render pass!")
    }
}

pub fn create_graphics_pipeline(device: &ash::Device<V1_0>, render_pass: &vk::RenderPass, swapchain_extent: &vk::Extent2D) -> (vk::Pipeline, vk::PipelineLayout) {

    let vert_shader_code = super::tools::read_shader_code(Path::new("shaders/spv/09-shader-base.vert.spv"));
    let frag_shader_code = super::tools::read_shader_code(Path::new("shaders/spv/09-shader-base.frag.spv"));

    let vert_shader_module = create_shader_module(device, vert_shader_code);
    let frag_shader_module = create_shader_module(device, frag_shader_code);

    let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

    let vert_shader_create_info = vk::PipelineShaderStageCreateInfo {
        s_type: vk::StructureType::PipelineShaderStageCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        module: vert_shader_module,
        p_name: main_function_name.as_ptr(),
        p_specialization_info: ptr::null(),
        stage: vk::SHADER_STAGE_VERTEX_BIT,
    };

    let frag_shader_create_info = vk::PipelineShaderStageCreateInfo {
        s_type: vk::StructureType::PipelineShaderStageCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        module: frag_shader_module,
        p_name: main_function_name.as_ptr(),
        p_specialization_info: ptr::null(),
        stage: vk::SHADER_STAGE_FRAGMENT_BIT,
    };

    let shader_stages = [
        vert_shader_create_info,
        frag_shader_create_info,
    ];

    let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PipelineVertexInputStateCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        vertex_attribute_description_count: 0,
        p_vertex_attribute_descriptions: ptr::null(),
        vertex_binding_description_count: 0,
        p_vertex_binding_descriptions: ptr::null(),
    };
    let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
        s_type: vk::StructureType::PipelineInputAssemblyStateCreateInfo,
        flags: Default::default(),
        p_next: ptr::null(),
        primitive_restart_enable: vk::VK_FALSE,
        topology: vk::PrimitiveTopology::TriangleList,
    };

    let viewports = [
        vk::Viewport {
            x: 0.0,
            y: 0.0,
            width:  swapchain_extent.width as f32,
            height: swapchain_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        },
    ];

    let scissors = [
        vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent.clone(),
        },
    ];

    let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PipelineViewportStateCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        scissor_count: scissors.len() as u32,
        p_scissors: scissors.as_ptr(),
        viewport_count: viewports.len() as u32,
        p_viewports: viewports.as_ptr(),
    };

    let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PipelineRasterizationStateCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        depth_clamp_enable: vk::VK_FALSE,
        cull_mode: vk::CULL_MODE_BACK_BIT,
        front_face: vk::FrontFace::Clockwise,
        line_width: 1.0,
        polygon_mode: vk::PolygonMode::Fill,
        rasterizer_discard_enable: vk::VK_FALSE,
        depth_bias_clamp: 0.0,
        depth_bias_constant_factor: 0.0,
        depth_bias_enable: vk::VK_FALSE,
        depth_bias_slope_factor: 0.0,
    };
    let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PipelineMultisampleStateCreateInfo,
        flags: Default::default(),
        p_next: ptr::null(),
        rasterization_samples: vk::SAMPLE_COUNT_1_BIT,
        sample_shading_enable: vk::VK_FALSE,
        min_sample_shading: 0.0,
        p_sample_mask: ptr::null(),
        alpha_to_one_enable: 0,
        alpha_to_coverage_enable: 0,
    };

    let stencil_state = vk::StencilOpState {
        fail_op: vk::StencilOp::Keep,
        pass_op: vk::StencilOp::Keep,
        depth_fail_op: vk::StencilOp::Keep,
        compare_op: vk::CompareOp::Always,
        compare_mask: 0,
        write_mask: 0,
        reference: 0,
    };

    let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo {
        s_type: vk::StructureType::PipelineDepthStencilStateCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        depth_test_enable: vk::VK_FALSE,
        depth_write_enable: vk::VK_FALSE,
        depth_compare_op: vk::CompareOp::LessOrEqual,
        depth_bounds_test_enable: vk::VK_FALSE,
        stencil_test_enable: vk::VK_FALSE,
        front: stencil_state.clone(),
        back:  stencil_state.clone(),
        max_depth_bounds: 1.0,
        min_depth_bounds: 0.0,
    };

    let color_blend_attachment_states = [
        vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::VK_FALSE,
            color_write_mask: vk::ColorComponentFlags::all(),
            src_color_blend_factor: vk::BlendFactor::One,
            dst_color_blend_factor: vk::BlendFactor::Zero,
            color_blend_op: vk::BlendOp::Add,
            src_alpha_blend_factor: vk::BlendFactor::One,
            dst_alpha_blend_factor: vk::BlendFactor::Zero,
            alpha_blend_op: vk::BlendOp::Add,
        },
    ];

    let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PipelineColorBlendStateCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        logic_op_enable: vk::VK_FALSE,
        logic_op: vk::LogicOp::Copy,
        attachment_count: color_blend_attachment_states.len() as u32,
        p_attachments: color_blend_attachment_states.as_ptr(),
        blend_constants: [0.0, 0.0, 0.0, 0.0],
    };

//        leaving the dynamic statue unconfigurated right now
//        let dynamic_state = [vk::DynamicState::Viewport, vk::DynamicState::Scissor];
//        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
//            s_type: vk::StructureType::PipelineDynamicStateCreateInfo,
//            p_next: ptr::null(),
//            flags: Default::default(),
//            dynamic_state_count: dynamic_state.len() as u32,
//            p_dynamic_states: dynamic_state.as_ptr(),
//        };

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PipelineLayoutCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        set_layout_count: 0,
        p_set_layouts: ptr::null(),
        push_constant_range_count: 0,
        p_push_constant_ranges: ptr::null(),
    };

    let pipeline_layout = unsafe {
        device.create_pipeline_layout(&pipeline_layout_create_info, None)
            .expect("Failed to create pipeline layout!")
    };


    let graphic_pipeline_create_infos = [
        vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GraphicsPipelineCreateInfo,
            p_next: ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: shader_stages.len() as u32,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_state_create_info,
            p_input_assembly_state: &vertex_input_assembly_state_info,
            p_tessellation_state: ptr::null(),
            p_viewport_state: &viewport_state_create_info,
            p_rasterization_state: &rasterization_statue_create_info,
            p_multisample_state: &multisample_state_create_info,
            p_depth_stencil_state: &depth_state_create_info,
            p_color_blend_state: &color_blend_state,
            p_dynamic_state: ptr::null(),
            layout: pipeline_layout,
            render_pass: render_pass.clone(),
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
        },
    ];

    let graphics_pipelines = unsafe {
        device.create_graphics_pipelines(vk::PipelineCache::null(), &graphic_pipeline_create_infos, None)
            .expect("Failed to create Graphics Pipeline!.")
    };

    unsafe {
        device.destroy_shader_module(vert_shader_module, None);
        device.destroy_shader_module(frag_shader_module, None);
    }

    (graphics_pipelines[0], pipeline_layout)
}

pub fn create_framebuffers(device: &ash::Device<V1_0>, render_pass: &vk::RenderPass, image_views: &Vec<vk::ImageView>, swapchain_extent: &vk::Extent2D) -> Vec<vk::Framebuffer> {

    let mut framebuffers = vec![];

    for &image_view in image_views.iter() {
        let attachments = [
            image_view
        ];

        let framebuffer_create_info = vk::FramebufferCreateInfo {
            s_type: vk::StructureType::FramebufferCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            render_pass: render_pass.clone(),
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            width:  swapchain_extent.width,
            height: swapchain_extent.height,
            layers: 1,
        };

        let framebuffer = unsafe {
            device.create_framebuffer(&framebuffer_create_info, None)
                .expect("Failed to create Framebuffer!")
        };

        framebuffers.push(framebuffer);
    }

    framebuffers
}

pub fn create_command_pool(device: &ash::Device<V1_0>, queue_families: &QueueFamilyIndices) -> vk::CommandPool {

    let command_pool_create_info = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::CommandPoolCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        queue_family_index: queue_families.graphics_family as u32,
    };

    unsafe {
        device.create_command_pool(&command_pool_create_info, None)
            .expect("Failed to create Command Pool!")
    }
}

pub fn create_command_buffers(device: &ash::Device<V1_0>, command_pool: &vk::CommandPool, graphics_pipeline: &vk::Pipeline, framebuffers: &Vec<vk::Framebuffer>, render_pass: &vk::RenderPass, surface_extent: &vk::Extent2D) -> Vec<vk::CommandBuffer> {

    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::CommandBufferAllocateInfo,
        p_next: ptr::null(),
        command_buffer_count: framebuffers.len() as u32,
        command_pool: command_pool.clone(),
        level: vk::CommandBufferLevel::Primary,
    };

    let command_buffers = unsafe {
        device.allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    };

    for (i, &command_buffer) in command_buffers.iter().enumerate() {

        let command_buffer_begin_info  = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::CommandBufferBeginInfo,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT,
        };

        unsafe {
            device.begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0]
                },
            }
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RenderPassBeginInfo,
            p_next: ptr::null(),
            render_pass: render_pass.clone(),
            framebuffer: framebuffers[i],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: surface_extent.clone(),
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };

        unsafe {
            device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::Inline);
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::Graphics, graphics_pipeline.clone());
            device.cmd_draw(command_buffer, 3, 1, 0, 0);

            device.cmd_end_render_pass(command_buffer);

            device.end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    }

    command_buffers
}

pub fn create_sync_objects(device: &ash::Device<V1_0>, max_frame_in_flight: usize) -> SyncObjects {

    let mut sync_objects = SyncObjects {
        image_available_semaphores: vec![],
        render_finished_semaphores: vec![],
        inflight_fences: vec![],
    };

    let semaphore_create_info = vk::SemaphoreCreateInfo {
        s_type: vk::StructureType::SemaphoreCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
    };

    let fence_create_info = vk::FenceCreateInfo {
        s_type: vk::StructureType::FenceCreateInfo,
        p_next: ptr::null(),
        flags: vk::FENCE_CREATE_SIGNALED_BIT,
    };

    for _ in 0..max_frame_in_flight {
        unsafe {
            let image_available_semaphore = device.create_semaphore(&semaphore_create_info, None)
                .expect("Failed to create Semaphore Object!");
            let render_finished_semaphore = device.create_semaphore(&semaphore_create_info, None)
                .expect("Failed to create Semaphore Object!");
            let inflight_fence = device.create_fence(&fence_create_info, None)
                .expect("Failed to create Fence Object!");

            sync_objects.image_available_semaphores.push(image_available_semaphore);
            sync_objects.render_finished_semaphores.push(render_finished_semaphore);
            sync_objects.inflight_fences.push(inflight_fence);
        }
    }

    sync_objects
}

pub fn create_buffer(device: &ash::Device<V1_0>, size: vk::DeviceSize, usage: vk::BufferUsageFlags, required_memory_properties: vk::MemoryPropertyFlags, device_memory_properties: &vk::PhysicalDeviceMemoryProperties)
    -> (vk::Buffer, vk::DeviceMemory) {

    let buffer_create_info = vk::BufferCreateInfo {
        s_type: vk::StructureType::BufferCreateInfo,
        p_next: ptr::null(),
        flags: vk::BufferCreateFlags::empty(),
        size,
        usage,
        sharing_mode: vk::SharingMode::Exclusive,
        queue_family_index_count: 0,
        p_queue_family_indices: ptr::null(),
    };

    let buffer = unsafe {
        device.create_buffer(&buffer_create_info, None)
            .expect("Failed to create Vertex Buffer")
    };

    let mem_requirements = device.get_buffer_memory_requirements(buffer);
    let memory_type = find_memory_type(mem_requirements.memory_type_bits, required_memory_properties, device_memory_properties);

    let allocate_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MemoryAllocateInfo,
        p_next: ptr::null(),
        allocation_size: mem_requirements.size,
        memory_type_index: memory_type,
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

pub fn copy_buffer(device: &ash::Device<V1_0>, submit_queue: vk::Queue, command_pool: vk::CommandPool, src_buffer: &vk::Buffer, dst_buffer: &vk::Buffer, size: vk::DeviceSize) {

    let allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::CommandBufferAllocateInfo,
        p_next: ptr::null(),
        command_buffer_count: 1,
        command_pool,
        level: vk::CommandBufferLevel::Primary,
    };

    let command_buffers = unsafe {
        device.allocate_command_buffers(&allocate_info)
            .expect("Failed to allocate Command Buffer")
    };
    let command_buffer = command_buffers[0];

    let begin_info = vk::CommandBufferBeginInfo {
        s_type: vk::StructureType::CommandBufferBeginInfo,
        p_next: ptr::null(),
        p_inheritance_info: ptr::null(),
        flags: vk::COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
    };

    unsafe {
        device.begin_command_buffer(command_buffer, &begin_info)
            .expect("Failed to begin Command Buffer");

        let copy_regions = [
            vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size,
            },
        ];

        device.cmd_copy_buffer(command_buffer, src_buffer.clone(), dst_buffer.clone(), &copy_regions);

        device.end_command_buffer(command_buffer)
            .expect("Failed to end Command Buffer");
    }

    let submit_info  = [
        vk::SubmitInfo {
            s_type: vk::StructureType::SubmitInfo,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: ptr::null(),
            p_wait_dst_stage_mask: ptr::null(),
            command_buffer_count: 1,
            p_command_buffers: &command_buffer,
            signal_semaphore_count: 0,
            p_signal_semaphores: ptr::null(),
        }
    ];

    unsafe {
        device.queue_submit(submit_queue, &submit_info, vk::Fence::null())
            .expect("Failed to Submit Queue.");
        device.queue_wait_idle(submit_queue)
            .expect("Failed to wait Queue idle");

        device.free_command_buffers(command_pool, &command_buffers);
    }
}

fn find_memory_type(type_filter: uint32_t, required_properties: vk::MemoryPropertyFlags, mem_properties: &vk::PhysicalDeviceMemoryProperties) -> uint32_t {

    for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
        if (type_filter & (1 << i)) > 0 && (memory_type.property_flags & required_properties) == required_properties {
            return i as uint32_t
        }
    }

    panic!("Failed to find suitable memory type!")
}
