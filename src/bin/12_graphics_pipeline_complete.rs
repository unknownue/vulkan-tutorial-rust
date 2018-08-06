
extern crate vulkan_tutorial_rust;
use vulkan_tutorial_rust::{
    utility, // the mod define some fixed functions that have been learned before.
    utility::debug::*,
    utility::vulkan::*,
};

extern crate winit;
extern crate ash;
extern crate num;

use winit::{ Event, EventsLoop, WindowEvent, ControlFlow, VirtualKeyCode };
use ash::vk;
use ash::version::{ V1_0, InstanceV1_0 };
use ash::version::DeviceV1_0;

type EntryV1 = ash::Entry<V1_0>;

use std::path::Path;
use std::ptr;
use std::ffi::CString;

// Constants
const WINDOW_TITLE: &'static str = "10.Fixed Functions";
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
    swapchain_imageviews: Vec<vk::ImageView>,

    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,
}

impl VulkanApp {

    pub fn new() -> VulkanApp {

        // init window stuff
        let events_loop = EventsLoop::new();
        let window = utility::window::init_window(&events_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        // init vulkan stuff
        let entry = EntryV1::new().unwrap();
        let instance = create_instance(&entry, WINDOW_TITLE, VALIDATION.is_enable, &VALIDATION.required_validation_layers.to_vec());
        let surface_stuff = create_surface(&entry, &instance, &window, WINDOW_WIDTH, WINDOW_HEIGHT);
        let (debug_report_loader, debug_callback) = setup_debug_callback( VALIDATION.is_enable, &entry, &instance);
        let physical_device = pick_physical_device(&instance, &surface_stuff, &DEVICE_EXTENSIONS);
        let (device, family_indices) = create_logical_device(&instance, &physical_device, &VALIDATION, &DEVICE_EXTENSIONS, &surface_stuff);
        let graphics_queue = unsafe { device.get_device_queue(family_indices.graphics_family as u32, 0) };
        let present_queue  = unsafe { device.get_device_queue(family_indices.present_family as u32, 0) };
        let swapchain_stuff = create_swapchain(&instance, &device, &physical_device, &surface_stuff, &family_indices);
        let swapchain_imageviews = create_image_view(&device, &swapchain_stuff.swapchain_format, &swapchain_stuff.swapchain_images);
        let render_pass = VulkanApp::create_render_pass(&device, swapchain_stuff.swapchain_format);
        let (graphics_pipeline, pipeline_layout) = VulkanApp::create_graphics_pipeline(&device, &render_pass, &swapchain_stuff.swapchain_extent);

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
            swapchain_imageviews,

            pipeline_layout,
            render_pass,
            graphics_pipeline,
        }
    }

    fn create_graphics_pipeline(device: &ash::Device<V1_0>, render_pass: &vk::RenderPass, swapchain_extent: &vk::Extent2D) -> (vk::Pipeline, vk::PipelineLayout) {

        let vert_shader_code = utility::tools::read_shader_code(Path::new("shaders/vert.spv"));
        let frag_shader_code = utility::tools::read_shader_code(Path::new("shaders/frag.spv"));

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

    fn create_render_pass(device: &ash::Device<V1_0>, surface_format: vk::Format) -> vk::RenderPass {

        let color_attachment = vk::AttachmentDescription {
            format: surface_format,
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
}

impl Drop for VulkanApp {

    fn drop(&mut self) {

        unsafe {

            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);

            for &imageview in self.swapchain_imageviews.iter() {
                self.device.destroy_image_view(imageview, None);
            }

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
