use vulkan_tutorial_rust::{
    utility, // the mod define some fixed functions that have been learned before.
    utility::constants::*,
    utility::debug::*,
    utility::share,
    utility::structures::*,
    utility::window::{ProgramProc, VulkanApp},
};

use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;
use cgmath::{Deg, Matrix4, Point3, Vector3};
use image::GenericImageView;

use std::cmp::max;
use std::ffi::CString;
use std::path::Path;
use std::ptr;

// Constants
const WINDOW_TITLE: &'static str = "28.Mipmapping";
const MODEL_PATH: &'static str = "assets/chalet.obj";
const TEXTURE_PATH: &'static str = "assets/chalet.jpg";

struct VulkanApp28 {
    window: winit::window::Window,

    // vulkan stuff
    _entry: ash::Entry,
    instance: ash::Instance,
    surface_loader: ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_merssager: vk::DebugUtilsMessengerEXT,

    physical_device: vk::PhysicalDevice,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    device: ash::Device,

    queue_family: QueueFamilyIndices,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_imageviews: Vec<vk::ImageView>,
    swapchain_framebuffers: Vec<vk::Framebuffer>,

    render_pass: vk::RenderPass,
    ubo_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,

    depth_image: vk::Image,
    depth_image_view: vk::ImageView,
    depth_image_memory: vk::DeviceMemory,

    _mip_levels: u32,
    texture_image: vk::Image,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,
    texture_image_memory: vk::DeviceMemory,

    _vertices: Vec<VertexV3>,
    indices: Vec<u32>,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,

    uniform_transform: UniformBufferObject,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,

    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,

    is_framebuffer_resized: bool,
}

