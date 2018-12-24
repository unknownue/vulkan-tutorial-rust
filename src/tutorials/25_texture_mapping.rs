
use vulkan_tutorial_rust::{
    utility, // the mod define some fixed functions that have been learned before.
    utility::share,
    utility::debug::*,
    utility::structures::*,
    utility::constants::*,
    utility::window::{ VulkanApp, ProgramProc },
};

use ash::vk;
use ash::version::InstanceV1_0;
use ash::version::DeviceV1_0;
use cgmath::{ Matrix4, Deg, Point3, Vector3 };
use memoffset::offset_of;

use std::path::Path;
use std::ptr;
use std::ffi::CString;

// Constants
const WINDOW_TITLE: &'static str = "25.Texture Mapping";
const TEXTURE_PATH: &'static str = "assets/texture.jpg";

#[derive(Debug, Clone, Copy)]
pub struct VertexV2 {
    pub pos:       [f32; 2],
    pub color:     [f32; 4],
    pub tex_coord: [f32; 2],
}
impl VertexV2 {

    pub fn get_binding_description() -> [vk::VertexInputBindingDescription; 1] {
        [
            vk::VertexInputBindingDescription {
                binding   : 0,
                stride    : std::mem::size_of::<VertexV2>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            },
        ]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            vk::VertexInputAttributeDescription {
                binding  : 0,
                location : 0,
                format   : vk::Format::R32G32_SFLOAT,
                offset   : offset_of!(VertexV2, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding  : 0,
                location : 1,
                format   : vk::Format::R32G32B32A32_SFLOAT,
                offset   : offset_of!(VertexV2, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding  : 0,
                location : 2,
                format   : vk::Format::R32G32_SFLOAT,
                offset   : offset_of!(VertexV2, tex_coord) as u32,
            },
        ]
    }
}
pub const RECT_TEX_COORD_VERTICES_DATA: [VertexV2; 4] = [
    VertexV2 { pos: [-0.75, -0.75], color: [1.0, 0.0, 0.0, 1.0], tex_coord: [1.0, 0.0] },
    VertexV2 { pos: [ 0.75, -0.75], color: [0.0, 1.0, 0.0, 1.0], tex_coord: [0.0, 0.0] },
    VertexV2 { pos: [ 0.75,  0.75], color: [0.0, 0.0, 1.0, 1.0], tex_coord: [0.0, 1.0] },
    VertexV2 { pos: [-0.75,  0.75], color: [1.0, 1.0, 1.0, 1.0], tex_coord: [1.0, 1.0] },
];

struct VulkanApp25 {

    window                     : winit::Window,

    // vulkan stuff
    _entry                     : ash::Entry,
    instance                   : ash::Instance,
    surface_loader             : ash::extensions::Surface,
    surface                    : vk::SurfaceKHR,
    debug_report_loader        : ash::extensions::DebugReport,
    debug_callback             : vk::DebugReportCallbackEXT,

    physical_device            : vk::PhysicalDevice,
    device                     : ash::Device,

    queue_family               : QueueFamilyIndices,
    graphics_queue             : vk::Queue,
    present_queue              : vk::Queue,

    swapchain_loader           : ash::extensions::Swapchain,
    swapchain                  : vk::SwapchainKHR,
    swapchain_images           : Vec<vk::Image>,
    swapchain_format           : vk::Format,
    swapchain_extent           : vk::Extent2D,
    swapchain_imageviews       : Vec<vk::ImageView>,
    swapchain_framebuffers     : Vec<vk::Framebuffer>,

    render_pass                : vk::RenderPass,
    ubo_layout                 : vk::DescriptorSetLayout,
    pipeline_layout            : vk::PipelineLayout,
    graphics_pipeline          : vk::Pipeline,

    texture_image              : vk::Image,
    texture_image_view         : vk::ImageView,
    texture_sampler            : vk::Sampler,
    texture_image_memory       : vk::DeviceMemory,

