
extern crate vulkan_tutorial_rust;
use vulkan_tutorial_rust::{
    utility, // the mod define some fixed functions that have been learned before.
    utility::debug::*,
    utility::vulkan::*,
    utility::structures::*,
};

extern crate winit;
extern crate ash;
extern crate num;
#[macro_use]
extern crate memoffset;

use winit::{ Event, EventsLoop, WindowEvent, VirtualKeyCode };
use ash::vk;
use ash::version::{ V1_0, InstanceV1_0 };
use ash::version::DeviceV1_0;
use vk::types::uint32_t;

type EntryV1 = ash::Entry<V1_0>;

use std::path::Path;
use std::ptr;
use std::ffi::CString;

// Constants
const WINDOW_TITLE: &'static str = "20.Index Buffer";
const WINDOW_WIDTH:  u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const IS_PAINT_FPS_COUNTER: bool = true;
const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: [
        "VK_LAYER_LUNARG_standard_validation",
    ],
};
const DEVICE_EXTENSIONS: DeviceExtension = DeviceExtension {
    names: [vk::VK_KHR_SWAPCHAIN_EXTENSION_NAME],
};
const MAX_FRAMES_IN_FLIGHT: usize = 2;

#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 4],
}
impl Vertex {

    fn get_binding_description() -> [vk::VertexInputBindingDescription; 1] {
        [
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: std::mem::size_of::<Vertex>() as u32,
                input_rate: vk::VertexInputRate::Vertex,
            },
        ]
    }

    fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription {
                binding:  0,
                location: 0,
                format: vk::Format::R32g32Sfloat,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding:  0,
                location: 1,
                format: vk::Format::R32g32b32a32Sfloat,
                offset: offset_of!(Vertex, color) as u32,
            }
        ]
    }
}

const VERTICES_DATA: [Vertex; 4] = [
    Vertex { pos: [-0.5, -0.5], color: [1.0, 0.0, 0.0, 1.0], },
    Vertex { pos: [ 0.5, -0.5], color: [0.0, 1.0, 0.0, 1.0], },
    Vertex { pos: [ 0.5,  0.5], color: [0.0, 0.0, 1.0, 1.0], },
    Vertex { pos: [-0.5,  0.5], color: [1.0, 1.0, 1.0, 1.0], },
];
const INDICES_DATA: [vk::types::uint32_t; 6] = [
    0, 1, 2, 2, 3, 0
];


struct VulkanApp {

    window: winit::Window,

    // vulkan stuff
    _entry: EntryV1,
    instance: ash::Instance<V1_0>,
    surface_loader: ash::extensions::Surface,
    surface: vk::SurfaceKHR,
    debug_report_loader: ash::extensions::DebugReport,
    debug_callback: vk::DebugReportCallbackEXT,

    physical_device: vk::PhysicalDevice,
    device: ash::Device<V1_0>,

    queue_family: QueueFamilyIndices,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    swapchain_loader: ash::extensions::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_imageviews: Vec<vk::ImageView>,
    swapchain_framebuffers: Vec<vk::Framebuffer>,

    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,

    is_framebuffer_resized: bool,
}

impl VulkanApp {