impl VulkanApp28 {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> VulkanApp28 {
        let window =
            utility::window::init_window(&event_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        // init vulkan stuff
        let entry = ash::Entry::new().unwrap();
        let instance = share::create_instance(
            &entry,
            WINDOW_TITLE,
            VALIDATION.is_enable,
            &VALIDATION.required_validation_layers.to_vec(),
        );
        let surface_stuff =
            share::create_surface(&entry, &instance, &window, WINDOW_WIDTH, WINDOW_HEIGHT);
        let (debug_utils_loader, debug_merssager) =
            setup_debug_utils(VALIDATION.is_enable, &entry, &instance);
        let physical_device =
            share::pick_physical_device(&instance, &surface_stuff, &DEVICE_EXTENSIONS);
        let physical_device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let (device, queue_family) = share::create_logical_device(
            &instance,
            physical_device,
            &VALIDATION,
            &DEVICE_EXTENSIONS,
            &surface_stuff,
        );
        let graphics_queue =
            unsafe { device.get_device_queue(queue_family.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(queue_family.present_family.unwrap(), 0) };
        let swapchain_stuff = share::create_swapchain(
            &instance,
            &device,
            physical_device,
            &window,
            &surface_stuff,
            &queue_family,
        );
        let swapchain_imageviews = share::v1::create_image_views(
            &device,
            swapchain_stuff.swapchain_format,
            &swapchain_stuff.swapchain_images,
        );
        let render_pass = VulkanApp28::create_render_pass(
            &instance,
            &device,
            physical_device,
            swapchain_stuff.swapchain_format,
        );
        let ubo_layout = share::v2::create_descriptor_set_layout(&device);
        let (graphics_pipeline, pipeline_layout) = VulkanApp28::create_graphics_pipeline(
            &device,
            render_pass,
            swapchain_stuff.swapchain_extent,
            ubo_layout,
        );
        let command_pool = share::v1::create_command_pool(&device, &queue_family);
        let (depth_image, depth_image_view, depth_image_memory) = share::v1::create_depth_resources(
            &instance,
            &device,
            physical_device,
            command_pool,
            graphics_queue,
            swapchain_stuff.swapchain_extent,
            &physical_device_memory_properties,
            vk::SampleCountFlags::TYPE_1,
        );
        let swapchain_framebuffers = VulkanApp28::create_framebuffers(
            &device,
            render_pass,
            &swapchain_imageviews,
            depth_image_view,
            swapchain_stuff.swapchain_extent,
        );
        let (vertices, indices) = share::load_model(&Path::new(MODEL_PATH));
        VulkanApp28::check_mipmap_support(&instance, physical_device, vk::Format::R8G8B8A8_UNORM);
        let (texture_image, texture_image_memory, mip_levels) = VulkanApp28::create_texture_image(
            &device,
            command_pool,
            graphics_queue,
            &physical_device_memory_properties,
            &Path::new(TEXTURE_PATH),
        );
        let texture_image_view =
            share::v1::create_texture_image_view(&device, texture_image, mip_levels);
        let texture_sampler = VulkanApp28::create_texture_sampler(&device, mip_levels);
        let (vertex_buffer, vertex_buffer_memory) = share::v1::create_vertex_buffer(
            &device,
            &physical_device_memory_properties,
            command_pool,
            graphics_queue,
            &vertices,
        );
        let (index_buffer, index_buffer_memory) = share::v1::create_index_buffer(
            &device,
            &physical_device_memory_properties,
            command_pool,
            graphics_queue,
            &indices,
        );
        let (uniform_buffers, uniform_buffers_memory) = share::v1::create_uniform_buffers(
            &device,
            &physical_device_memory_properties,
            swapchain_stuff.swapchain_images.len(),
        );
        let descriptor_pool =
            share::v2::create_descriptor_pool(&device, swapchain_stuff.swapchain_images.len());
        let descriptor_sets = share::v2::create_descriptor_sets(
            &device,
            descriptor_pool,
            ubo_layout,
            &uniform_buffers,
            texture_image_view,
            texture_sampler,
            swapchain_stuff.swapchain_images.len(),
        );
        let command_buffers = VulkanApp28::create_command_buffers(
            &device,
            command_pool,
            graphics_pipeline,
            &swapchain_framebuffers,
            render_pass,
            swapchain_stuff.swapchain_extent,
            vertex_buffer,
            index_buffer,
            pipeline_layout,
            &descriptor_sets,
            indices.len() as u32,
        );
        let sync_ojbects = share::v1::create_sync_objects(&device, MAX_FRAMES_IN_FLIGHT);

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp28 {
            // winit stuff
            window,

            // vulkan stuff
            _entry: entry,
            instance,
            surface: surface_stuff.surface,
            surface_loader: surface_stuff.surface_loader,
            debug_utils_loader,
            debug_merssager,

            physical_device,
            memory_properties: physical_device_memory_properties,
            device,

            queue_family,
            graphics_queue,
            present_queue,

            swapchain_loader: swapchain_stuff.swapchain_loader,
            swapchain: swapchain_stuff.swapchain,
            swapchain_format: swapchain_stuff.swapchain_format,
            swapchain_images: swapchain_stuff.swapchain_images,
            swapchain_extent: swapchain_stuff.swapchain_extent,
            swapchain_imageviews,
            swapchain_framebuffers,

            pipeline_layout,
            ubo_layout,
            render_pass,
            graphics_pipeline,

            depth_image,
            depth_image_view,
            depth_image_memory,

            _mip_levels: mip_levels,
            texture_image,
            texture_image_view,
            texture_sampler,
            texture_image_memory,

            _vertices: vertices,
            indices,

            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,

            uniform_transform: UniformBufferObject {
                model: Matrix4::from_angle_z(Deg(90.0)),
                view: Matrix4::look_at(
                    Point3::new(2.0, 2.0, 2.0),
                    Point3::new(0.0, 0.0, 0.0),
                    Vector3::new(0.0, 0.0, 1.0),
                ),
                proj: {
                    let mut proj = cgmath::perspective(
                        Deg(45.0),
                        swapchain_stuff.swapchain_extent.width as f32
                            / swapchain_stuff.swapchain_extent.height as f32,
                        0.1,
                        10.0,
                    );
                    proj[1][1] = proj[1][1] * -1.0;
                    proj
                },
            },
            uniform_buffers,
            uniform_buffers_memory,

            descriptor_pool,
            descriptor_sets,

            command_pool,
            command_buffers,

            image_available_semaphores: sync_ojbects.image_available_semaphores,
            render_finished_semaphores: sync_ojbects.render_finished_semaphores,
            in_flight_fences: sync_ojbects.inflight_fences,
            current_frame: 0,

            is_framebuffer_resized: false,
        }
    }

    fn create_texture_image(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        image_path: &Path,
    ) -> (vk::Image, vk::DeviceMemory, u32) {
        let mut image_object = image::open(image_path).unwrap(); // this function is slow in debug mode.
        image_object = image_object.flipv();
        let (image_width, image_height) = (image_object.width(), image_object.height());
        let image_data = match &image_object {
            image::DynamicImage::ImageBgr8(_)
            | image::DynamicImage::ImageLuma8(_)
            | image::DynamicImage::ImageRgb8(_) => image_object.to_rgba().into_raw(),
            image::DynamicImage::ImageBgra8(_)
            | image::DynamicImage::ImageLumaA8(_)
            | image::DynamicImage::ImageRgba8(_) => image_object.raw_pixels(),
        };
        let image_size =
            (::std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;
        let mip_levels = ((::std::cmp::max(image_width, image_height) as f32)
            .log2()
            .floor() as u32)
            + 1;

        if image_size <= 0 {
            panic!("Failed to load texture image!")
        }

        let (staging_buffer, staging_buffer_memory) = share::create_buffer(
            device,
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device_memory_properties,
        );

        unsafe {
            let data_ptr = device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    image_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut u8;

            data_ptr.copy_from_nonoverlapping(image_data.as_ptr(), image_data.len());

            device.unmap_memory(staging_buffer_memory);
        }

        let (texture_image, texture_image_memory) = share::v1::create_image(
            device,
            image_width,
            image_height,
            mip_levels,
            vk::SampleCountFlags::TYPE_1,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            device_memory_properties,
        );

        share::v1::transition_image_layout(
            device,
            command_pool,
            submit_queue,
            texture_image,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            mip_levels,
        );

        share::copy_buffer_to_image(
            device,
            command_pool,
            submit_queue,
            staging_buffer,
            texture_image,
            image_width,
            image_height,
        );

        VulkanApp28::generate_mipmaps(
            device,
            command_pool,
            submit_queue,
            texture_image,
            image_width,
            image_height,
            mip_levels,
        );

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        (texture_image, texture_image_memory, mip_levels)
    }

    fn generate_mipmaps(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        image: vk::Image,
        tex_width: u32,
        tex_height: u32,
        mip_levels: u32,
    ) {
        let command_buffer = share::begin_single_time_command(device, command_pool);

        let mut image_barrier = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::empty(),
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::UNDEFINED,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        };

        let mut mip_width = tex_width as i32;
        let mut mip_height = tex_height as i32;

        for i in 1..mip_levels {
            image_barrier.subresource_range.base_mip_level = i - 1;
            image_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            image_barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            image_barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[image_barrier.clone()],
                );
            }

            let blits = [vk::ImageBlit {
                src_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: i - 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                src_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: mip_width,
                        y: mip_height,
                        z: 1,
                    },
                ],
                dst_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: i,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                dst_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: max(mip_width / 2, 1),
                        y: max(mip_height / 2, 1),
                        z: 1,
                    },
                ],
            }];

            unsafe {
                device.cmd_blit_image(
                    command_buffer,
                    image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &blits,
                    vk::Filter::LINEAR,
                );
            }

            image_barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            image_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
            image_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[image_barrier.clone()],
                );
            }

            mip_width = max(mip_width / 2, 1);
            mip_height = max(mip_height / 2, 1);
        }

        image_barrier.subresource_range.base_mip_level = mip_levels - 1;
        image_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        image_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        image_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier.clone()],
            );
        }

        share::end_single_time_command(device, command_pool, submit_queue, command_buffer);
    }

    fn check_mipmap_support(
        instance: &ash::Instance,
        physcial_device: vk::PhysicalDevice,
        image_format: vk::Format,
    ) {
        let format_properties = unsafe {
            instance.get_physical_device_format_properties(physcial_device, image_format)
        };

        let is_sample_image_filter_linear_support = format_properties
            .optimal_tiling_features
            .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR);

        if is_sample_image_filter_linear_support == false {
            panic!("Texture Image format does not support linear blitting!")
        }
    }

    fn create_texture_sampler(device: &ash::Device, mip_levels: u32) -> vk::Sampler {
        let sampler_create_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SamplerCreateFlags::empty(),
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: 16.0,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            min_lod: 0.0,
            max_lod: mip_levels as f32,
            mip_lod_bias: 0.0,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
        };

        unsafe {
            device
                .create_sampler(&sampler_create_info, None)
                .expect("Failed to create Sampler!")
        }
    }
}