    vertex_buffer              : vk::Buffer,
    vertex_buffer_memory       : vk::DeviceMemory,
    index_buffer               : vk::Buffer,
    index_buffer_memory        : vk::DeviceMemory,

    uniform_transform          : UniformBufferObject,
    uniform_buffers            : Vec<vk::Buffer>,
    uniform_buffers_memory     : Vec<vk::DeviceMemory>,

    descriptor_pool            : vk::DescriptorPool,
    descriptor_sets            : Vec<vk::DescriptorSet>,

    command_pool               : vk::CommandPool,
    command_buffers            : Vec<vk::CommandBuffer>,

    image_available_semaphores : Vec<vk::Semaphore>,
    render_finished_semaphores : Vec<vk::Semaphore>,
    in_flight_fences           : Vec<vk::Fence>,
    current_frame              : usize,

    is_framebuffer_resized     : bool,
}

impl VulkanApp25 {

    pub fn new(event_loop: &winit::EventsLoop) -> VulkanApp25 {

        let window = utility::window::init_window(&event_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        // init vulkan stuff
        let entry = ash::Entry::new().unwrap();
        let instance = share::create_instance(&entry, WINDOW_TITLE, VALIDATION.is_enable, &VALIDATION.required_validation_layers.to_vec());
        let surface_stuff = share::create_surface(&entry, &instance, &window, WINDOW_WIDTH, WINDOW_HEIGHT);
        let (debug_report_loader, debug_callback) = setup_debug_callback(VALIDATION.is_enable, &entry, &instance);
        let physical_device = share::pick_physical_device(&instance, &surface_stuff, &DEVICE_EXTENSIONS);
        let physical_device_memory_properties = unsafe {
            instance.get_physical_device_memory_properties(physical_device)
        };
        let (device, queue_family) = share::create_logical_device(&instance, physical_device, &VALIDATION, &DEVICE_EXTENSIONS, &surface_stuff);
        let graphics_queue = unsafe { device.get_device_queue(queue_family.graphics_family as u32, 0) };
        let present_queue  = unsafe { device.get_device_queue(queue_family.present_family as u32, 0) };
        let swapchain_stuff = share::create_swapchain(&instance, &device, physical_device, &window, &surface_stuff, &queue_family);
        let swapchain_imageviews = share::v1::create_image_views(&device, swapchain_stuff.swapchain_format, &swapchain_stuff.swapchain_images);
        let render_pass = share::v1::create_render_pass(&device, swapchain_stuff.swapchain_format);
        let ubo_layout = VulkanApp25::create_descriptor_set_layout(&device);
        let (graphics_pipeline, pipeline_layout) = VulkanApp25::create_graphics_pipeline(&device, render_pass, swapchain_stuff.swapchain_extent, ubo_layout);
        let swapchain_framebuffers = share::v1::create_framebuffers(&device, render_pass, &swapchain_imageviews, swapchain_stuff.swapchain_extent);
        let command_pool = share::v1::create_command_pool(&device, &queue_family);
        let (texture_image, texture_image_memory) = share::v1::create_texture_image(&device, command_pool, graphics_queue, &physical_device_memory_properties, &Path::new(TEXTURE_PATH));
        let texture_image_view = share::v1::create_texture_image_view(&device, texture_image, 1);
        let texture_sampler = share::v1::create_texture_sampler(&device);
        let (vertex_buffer, vertex_buffer_memory) = share::v1::create_vertex_buffer(&device, &physical_device_memory_properties, command_pool, graphics_queue, &RECT_TEX_COORD_VERTICES_DATA);
        let (index_buffer, index_buffer_memory) = share::v1::create_index_buffer(&device, &physical_device_memory_properties, command_pool, graphics_queue, &RECT_INDICES_DATA);
        let (uniform_buffers, uniform_buffers_memory) = share::v1::create_uniform_buffers(&device, &physical_device_memory_properties, swapchain_stuff.swapchain_images.len());
        let descriptor_pool = VulkanApp25::create_descriptor_pool(&device, swapchain_stuff.swapchain_images.len());
        let descriptor_sets = VulkanApp25::create_descriptor_sets(&device, descriptor_pool, ubo_layout, &uniform_buffers, texture_image_view, texture_sampler, swapchain_stuff.swapchain_images.len());
        let command_buffers = VulkanApp25::create_command_buffers(&device, command_pool, graphics_pipeline, &swapchain_framebuffers, render_pass, swapchain_stuff.swapchain_extent, vertex_buffer, index_buffer, pipeline_layout, &descriptor_sets);
        let sync_ojbects = share::v1::create_sync_objects(&device, MAX_FRAMES_IN_FLIGHT);

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp25 {
            // winit stuff
            window,

            // vulkan stuff
            _entry: entry,
            instance,
            surface: surface_stuff.surface,
            surface_loader: surface_stuff.surface_loader,
            debug_report_loader,
            debug_callback,

            physical_device,
            device,

            queue_family,
            graphics_queue,
            present_queue,

            swapchain_loader: swapchain_stuff.swapchain_loader,
            swapchain:        swapchain_stuff.swapchain,
            swapchain_format: swapchain_stuff.swapchain_format,
            swapchain_images: swapchain_stuff.swapchain_images,
            swapchain_extent: swapchain_stuff.swapchain_extent,
            swapchain_imageviews,
            swapchain_framebuffers,

            pipeline_layout,
            ubo_layout,
            render_pass,
            graphics_pipeline,

            texture_image,
            texture_image_view,
            texture_sampler,
            texture_image_memory,

            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,

            uniform_transform: UniformBufferObject {
                model: Matrix4::from_angle_z(Deg(90.0)),
                view: Matrix4::look_at(Point3::new(2.0, 2.0, 2.0), Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
                proj: {
                    let mut proj = cgmath::perspective(Deg(45.0), swapchain_stuff.swapchain_extent.width as f32 / swapchain_stuff.swapchain_extent.height as f32, 0.1, 10.0);
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
            in_flight_fences          : sync_ojbects.inflight_fences,
            current_frame: 0,

            is_framebuffer_resized: false,
        }
    }

    fn create_descriptor_pool(device: &ash::Device, swapchain_images_size: usize) -> vk::DescriptorPool {

        let pool_sizes = [
            vk::DescriptorPoolSize { // transform descriptor pool
                ty               : vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count : swapchain_images_size as u32
            },
            vk::DescriptorPoolSize { // sampler descriptor pool
                ty               : vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count : swapchain_images_size as u32,
            }
        ];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            s_type          : vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next          : ptr::null(),
            flags           : vk::DescriptorPoolCreateFlags::empty(),
            max_sets        : swapchain_images_size as u32,
            pool_size_count : pool_sizes.len() as u32,
            p_pool_sizes    : pool_sizes.as_ptr(),
        };

        unsafe {
            device.create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Failed to create Descriptor Pool!")
        }
    }

    fn create_descriptor_sets(device: &ash::Device, descriptor_pool: vk::DescriptorPool, descriptor_set_layout: vk::DescriptorSetLayout, uniforms_buffers: &Vec<vk::Buffer>, texture_image_view: vk::ImageView, texture_sampler: vk::Sampler, swapchain_images_size: usize) -> Vec<vk::DescriptorSet> {

        let mut layouts: Vec<vk::DescriptorSetLayout> = vec![];
        for _ in 0..swapchain_images_size {
            layouts.push(descriptor_set_layout);
        }

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            s_type               : vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next               : ptr::null(),
            descriptor_pool,
            descriptor_set_count : swapchain_images_size as u32,
            p_set_layouts        : layouts.as_ptr()
        };

        let descriptor_sets = unsafe {
            device.allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets!")
        };

        for (i, &descritptor_set) in descriptor_sets.iter().enumerate() {
            let descriptor_buffer_infos = [
                vk::DescriptorBufferInfo {
                    buffer : uniforms_buffers[i],
                    offset : 0,
                    range  : ::std::mem::size_of::<UniformBufferObject>() as u64,
                },
            ];

            let descriptor_image_infos = [
                vk::DescriptorImageInfo {
                    sampler      : texture_sampler,
                    image_view   : texture_image_view,
                    image_layout : vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                },
            ];

            let descriptor_write_sets = [
                vk::WriteDescriptorSet { // transform uniform
                    s_type              : vk::StructureType::WRITE_DESCRIPTOR_SET,
                    p_next              : ptr::null(),
                    dst_set             : descritptor_set,
                    dst_binding         : 0,
                    dst_array_element   : 0,
                    descriptor_count    : 1,
                    descriptor_type     : vk::DescriptorType::UNIFORM_BUFFER,
                    p_image_info        : ptr::null(),
                    p_buffer_info       : descriptor_buffer_infos.as_ptr(),
                    p_texel_buffer_view : ptr::null(),
                },
                vk::WriteDescriptorSet { // sampler uniform
                    s_type              : vk::StructureType::WRITE_DESCRIPTOR_SET,
                    p_next              : ptr::null(),
                    dst_set             : descritptor_set,
                    dst_binding         : 1,
                    dst_array_element   : 0,
                    descriptor_count    : 1,
                    descriptor_type     : vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    p_image_info        : descriptor_image_infos.as_ptr(),
                    p_buffer_info       : ptr::null(),
                    p_texel_buffer_view : ptr::null(),
                }
            ];

            unsafe {
                device.update_descriptor_sets(&descriptor_write_sets, &[]);
            }
        }

        descriptor_sets
    }

    fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {

        let ubo_layout_bindings = [
            vk::DescriptorSetLayoutBinding { // transform uniform
                binding              : 0,
                descriptor_type      : vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count     : 1,
                stage_flags          : vk::ShaderStageFlags::VERTEX,
                p_immutable_samplers : ptr::null(),
            },
            vk::DescriptorSetLayoutBinding { // sampler uniform
                binding              : 1,
                descriptor_type      : vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count     : 1,
                stage_flags          : vk::ShaderStageFlags::FRAGMENT,
                p_immutable_samplers : ptr::null(),
            }
        ];

        let ubo_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type        : vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next        : ptr::null(),
            flags         : vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count : ubo_layout_bindings.len() as u32,
            p_bindings    : ubo_layout_bindings.as_ptr(),
        };

        unsafe {
            device.create_descriptor_set_layout(&ubo_layout_create_info, None)
                .expect("Failed to create Descriptor Set Layout!")
        }
    }
}





// Fix content -------------------------------------------------------------------------------
impl VulkanApp25 {