    pub fn new(event_loop: &winit::EventsLoop) -> VulkanApp {

        let window = utility::window::init_window(&event_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        // init vulkan stuff
        let entry = EntryV1::new().unwrap();
        let instance = create_instance(&entry, WINDOW_TITLE, VALIDATION.is_enable, &VALIDATION.required_validation_layers.to_vec());
        let surface_stuff = create_surface(&entry, &instance, &window, WINDOW_WIDTH, WINDOW_HEIGHT);
        let (debug_report_loader, debug_callback) = setup_debug_callback(VALIDATION.is_enable, &entry, &instance);
        let physical_device = pick_physical_device(&instance, &surface_stuff, &DEVICE_EXTENSIONS);
        let (device, queue_family) = create_logical_device(&instance, &physical_device, &VALIDATION, &DEVICE_EXTENSIONS, &surface_stuff);
        let graphics_queue = unsafe { device.get_device_queue(queue_family.graphics_family as u32, 0) };
        let present_queue  = unsafe { device.get_device_queue(queue_family.present_family as u32, 0) };
        let swapchain_stuff = create_swapchain(&instance, &device, &physical_device, &window, &surface_stuff, &queue_family);
        let swapchain_imageviews = create_image_view(&device, &swapchain_stuff.swapchain_format, &swapchain_stuff.swapchain_images);
        let render_pass = create_render_pass(&device, &swapchain_stuff.swapchain_format);
        let (graphics_pipeline, pipeline_layout) = VulkanApp::create_graphics_pipeline(&device, &render_pass, &swapchain_stuff.swapchain_extent);
        let swapchain_framebuffers = create_framebuffers(&device, &render_pass, &swapchain_imageviews, &swapchain_stuff.swapchain_extent);
        let command_pool = create_command_pool(&device, &queue_family);
        let (vertex_buffer, vertex_buffer_memory) = VulkanApp::create_vertex_buffer(&instance, &physical_device, &device, &command_pool, &graphics_queue);
        let (index_buffer, index_buffer_memory) = VulkanApp::create_index_buffer(&instance, &physical_device, &device, &command_pool, &graphics_queue);
        let command_buffers = VulkanApp::create_command_buffers(&device, &command_pool, &graphics_pipeline, &swapchain_framebuffers, &render_pass, &swapchain_stuff.swapchain_extent, &vertex_buffer, &index_buffer);
        let sync_ojbects = create_sync_objects(&device, MAX_FRAMES_IN_FLIGHT);

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp {
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
            render_pass,
            graphics_pipeline,

            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,

            command_pool,
            command_buffers,

            image_available_semaphores: sync_ojbects.image_available_semaphores,
            render_finished_semaphores: sync_ojbects.render_finished_semaphores,
            in_flight_fences:           sync_ojbects.inflight_fences,
            current_frame: 0,

            is_framebuffer_resized: false,
        }
    }

    fn create_vertex_buffer(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice, device: &ash::Device<V1_0>, command_pool: &vk::CommandPool, submit_queue: &vk::Queue)
                            -> (vk::Buffer, vk::DeviceMemory) {

        let buffer_size = std::mem::size_of_val(&VERTICES_DATA) as vk::DeviceSize;;
        let device_memory_properties = instance.get_physical_device_memory_properties(physical_device.clone());

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            device,
            buffer_size,
            vk::BUFFER_USAGE_TRANSFER_SRC_BIT,
            vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT,
            &device_memory_properties,
        );

        unsafe {
            let data_ptr = device.map_memory(staging_buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory");
            let mut vert_align = ash::util::Align::new(data_ptr, std::mem::align_of::<Vertex>() as u64, buffer_size);
            vert_align.copy_from_slice(&VERTICES_DATA);
            device.unmap_memory(staging_buffer_memory);
        }

        let (vertex_buffer, vertex_buffer_memory) = create_buffer(
            device,
            buffer_size,
            vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_VERTEX_BUFFER_BIT,
            vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            &device_memory_properties,
        );

        copy_buffer(device, submit_queue.clone(), command_pool.clone(), &staging_buffer, &vertex_buffer, buffer_size);

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        (vertex_buffer, vertex_buffer_memory)
    }

    fn create_index_buffer(instance: &ash::Instance<V1_0>, physical_device: &vk::PhysicalDevice, device: &ash::Device<V1_0>, command_pool: &vk::CommandPool, submit_queue: &vk::Queue)
        -> (vk::Buffer, vk::DeviceMemory) {

        let buffer_size = std::mem::size_of_val(&INDICES_DATA) as vk::DeviceSize;;
        let device_memory_properties = instance.get_physical_device_memory_properties(physical_device.clone());

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            device,
            buffer_size,
            vk::BUFFER_USAGE_TRANSFER_SRC_BIT,
            vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT,
            &device_memory_properties,
        );

        unsafe {
            let data_ptr = device.map_memory(staging_buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory");
            let mut vert_align = ash::util::Align::new(data_ptr, std::mem::align_of::<uint32_t>() as u64, buffer_size);
            vert_align.copy_from_slice(&INDICES_DATA);
            device.unmap_memory(staging_buffer_memory);
        }

        let (index_buffer, index_buffer_memory) = create_buffer(
            device,
            buffer_size,
            vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_INDEX_BUFFER_BIT,
            vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            &device_memory_properties,
        );

        copy_buffer(device, submit_queue.clone(), command_pool.clone(), &staging_buffer, &index_buffer, buffer_size);

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        (index_buffer, index_buffer_memory)
    }

    fn create_command_buffers(device: &ash::Device<V1_0>, command_pool: &vk::CommandPool, graphics_pipeline: &vk::Pipeline, framebuffers: &Vec<vk::Framebuffer>, render_pass: &vk::RenderPass, surface_extent: &vk::Extent2D, vertex_buffer: &vk::Buffer, index_buffer: &vk::Buffer) -> Vec<vk::CommandBuffer> {

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

                let vertex_buffers = [
                    vertex_buffer.clone()
                ];
                let offsets = [
                    0_u64
                ];

                device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
                device.cmd_bind_index_buffer(command_buffer, index_buffer.clone(), 0, vk::IndexType::Uint32);

                device.cmd_draw_indexed(command_buffer, INDICES_DATA.len() as u32, 1, 0, 0, 0);

                device.cmd_end_render_pass(command_buffer);

                device.end_command_buffer(command_buffer)
                    .expect("Failed to record Command Buffer at Ending!");
            }
        }