// Fix content -------------------------------------------------------------------------------
impl VulkanApp28 {
    fn create_render_pass(
        instance: &ash::Instance,
        device: &ash::Device,
        physcial_device: vk::PhysicalDevice,
        surface_format: vk::Format,
    ) -> vk::RenderPass {
        let color_attachment = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: surface_format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        };

        let depth_attachment = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: share::find_depth_format(instance, physcial_device),
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let subpasses = [vk::SubpassDescription {
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            p_depth_stencil_attachment: &depth_attachment_ref,
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            p_resolve_attachments: ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
        }];

        let render_pass_attachments = [color_attachment, depth_attachment];

        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
        }];

        let renderpass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            flags: vk::RenderPassCreateFlags::empty(),
            p_next: ptr::null(),
            attachment_count: render_pass_attachments.len() as u32,
            p_attachments: render_pass_attachments.as_ptr(),
            subpass_count: subpasses.len() as u32,
            p_subpasses: subpasses.as_ptr(),
            dependency_count: subpass_dependencies.len() as u32,
            p_dependencies: subpass_dependencies.as_ptr(),
        };

        unsafe {
            device
                .create_render_pass(&renderpass_create_info, None)
                .expect("Failed to create render pass!")
        }
    }

    fn create_framebuffers(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        image_views: &Vec<vk::ImageView>,
        depth_image_view: vk::ImageView,
        swapchain_extent: vk::Extent2D,
    ) -> Vec<vk::Framebuffer> {
        let mut framebuffers = vec![];

        for &image_view in image_views.iter() {
            let attachments = [image_view, depth_image_view];

            let framebuffer_create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::FramebufferCreateFlags::empty(),
                render_pass,
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: swapchain_extent.width,
                height: swapchain_extent.height,
                layers: 1,
            };

            let framebuffer = unsafe {
                device
                    .create_framebuffer(&framebuffer_create_info, None)
                    .expect("Failed to create Framebuffer!")
            };

            framebuffers.push(framebuffer);
        }

        framebuffers
    }

    fn create_graphics_pipeline(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
        ubo_set_layout: vk::DescriptorSetLayout,
    ) -> (vk::Pipeline, vk::PipelineLayout) {
        let vert_shader_module = share::create_shader_module(
            device,
            include_bytes!("../../shaders/spv/26-shader-depth.vert.spv").to_vec(),
        );
        let frag_shader_module = share::create_shader_module(
            device,
            include_bytes!("../../shaders/spv/26-shader-depth.frag.spv").to_vec(),
        );

        let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                // Vertex Shader
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                module: vert_shader_module,
                p_name: main_function_name.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: vk::ShaderStageFlags::VERTEX,
            },
            vk::PipelineShaderStageCreateInfo {
                // Fragment Shader
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                module: frag_shader_module,
                p_name: main_function_name.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
        ];

        let binding_description = VertexV3::get_binding_descriptions();
        let attribute_description = VertexV3::get_attribute_descriptions();

        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_attribute_description_count: attribute_description.len() as u32,
            p_vertex_attribute_descriptions: attribute_description.as_ptr(),
            vertex_binding_description_count: binding_description.len() as u32,
            p_vertex_binding_descriptions: binding_description.as_ptr(),
        };
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
            p_next: ptr::null(),
            primitive_restart_enable: vk::FALSE,
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        };

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain_extent.width as f32,
            height: swapchain_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        }];

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineViewportStateCreateFlags::empty(),
            scissor_count: scissors.len() as u32,
            p_scissors: scissors.as_ptr(),
            viewport_count: viewports.len() as u32,
            p_viewports: viewports.as_ptr(),
        };

        let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: vk::FALSE,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            rasterizer_discard_enable: vk::FALSE,
            depth_bias_clamp: 0.0,
            depth_bias_constant_factor: 0.0,
            depth_bias_enable: vk::FALSE,
            depth_bias_slope_factor: 0.0,
        };

        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            flags: vk::PipelineMultisampleStateCreateFlags::empty(),
            p_next: ptr::null(),
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            sample_shading_enable: vk::FALSE,
            min_sample_shading: 0.0,
            p_sample_mask: ptr::null(),
            alpha_to_one_enable: vk::FALSE,
            alpha_to_coverage_enable: vk::FALSE,
        };

        let stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        };

        let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,
            depth_compare_op: vk::CompareOp::LESS,
            depth_bounds_test_enable: vk::FALSE,
            stencil_test_enable: vk::FALSE,
            front: stencil_state,
            back: stencil_state,
            max_depth_bounds: 1.0,
            min_depth_bounds: 0.0,
        };

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::FALSE,
            color_write_mask: vk::ColorComponentFlags::all(),
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        }];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: vk::FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: color_blend_attachment_states.len() as u32,
            p_attachments: color_blend_attachment_states.as_ptr(),
            blend_constants: [0.0, 0.0, 0.0, 0.0],
        };

        let set_layouts = [ubo_set_layout];

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: set_layouts.len() as u32,
            p_set_layouts: set_layouts.as_ptr(),
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
        };

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Failed to create pipeline layout!")
        };

        let graphic_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
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
            render_pass,
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
        }];

        let graphics_pipelines = unsafe {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &graphic_pipeline_create_infos,
                    None,
                )
                .expect("Failed to create Graphics Pipeline!.")
        };

        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        }

        (graphics_pipelines[0], pipeline_layout)
    }

    fn create_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        graphics_pipeline: vk::Pipeline,
        framebuffers: &Vec<vk::Framebuffer>,
        render_pass: vk::RenderPass,
        surface_extent: vk::Extent2D,
        vertex_buffer: vk::Buffer,
        index_buffer: vk::Buffer,
        pipeline_layout: vk::PipelineLayout,
        descriptor_sets: &Vec<vk::DescriptorSet>,
        index_count: u32,
    ) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: framebuffers.len() as u32,
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        };

        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
                p_next: ptr::null(),
                p_inheritance_info: ptr::null(),
                flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
            };

            unsafe {
                device
                    .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                    .expect("Failed to begin recording Command Buffer at beginning!");
            }

            let clear_values = [
                vk::ClearValue {
                    // clear value for color buffer
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    },
                },
                vk::ClearValue {
                    // clear value for depth buffer
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ];

            let render_pass_begin_info = vk::RenderPassBeginInfo {
                s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                p_next: ptr::null(),
                render_pass,
                framebuffer: framebuffers[i],
                render_area: vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: surface_extent,
                },
                clear_value_count: clear_values.len() as u32,
                p_clear_values: clear_values.as_ptr(),
            };

            unsafe {
                device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );
                device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    graphics_pipeline,
                );

                let vertex_buffers = [vertex_buffer];
                let offsets = [0_u64];
                let descriptor_sets_to_bind = [descriptor_sets[i]];

                device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
                device.cmd_bind_index_buffer(
                    command_buffer,
                    index_buffer,
                    0,
                    vk::IndexType::UINT32,
                );
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout,
                    0,
                    &descriptor_sets_to_bind,
                    &[],
                );

                device.cmd_draw_indexed(command_buffer, index_count, 1, 0, 0, 0);

                device.cmd_end_render_pass(command_buffer);

                device
                    .end_command_buffer(command_buffer)
                    .expect("Failed to record Command Buffer at Ending!");
            }
        }

        command_buffers
    }

    fn update_uniform_buffer(&mut self, current_image: usize, delta_time: f32) {
        self.uniform_transform.model =
            Matrix4::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), Deg(90.0) * delta_time)
                * self.uniform_transform.model;

        let ubos = [self.uniform_transform.clone()];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            let data_ptr =
                self.device
                    .map_memory(
                        self.uniform_buffers_memory[current_image],
                        0,
                        buffer_size,
                        vk::MemoryMapFlags::empty(),
                    )
                    .expect("Failed to Map Memory") as *mut UniformBufferObject;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            self.device
                .unmap_memory(self.uniform_buffers_memory[current_image]);
        }
    }
}