    fn create_command_buffers(device: &ash::Device, command_pool: vk::CommandPool, graphics_pipeline: vk::Pipeline, framebuffers: &Vec<vk::Framebuffer>, render_pass: vk::RenderPass, surface_extent: vk::Extent2D, vertex_buffer: vk::Buffer, index_buffer: vk::Buffer, pipeline_layout: vk::PipelineLayout, descriptor_sets: &Vec<vk::DescriptorSet>) -> Vec<vk::CommandBuffer> {

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type               : vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next               : ptr::null(),
            command_buffer_count : framebuffers.len() as u32,
            command_pool,
            level                : vk::CommandBufferLevel::PRIMARY,
        };

        let command_buffers = unsafe {
            device.allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        };

        for (i, &command_buffer) in command_buffers.iter().enumerate() {

            let command_buffer_begin_info  = vk::CommandBufferBeginInfo {
                s_type             : vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
                p_next             : ptr::null(),
                p_inheritance_info : ptr::null(),
                flags              : vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
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
                s_type            : vk::StructureType::RENDER_PASS_BEGIN_INFO,
                p_next            : ptr::null(),
                render_pass,
                framebuffer       : framebuffers[i],
                render_area: vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: surface_extent,
                },
                clear_value_count : clear_values.len() as u32,
                p_clear_values    : clear_values.as_ptr(),
            };