        command_buffers
    }
}




// Fix content -------------------------------------------------------------------------------
impl VulkanApp {

    fn create_graphics_pipeline(device: &ash::Device<V1_0>, render_pass: &vk::RenderPass, swapchain_extent: &vk::Extent2D) -> (vk::Pipeline, vk::PipelineLayout) {

        let vert_shader_code = utility::tools::read_shader_code(Path::new("shaders/spv/17-shader-vertexbuffer.vert.spv"));
        let frag_shader_code = utility::tools::read_shader_code(Path::new("shaders/spv/17-shader-vertexbuffer.frag.spv"));

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

        let binding_description = Vertex::get_binding_description();
        let attribute_description = Vertex::get_attribute_descriptions();

        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PipelineVertexInputStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            vertex_attribute_description_count: attribute_description.len() as u32,
            p_vertex_attribute_descriptions: attribute_description.as_ptr(),
            vertex_binding_description_count: binding_description.len() as u32,
            p_vertex_binding_descriptions: binding_description.as_ptr(),
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

    fn draw_frame(&mut self) {

        let wait_fences = [
            self.in_flight_fences[self.current_frame]
        ];

        unsafe {
            self.device.wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
        }

        let image_index = unsafe {
            let result = self.swapchain_loader.acquire_next_image_khr(self.swapchain, std::u64::MAX, self.image_available_semaphores[self.current_frame], vk::Fence::null());
            match result {
                | Ok(image_index) => image_index,
                | Err(vk_result) => match vk_result {
                    | vk::types::Result::ErrorOutOfDateKhr => {
                        self.recreate_swapchain();
                        return
                    },
                    | _ => panic!("Failed to acquire Swap Chain Image!")
                }
            }
        };

        let wait_semaphores = [
            self.image_available_semaphores[self.current_frame],
        ];
        let wait_stages = [
            vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
        ];
        let signal_semaphores = [
            self.render_finished_semaphores[self.current_frame],
        ];

        let submit_infos = [
            vk::SubmitInfo {
                s_type: vk::StructureType::SubmitInfo,
                p_next: ptr::null(),
                wait_semaphore_count: wait_semaphores.len() as u32,
                p_wait_semaphores: wait_semaphores.as_ptr(),
                p_wait_dst_stage_mask: wait_stages.as_ptr(),
                command_buffer_count: 1,
                p_command_buffers: &self.command_buffers[image_index as usize],
                signal_semaphore_count: signal_semaphores.len() as u32,
                p_signal_semaphores: signal_semaphores.as_ptr(),
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
            s_type: vk::StructureType::PresentInfoKhr,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
        };

        let result = unsafe {
            self.swapchain_loader.queue_present_khr(self.present_queue, &present_info)
        };

        let is_resized = match result {
            Ok(_) => self.is_framebuffer_resized,
            Err(vk_result) => match vk_result {
                | vk::Result::ErrorOutOfDateKhr
                | vk::Result::SuboptimalKhr => {
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

        self.device.device_wait_idle()
            .expect("Failed to wait device idle!");
        self.cleanup_swapchain();

        let swapchain_stuff = create_swapchain(&self.instance, &self.device, &self.physical_device, &self.window, &surface_suff, &self.queue_family);
        self.swapchain_loader = swapchain_stuff.swapchain_loader;
        self.swapchain        = swapchain_stuff.swapchain;
        self.swapchain_images = swapchain_stuff.swapchain_images;
        self.swapchain_format = swapchain_stuff.swapchain_format;
        self.swapchain_extent = swapchain_stuff.swapchain_extent;

        self.swapchain_imageviews = create_image_view(&self.device, &self.swapchain_format, &self.swapchain_images);
        self.render_pass = create_render_pass(&self.device, &self.swapchain_format);
        let (graphics_pipeline, pipeline_layout) = VulkanApp::create_graphics_pipeline(&self.device, &self.render_pass, &swapchain_stuff.swapchain_extent);
        self.graphics_pipeline = graphics_pipeline;
        self.pipeline_layout = pipeline_layout;

        self.swapchain_framebuffers = create_framebuffers(&self.device, &self.render_pass, &self.swapchain_imageviews, &self.swapchain_extent);
        self.command_buffers = VulkanApp::create_command_buffers(&self.device, &self.command_pool, &self.graphics_pipeline, &self.swapchain_framebuffers, &self.render_pass, &self.swapchain_extent, &self.vertex_buffer, &self.index_buffer);
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
}

impl Drop for VulkanApp {

    fn drop(&mut self) {

        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device.destroy_semaphore(self.image_available_semaphores[i], None);
                self.device.destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.in_flight_fences[i], None);
            }

            self.cleanup_swapchain();

            self.device.destroy_buffer(self.index_buffer, None);
            self.device.free_memory(self.index_buffer_memory, None);

            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);

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

struct ProgramProc {

    events_loop: EventsLoop,
}

impl ProgramProc {

    fn new() -> ProgramProc {

        // init window stuff
        let events_loop = EventsLoop::new();

        ProgramProc {
            events_loop,
        }
    }

    fn main_loop(&mut self, vulkan_app: &mut VulkanApp) {

        let mut is_first_toggle_resize = true;
        let mut tick_counter = utility::fps_limiter::FPSLimiter::new();
        let mut is_running = true;

        'mainloop: loop {
            self.events_loop.poll_events(|event| {
                match event {
                    // handling keyboard event
                    | Event::WindowEvent { event, .. } => match event {
                        | WindowEvent::KeyboardInput { input, .. } => {
                            if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                                is_running = false;
                            }
                        }
                        | WindowEvent::Resized(_) => {
                            if is_first_toggle_resize == false {
                                vulkan_app.is_framebuffer_resized = true;
                            } else {
                                is_first_toggle_resize = false;
                            }
                        },
                        | WindowEvent::CloseRequested => {
                            is_running = false;
                        },
                        | _ => (),
                    },
                    | _ => (),
                }
            });

            vulkan_app.draw_frame();

            tick_counter.tick_frame();
            if IS_PAINT_FPS_COUNTER {
                print!("FPS: {}\r", tick_counter.fps());
            }

            if is_running == false {
                break 'mainloop
            }
        }

        vulkan_app.device.device_wait_idle()
            .expect("Failed to wait device idle!");
    }
}

fn main() {

    let mut program_proc = ProgramProc::new();
    let mut vulkan_app = VulkanApp::new(&program_proc.events_loop);

    program_proc.main_loop(&mut vulkan_app);
}
// -------------------------------------------------------------------------------------------