impl Drop for VulkanApp28 {
    fn drop(&mut self) {
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.device
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.in_flight_fences[i], None);
            }

            self.cleanup_swapchain();

            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);

            for i in 0..self.uniform_buffers.len() {
                self.device.destroy_buffer(self.uniform_buffers[i], None);
                self.device
                    .free_memory(self.uniform_buffers_memory[i], None);
            }

            self.device.destroy_buffer(self.index_buffer, None);
            self.device.free_memory(self.index_buffer_memory, None);

            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);

            self.device.destroy_sampler(self.texture_sampler, None);
            self.device
                .destroy_image_view(self.texture_image_view, None);

            self.device.destroy_image(self.texture_image, None);
            self.device.free_memory(self.texture_image_memory, None);

            self.device
                .destroy_descriptor_set_layout(self.ubo_layout, None);

            self.device.destroy_command_pool(self.command_pool, None);

            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);

            if VALIDATION.is_enable {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_merssager, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

impl VulkanApp for VulkanApp28 {
    fn draw_frame(&mut self, delta_time: f32) {
        let wait_fences = [self.in_flight_fences[self.current_frame]];

        unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
        }

        let (image_index, _is_sub_optimal) = unsafe {
            let result = self.swapchain_loader.acquire_next_image(
                self.swapchain,
                std::u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            );
            match result {
                Ok(image_index) => image_index,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.recreate_swapchain();
                        return;
                    }
                    _ => panic!("Failed to acquire Swap Chain Image!"),
                },
            }
        };

        self.update_uniform_buffer(image_index as usize, delta_time);

        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffers[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];

        unsafe {
            self.device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            self.device
                .queue_submit(
                    self.graphics_queue,
                    &submit_infos,
                    self.in_flight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [self.swapchain];

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
        };

        let result = unsafe {
            self.swapchain_loader
                .queue_present(self.present_queue, &present_info)
        };

        let is_resized = match result {
            Ok(_) => self.is_framebuffer_resized,
            Err(vk_result) => match vk_result {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                _ => panic!("Failed to execute queue present."),
            },
        };
        if is_resized {
            self.is_framebuffer_resized = false;
            self.recreate_swapchain();
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn recreate_swapchain(&mut self) {
        // parameters -------------
        let surface_suff = SurfaceStuff {
            surface_loader: self.surface_loader.clone(),
            surface: self.surface,
            screen_width: WINDOW_WIDTH,
            screen_height: WINDOW_HEIGHT,
        };
        // ------------------------

        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait device idle!")
        };
        self.cleanup_swapchain();

        let swapchain_stuff = share::create_swapchain(
            &self.instance,
            &self.device,
            self.physical_device,
            &self.window,
            &surface_suff,
            &self.queue_family,
        );
        self.swapchain_loader = swapchain_stuff.swapchain_loader;
        self.swapchain = swapchain_stuff.swapchain;
        self.swapchain_images = swapchain_stuff.swapchain_images;
        self.swapchain_format = swapchain_stuff.swapchain_format;
        self.swapchain_extent = swapchain_stuff.swapchain_extent;

        self.swapchain_imageviews = share::v1::create_image_views(
            &self.device,
            self.swapchain_format,
            &self.swapchain_images,
        );
        self.render_pass = VulkanApp28::create_render_pass(
            &self.instance,
            &self.device,
            self.physical_device,
            self.swapchain_format,
        );
        let (graphics_pipeline, pipeline_layout) = VulkanApp28::create_graphics_pipeline(
            &self.device,
            self.render_pass,
            swapchain_stuff.swapchain_extent,
            self.ubo_layout,
        );
        self.graphics_pipeline = graphics_pipeline;
        self.pipeline_layout = pipeline_layout;

        let depth_resources = share::v1::create_depth_resources(
            &self.instance,
            &self.device,
            self.physical_device,
            self.command_pool,
            self.graphics_queue,
            self.swapchain_extent,
            &self.memory_properties,
            vk::SampleCountFlags::TYPE_1,
        );
        self.depth_image = depth_resources.0;
        self.depth_image_view = depth_resources.1;
        self.depth_image_memory = depth_resources.2;

        self.swapchain_framebuffers = VulkanApp28::create_framebuffers(
            &self.device,
            self.render_pass,
            &self.swapchain_imageviews,
            self.depth_image_view,
            self.swapchain_extent,
        );
        self.command_buffers = VulkanApp28::create_command_buffers(
            &self.device,
            self.command_pool,
            self.graphics_pipeline,
            &self.swapchain_framebuffers,
            self.render_pass,
            self.swapchain_extent,
            self.vertex_buffer,
            self.index_buffer,
            self.pipeline_layout,
            &self.descriptor_sets,
            self.indices.len() as u32,
        );
    }

    fn cleanup_swapchain(&self) {
        unsafe {
            self.device.destroy_image_view(self.depth_image_view, None);
            self.device.destroy_image(self.depth_image, None);
            self.device.free_memory(self.depth_image_memory, None);

            self.device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            for &framebuffer in self.swapchain_framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            for &image_view in self.swapchain_imageviews.iter() {
                self.device.destroy_image_view(image_view, None);
            }
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }

    fn wait_device_idle(&self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait device idle!")
        };
    }

    fn resize_framebuffer(&mut self) {
        self.is_framebuffer_resized = true;
    }

    fn window_ref(&self) -> &winit::window::Window {
        &self.window
    }
}

fn main() {
    let program_proc = ProgramProc::new();
    let vulkan_app = VulkanApp28::new(&program_proc.event_loop);

    program_proc.main_loop(vulkan_app);
}
// -------------------------------------------------------------------------------------------