            unsafe {
                device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
                device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline);

                let vertex_buffers = [
                    vertex_buffer
                ];
                let offsets = [
                    0_u64
                ];
                let descriptor_sets_to_bind = [
                    descriptor_sets[i],
                ];

                device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
                device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
                device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline_layout, 0, &descriptor_sets_to_bind, &[]);

                device.cmd_draw_indexed(command_buffer, RECT_INDICES_DATA.len() as u32, 1, 0, 0, 0);

                device.cmd_end_render_pass(command_buffer);

                device.end_command_buffer(command_buffer)
                    .expect("Failed to record Command Buffer at Ending!");
            }
        }

        command_buffers
    }

    fn update_uniform_buffer(&mut self, current_image: usize, _delta_time: f32) {

        let ubos = [
            self.uniform_transform.clone(),
        ];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            let data_ptr = self.device.map_memory(self.uniform_buffers_memory[current_image], 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory");
            let mut align = ash::util::Align::new(data_ptr, std::mem::align_of::<UniformBufferObject>() as u64, buffer_size);
            align.copy_from_slice(&ubos);
            self.device.unmap_memory(self.uniform_buffers_memory[current_image]);
        }
    }

    fn create_graphics_pipeline(device: &ash::Device, render_pass: vk::RenderPass, swapchain_extent: vk::Extent2D, ubo_set_layout: vk::DescriptorSetLayout) -> (vk::Pipeline, vk::PipelineLayout) {

        let vert_shader_code = utility::tools::read_shader_code(Path::new("shaders/spv/25-shader-textures.vert.spv"));
        let frag_shader_code = utility::tools::read_shader_code(Path::new("shaders/spv/25-shader-textures.frag.spv"));

        let vert_shader_module = share::create_shader_module(device, vert_shader_code);
        let frag_shader_module = share::create_shader_module(device, frag_shader_code);

        let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo { // Vertex Shader
                s_type                : vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next                : ptr::null(),
                flags                 : vk::PipelineShaderStageCreateFlags::empty(),
                module                : vert_shader_module,
                p_name                : main_function_name.as_ptr(),
                p_specialization_info : ptr::null(),
                stage                 : vk::ShaderStageFlags::VERTEX,
            },
            vk::PipelineShaderStageCreateInfo { // Fragment Shader
                s_type                : vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next                : ptr::null(),
                flags                 : vk::PipelineShaderStageCreateFlags::empty(),
                module                : frag_shader_module,
                p_name                : main_function_name.as_ptr(),
                p_specialization_info : ptr::null(),
                stage                 : vk::ShaderStageFlags::FRAGMENT,
            },
        ];

        let binding_description   = VertexV2::get_binding_description();
        let attribute_description = VertexV2::get_attribute_descriptions();

        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
            s_type                             : vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next                             : ptr::null(),
            flags                              : vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_attribute_description_count : attribute_description.len() as u32,
            p_vertex_attribute_descriptions    : attribute_description.as_ptr(),
            vertex_binding_description_count   : binding_description.len() as u32,
            p_vertex_binding_descriptions      : binding_description.as_ptr(),
        };
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type                   : vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            flags                    : vk::PipelineInputAssemblyStateCreateFlags::empty(),
            p_next                   : ptr::null(),
            primitive_restart_enable : vk::FALSE,
            topology                 : vk::PrimitiveTopology::TRIANGLE_LIST,
        };

        let viewports = [
            vk::Viewport {
                x         : 0.0,
                y         : 0.0,
                width     : swapchain_extent.width as f32,
                height    : swapchain_extent.height as f32,
                min_depth : 0.0,
                max_depth : 1.0,
            },
        ];

        let scissors = [
            vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain_extent,
            },
        ];

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
            s_type         : vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next         : ptr::null(),
            flags          : vk::PipelineViewportStateCreateFlags::empty(),
            scissor_count  : scissors.len()  as u32,
            p_scissors     : scissors.as_ptr(),
            viewport_count : viewports.len() as u32,
            p_viewports    : viewports.as_ptr(),
        };

        let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo {
            s_type                     : vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next                     : ptr::null(),
            flags                      : vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable         : vk::FALSE,
            cull_mode                  : vk::CullModeFlags::BACK,
            front_face                 : vk::FrontFace::COUNTER_CLOCKWISE,
            line_width                 : 1.0,
            polygon_mode               : vk::PolygonMode::FILL,
            rasterizer_discard_enable  : vk::FALSE,
            depth_bias_clamp           : 0.0,
            depth_bias_constant_factor : 0.0,
            depth_bias_enable          : vk::FALSE,
            depth_bias_slope_factor    : 0.0,
        };

        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
            s_type                   : vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            flags                    : vk::PipelineMultisampleStateCreateFlags::empty(),
            p_next                   : ptr::null(),
            rasterization_samples    : vk::SampleCountFlags::TYPE_1,
            sample_shading_enable    : vk::FALSE,
            min_sample_shading       : 0.0,
            p_sample_mask            : ptr::null(),
            alpha_to_one_enable      : vk::FALSE,
            alpha_to_coverage_enable : vk::FALSE,
        };

        let stencil_state = vk::StencilOpState {
            fail_op       : vk::StencilOp::KEEP,
            pass_op       : vk::StencilOp::KEEP,
            depth_fail_op : vk::StencilOp::KEEP,
            compare_op    : vk::CompareOp::ALWAYS,
            compare_mask  : 0,
            write_mask    : 0,
            reference     : 0,
        };

        let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo {
            s_type                   : vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next                   : ptr::null(),
            flags                    : vk::PipelineDepthStencilStateCreateFlags::empty(),
            depth_test_enable        : vk::FALSE,
            depth_write_enable       : vk::FALSE,
            depth_compare_op         : vk::CompareOp::LESS_OR_EQUAL,
            depth_bounds_test_enable : vk::FALSE,
            stencil_test_enable      : vk::FALSE,
            front                    : stencil_state,
            back                     : stencil_state,
            max_depth_bounds         : 1.0,
            min_depth_bounds         : 0.0,
        };

        let color_blend_attachment_states = [
            vk::PipelineColorBlendAttachmentState {
                blend_enable           : vk::FALSE,
                color_write_mask       : vk::ColorComponentFlags::all(),
                src_color_blend_factor : vk::BlendFactor::ONE,
                dst_color_blend_factor : vk::BlendFactor::ZERO,
                color_blend_op         : vk::BlendOp::ADD,
                src_alpha_blend_factor : vk::BlendFactor::ONE,
                dst_alpha_blend_factor : vk::BlendFactor::ZERO,
                alpha_blend_op         : vk::BlendOp::ADD,
            },
        ];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            s_type           : vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next           : ptr::null(),
            flags            : vk::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable  : vk::FALSE,
            logic_op         : vk::LogicOp::COPY,
            attachment_count : color_blend_attachment_states.len() as u32,
            p_attachments    : color_blend_attachment_states.as_ptr(),
            blend_constants  : [0.0, 0.0, 0.0, 0.0],
        };

        let set_layouts = [
            ubo_set_layout,
        ];

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type                    : vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next                    : ptr::null(),
            flags                     : vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count          : set_layouts.len() as u32,
            p_set_layouts             : set_layouts.as_ptr(),
            push_constant_range_count : 0,
            p_push_constant_ranges    : ptr::null(),
        };

        let pipeline_layout = unsafe {
            device.create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Failed to create pipeline layout!")
        };

        let graphic_pipeline_create_infos = [
            vk::GraphicsPipelineCreateInfo {
                s_type                 : vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
                p_next                 : ptr::null(),
                flags                  : vk::PipelineCreateFlags::empty(),
                stage_count            : shader_stages.len() as u32,
                p_stages               : shader_stages.as_ptr(),
                p_vertex_input_state   : &vertex_input_state_create_info,
                p_input_assembly_state : &vertex_input_assembly_state_info,
                p_tessellation_state   : ptr::null(),
                p_viewport_state       : &viewport_state_create_info,
                p_rasterization_state  : &rasterization_statue_create_info,
                p_multisample_state    : &multisample_state_create_info,
                p_depth_stencil_state  : &depth_state_create_info,
                p_color_blend_state    : &color_blend_state,
                p_dynamic_state        : ptr::null(),
                layout                 : pipeline_layout,
                render_pass,
                subpass                : 0,
                base_pipeline_handle   : vk::Pipeline::null(),
                base_pipeline_index    : -1,
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
}

impl Drop for VulkanApp25 {

    fn drop(&mut self) {

        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device.destroy_semaphore(self.image_available_semaphores[i], None);
                self.device.destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.in_flight_fences[i], None);
            }

            self.cleanup_swapchain();

            self.device.destroy_descriptor_pool(self.descriptor_pool, None);

            for i in 0..self.uniform_buffers.len() {
                self.device.destroy_buffer(self.uniform_buffers[i], None);
                self.device.free_memory(self.uniform_buffers_memory[i], None);
            }

            self.device.destroy_buffer(self.index_buffer, None);
            self.device.free_memory(self.index_buffer_memory, None);

            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);

            self.device.destroy_sampler(self.texture_sampler, None);
            self.device.destroy_image_view(self.texture_image_view, None);

            self.device.destroy_image(self.texture_image, None);
            self.device.free_memory(self.texture_image_memory, None);

            self.device.destroy_descriptor_set_layout(self.ubo_layout, None);

            self.device.destroy_command_pool(self.command_pool, None);

            self.device.destroy_device(None);
            self.surface_loader.destroy_surface_khr(self.surface, None);

            if VALIDATION.is_enable {
                self.debug_report_loader.destroy_debug_report_callback_ext(self.debug_callback, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

impl VulkanApp for VulkanApp25 {

    fn draw_frame(&mut self, delta_time: f32) {

        let wait_fences = [
            self.in_flight_fences[self.current_frame]
        ];

        unsafe {
            self.device.wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
        }

        let (image_index, _is_sub_optimal) = unsafe {
            let result = self.swapchain_loader.acquire_next_image_khr(self.swapchain, std::u64::MAX, self.image_available_semaphores[self.current_frame], vk::Fence::null());
            match result {
                | Ok(image_index) => image_index,
                | Err(vk_result) => match vk_result {
                    | vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.recreate_swapchain();
                        return
                    },
                    | _ => panic!("Failed to acquire Swap Chain Image!")
                }
            }
        };

        self.update_uniform_buffer(image_index as usize, delta_time);

        let wait_semaphores = [
            self.image_available_semaphores[self.current_frame],
        ];
        let wait_stages = [
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        ];
        let signal_semaphores = [
            self.render_finished_semaphores[self.current_frame],
        ];

        let submit_infos = [
            vk::SubmitInfo {
                s_type                 : vk::StructureType::SUBMIT_INFO,
                p_next                 : ptr::null(),
                wait_semaphore_count   : wait_semaphores.len() as u32,
                p_wait_semaphores      : wait_semaphores.as_ptr(),
                p_wait_dst_stage_mask  : wait_stages.as_ptr(),
                command_buffer_count   : 1,
                p_command_buffers      : &self.command_buffers[image_index as usize],
                signal_semaphore_count : signal_semaphores.len() as u32,
                p_signal_semaphores    : signal_semaphores.as_ptr(),
            }
        ];

        unsafe {
            self.device.reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            self.device.queue_submit(self.graphics_queue, &submit_infos, self.in_flight_fences[self.current_frame])
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [
            self.swapchain
        ];

        let present_info = vk::PresentInfoKHR {
            s_type               : vk::StructureType::PRESENT_INFO_KHR,
            p_next               : ptr::null(),
            wait_semaphore_count : 1,
            p_wait_semaphores    : signal_semaphores.as_ptr(),
            swapchain_count      : 1,
            p_swapchains         : swapchains.as_ptr(),
            p_image_indices      : &image_index,
            p_results            : ptr::null_mut(),
        };

        let result = unsafe {
            self.swapchain_loader.queue_present_khr(self.present_queue, &present_info)
        };

        let is_resized = match result {
            Ok(_) => self.is_framebuffer_resized,
            Err(vk_result) => match vk_result {
                | vk::Result::ERROR_OUT_OF_DATE_KHR
                | vk::Result::SUBOPTIMAL_KHR => {
                    true
                }
                | _ => panic!("Failed to execute queue present.")
            }
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
            surface_loader: self.surface_loader.clone(), surface: self.surface,
            screen_width: WINDOW_WIDTH, screen_height: WINDOW_HEIGHT,
        };
        // ------------------------

        unsafe {
            self.device.device_wait_idle()
                .expect("Failed to wait device idle!")
        };
        self.cleanup_swapchain();

        let swapchain_stuff = share::create_swapchain(&self.instance, &self.device, self.physical_device, &self.window, &surface_suff, &self.queue_family);
        self.swapchain_loader = swapchain_stuff.swapchain_loader;
        self.swapchain        = swapchain_stuff.swapchain;
        self.swapchain_images = swapchain_stuff.swapchain_images;
        self.swapchain_format = swapchain_stuff.swapchain_format;
        self.swapchain_extent = swapchain_stuff.swapchain_extent;

        self.swapchain_imageviews = share::v1::create_image_views(&self.device, self.swapchain_format, &self.swapchain_images);
        self.render_pass = share::v1::create_render_pass(&self.device, self.swapchain_format);
        let (graphics_pipeline, pipeline_layout) = VulkanApp25::create_graphics_pipeline(&self.device, self.render_pass, swapchain_stuff.swapchain_extent, self.ubo_layout);
        self.graphics_pipeline = graphics_pipeline;
        self.pipeline_layout = pipeline_layout;

        self.swapchain_framebuffers = share::v1::create_framebuffers(&self.device, self.render_pass, &self.swapchain_imageviews, self.swapchain_extent);
        self.command_buffers = VulkanApp25::create_command_buffers(&self.device, self.command_pool, self.graphics_pipeline, &self.swapchain_framebuffers, self.render_pass, self.swapchain_extent, self.vertex_buffer, self.index_buffer, self.pipeline_layout, &self.descriptor_sets);
    }

    fn cleanup_swapchain(&self) {
        unsafe {
            self.device.free_command_buffers(self.command_pool, &self.command_buffers);
            for &framebuffer in self.swapchain_framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            for &image_view in self.swapchain_imageviews.iter() {
                self.device.destroy_image_view(image_view, None);
            }
            self.swapchain_loader.destroy_swapchain_khr(self.swapchain, None);
        }
    }

    fn wait_device_idle(&self) {
        unsafe {
            self.device.device_wait_idle()
                .expect("Failed to wait device idle!")
        };
    }

    fn resize_framebuffer(&mut self) {
        self.is_framebuffer_resized = true;
    }
}

fn main() {

    let mut program_proc = ProgramProc::new();
    let mut vulkan_app = VulkanApp25::new(&program_proc.events_loop);

    program_proc.main_loop(&mut vulkan_app);
}
// -------------------------------------------------------------------------------------------
