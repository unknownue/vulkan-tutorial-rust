
use ash;
use ash::vk;
use ash::version::{ V1_0, DeviceV1_0 };
use ash::vk::types::{ uint32_t, int32_t };
use image;
use image::GenericImage;

use std::ptr;
use std::ffi::CString;
use std::path::Path;
use std::cmp::max;

use ::utility::tools;

use super::*;

pub fn create_render_pass(device: &ash::Device<V1_0>, surface_format: vk::Format) -> vk::RenderPass {

    let color_attachment = vk::AttachmentDescription {
        format           : surface_format,
        flags            : vk::AttachmentDescriptionFlags::empty(),
        samples          : vk::SAMPLE_COUNT_1_BIT,
        load_op          : vk::AttachmentLoadOp::Clear,
        store_op         : vk::AttachmentStoreOp::Store,
        stencil_load_op  : vk::AttachmentLoadOp::DontCare,
        stencil_store_op : vk::AttachmentStoreOp::DontCare,
        initial_layout   : vk::ImageLayout::Undefined,
        final_layout     : vk::ImageLayout::PresentSrcKhr,
    };

    let color_attachment_ref = vk::AttachmentReference {
        attachment : 0,
        layout     : vk::ImageLayout::ColorAttachmentOptimal,
    };

    let subpasses = [
        vk::SubpassDescription {
            color_attachment_count     : 1,
            p_color_attachments        : &color_attachment_ref,
            p_depth_stencil_attachment : ptr::null(),
            flags                      : vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point        : vk::PipelineBindPoint::Graphics,
            input_attachment_count     : 0,
            p_input_attachments        : ptr::null(),
            p_resolve_attachments      : ptr::null(),
            preserve_attachment_count  : 0,
            p_preserve_attachments     : ptr::null(),
        },
    ];

    let render_pass_attachments = [
        color_attachment,
    ];

    let subpass_dependencies = [
        vk::SubpassDependency {
            src_subpass      : vk::VK_SUBPASS_EXTERNAL,
            dst_subpass      : 0,
            src_stage_mask   : vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
            dst_stage_mask   : vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
            src_access_mask  : vk::AccessFlags::empty(),
            dst_access_mask  : vk::ACCESS_COLOR_ATTACHMENT_READ_BIT | vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
            dependency_flags : vk::DependencyFlags::empty(),
        },
    ];

    let renderpass_create_info = vk::RenderPassCreateInfo {
        s_type           : vk::StructureType::RenderPassCreateInfo,
        flags            : vk::RenderPassCreateFlags::empty(),
        p_next           : ptr::null(),
        attachment_count : render_pass_attachments.len() as u32,
        p_attachments    : render_pass_attachments.as_ptr(),
        subpass_count    : subpasses.len() as u32,
        p_subpasses      : subpasses.as_ptr(),
        dependency_count : subpass_dependencies.len() as u32,
        p_dependencies   : subpass_dependencies.as_ptr(),
    };

    unsafe {
        device.create_render_pass(&renderpass_create_info, None)
            .expect("Failed to create render pass!")
    }
}

pub fn create_graphics_pipeline(device: &ash::Device<V1_0>, render_pass: vk::RenderPass, swapchain_extent: vk::Extent2D) -> (vk::Pipeline, vk::PipelineLayout) {

    let vert_shader_code = tools::read_shader_code(Path::new("shaders/spv/09-shader-base.vert.spv"));
    let frag_shader_code = tools::read_shader_code(Path::new("shaders/spv/09-shader-base.frag.spv"));

    let vert_shader_module = create_shader_module(device, vert_shader_code);
    let frag_shader_module = create_shader_module(device, frag_shader_code);

    let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

    let shader_stages = [
        vk::PipelineShaderStageCreateInfo { // Vertex Shader
            s_type                : vk::StructureType::PipelineShaderStageCreateInfo,
            p_next                : ptr::null(),
            flags                 : vk::PipelineShaderStageCreateFlags::empty(),
            module                : vert_shader_module,
            p_name                : main_function_name.as_ptr(),
            p_specialization_info : ptr::null(),
            stage                 : vk::SHADER_STAGE_VERTEX_BIT,
        },
        vk::PipelineShaderStageCreateInfo { // Fragment Shader
            s_type                : vk::StructureType::PipelineShaderStageCreateInfo,
            p_next                : ptr::null(),
            flags                 : vk::PipelineShaderStageCreateFlags::empty(),
            module                : frag_shader_module,
            p_name                : main_function_name.as_ptr(),
            p_specialization_info : ptr::null(),
            stage                 : vk::SHADER_STAGE_FRAGMENT_BIT,
        },
    ];

    let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
        s_type                             : vk::StructureType::PipelineVertexInputStateCreateInfo,
        p_next                             : ptr::null(),
        flags                              : vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_attribute_description_count : 0,
        p_vertex_attribute_descriptions    : ptr::null(),
        vertex_binding_description_count   : 0,
        p_vertex_binding_descriptions      : ptr::null(),
    };
    let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
        s_type                   : vk::StructureType::PipelineInputAssemblyStateCreateInfo,
        flags                    : vk::PipelineInputAssemblyStateCreateFlags::empty(),
        p_next                   : ptr::null(),
        primitive_restart_enable : vk::VK_FALSE,
        topology                 : vk::PrimitiveTopology::TriangleList,
    };

    let viewports = [
        vk::Viewport {
            x         : 0.0,
            y         : 0.0,
            width     : swapchain_extent.width  as f32,
            height    : swapchain_extent.height as f32,
            min_depth : 0.0,
            max_depth : 1.0,
        },
    ];

    let scissors = [
        vk::Rect2D {
            offset : vk::Offset2D { x: 0, y: 0 },
            extent : swapchain_extent,
        },
    ];

    let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
        s_type         : vk::StructureType::PipelineViewportStateCreateInfo,
        p_next         : ptr::null(),
        flags          : vk::PipelineViewportStateCreateFlags::empty(),
        scissor_count  : scissors.len()  as u32,
        p_scissors     : scissors.as_ptr(),
        viewport_count : viewports.len() as u32,
        p_viewports    : viewports.as_ptr(),
    };

    let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo {
        s_type                     : vk::StructureType::PipelineRasterizationStateCreateInfo,
        p_next                     : ptr::null(),
        flags                      : vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable         : vk::VK_FALSE,
        cull_mode                  : vk::CULL_MODE_BACK_BIT,
        front_face                 : vk::FrontFace::Clockwise,
        line_width                 : 1.0,
        polygon_mode               : vk::PolygonMode::Fill,
        rasterizer_discard_enable  : vk::VK_FALSE,
        depth_bias_clamp           : 0.0,
        depth_bias_constant_factor : 0.0,
        depth_bias_enable          : vk::VK_FALSE,
        depth_bias_slope_factor    : 0.0,
    };
    let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
        s_type                   : vk::StructureType::PipelineMultisampleStateCreateInfo,
        flags                    : vk::PipelineMultisampleStateCreateFlags::empty(),
        p_next                   : ptr::null(),
        rasterization_samples    : vk::SAMPLE_COUNT_1_BIT,
        sample_shading_enable    : vk::VK_FALSE,
        min_sample_shading       : 0.0,
        p_sample_mask            : ptr::null(),
        alpha_to_one_enable      : vk::VK_FALSE,
        alpha_to_coverage_enable : vk::VK_FALSE,
    };

    let stencil_state = vk::StencilOpState {
        fail_op       : vk::StencilOp::Keep,
        pass_op       : vk::StencilOp::Keep,
        depth_fail_op : vk::StencilOp::Keep,
        compare_op    : vk::CompareOp::Always,
        compare_mask  : 0,
        write_mask    : 0,
        reference     : 0,
    };

    let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo {
        s_type                   : vk::StructureType::PipelineDepthStencilStateCreateInfo,
        p_next                   : ptr::null(),
        flags                    : vk::PipelineDepthStencilStateCreateFlags::empty(),
        depth_test_enable        : vk::VK_FALSE,
        depth_write_enable       : vk::VK_FALSE,
        depth_compare_op         : vk::CompareOp::LessOrEqual,
        depth_bounds_test_enable : vk::VK_FALSE,
        stencil_test_enable      : vk::VK_FALSE,
        front                    : stencil_state,
        back                     : stencil_state,
        max_depth_bounds         : 1.0,
        min_depth_bounds         : 0.0,
    };

    let color_blend_attachment_states = [
        vk::PipelineColorBlendAttachmentState {
            blend_enable           : vk::VK_FALSE,
            color_write_mask       : vk::ColorComponentFlags::all(),
            src_color_blend_factor : vk::BlendFactor::One,
            dst_color_blend_factor : vk::BlendFactor::Zero,
            color_blend_op         : vk::BlendOp::Add,
            src_alpha_blend_factor : vk::BlendFactor::One,
            dst_alpha_blend_factor : vk::BlendFactor::Zero,
            alpha_blend_op         : vk::BlendOp::Add,
        },
    ];

    let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
        s_type           : vk::StructureType::PipelineColorBlendStateCreateInfo,
        p_next           : ptr::null(),
        flags            : vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable  : vk::VK_FALSE,
        logic_op         : vk::LogicOp::Copy,
        attachment_count : color_blend_attachment_states.len() as u32,
        p_attachments    : color_blend_attachment_states.as_ptr(),
        blend_constants  : [0.0, 0.0, 0.0, 0.0],
    };

    //        leaving the dynamic statue unconfigurated right now
    //        let dynamic_state = [vk::DynamicState::Viewport, vk::DynamicState::Scissor];
    //        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
    //            s_type: vk::StructureType::PipelineDynamicStateCreateInfo,
    //            p_next: ptr::null(),
    //            flags: vk::PipelineDynamicStateCreateFlags::empty(),
    //            dynamic_state_count: dynamic_state.len() as u32,
    //            p_dynamic_states: dynamic_state.as_ptr(),
    //        };

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
        s_type                    : vk::StructureType::PipelineLayoutCreateInfo,
        p_next                    : ptr::null(),
        flags                     : vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count          : 0,
        p_set_layouts             : ptr::null(),
        push_constant_range_count : 0,
        p_push_constant_ranges    : ptr::null(),
    };

    let pipeline_layout = unsafe {
        device.create_pipeline_layout(&pipeline_layout_create_info, None)
            .expect("Failed to create pipeline layout!")
    };

    let graphic_pipeline_create_infos = [
        vk::GraphicsPipelineCreateInfo {
            s_type                 : vk::StructureType::GraphicsPipelineCreateInfo,
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

pub fn create_framebuffers(device: &ash::Device<V1_0>, render_pass: vk::RenderPass, image_views: &Vec<vk::ImageView>, swapchain_extent: vk::Extent2D) -> Vec<vk::Framebuffer> {

    let mut framebuffers = vec![];

    for &image_view in image_views.iter() {
        let attachments = [
            image_view
        ];

        let framebuffer_create_info = vk::FramebufferCreateInfo {
            s_type           : vk::StructureType::FramebufferCreateInfo,
            p_next           : ptr::null(),
            flags            : vk::FramebufferCreateFlags::empty(),
            render_pass,
            attachment_count : attachments.len() as u32,
            p_attachments    : attachments.as_ptr(),
            width            : swapchain_extent.width,
            height           : swapchain_extent.height,
            layers           : 1,
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
        s_type             : vk::StructureType::CommandPoolCreateInfo,
        p_next             : ptr::null(),
        flags              : vk::CommandPoolCreateFlags::empty(),
        queue_family_index : queue_families.graphics_family as u32,
    };

    unsafe {
        device.create_command_pool(&command_pool_create_info, None)
            .expect("Failed to create Command Pool!")
    }
}

pub fn create_command_buffers(device: &ash::Device<V1_0>, command_pool: vk::CommandPool, graphics_pipeline: vk::Pipeline, framebuffers: &Vec<vk::Framebuffer>, render_pass: vk::RenderPass, surface_extent: vk::Extent2D) -> Vec<vk::CommandBuffer> {

    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
        s_type               : vk::StructureType::CommandBufferAllocateInfo,
        p_next               : ptr::null(),
        command_buffer_count : framebuffers.len() as u32,
        command_pool,
        level                : vk::CommandBufferLevel::Primary,
    };

    let command_buffers = unsafe {
        device.allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    };

    for (i, &command_buffer) in command_buffers.iter().enumerate() {

        let command_buffer_begin_info  = vk::CommandBufferBeginInfo {
            s_type             : vk::StructureType::CommandBufferBeginInfo,
            p_next             : ptr::null(),
            p_inheritance_info : ptr::null(),
            flags              : vk::COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT,
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
            s_type            : vk::StructureType::RenderPassBeginInfo,
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
            device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::Inline);
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::Graphics, graphics_pipeline);
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
        inflight_fences           : vec![],
    };

    let semaphore_create_info = vk::SemaphoreCreateInfo {
        s_type : vk::StructureType::SemaphoreCreateInfo,
        p_next : ptr::null(),
        flags  : vk::SemaphoreCreateFlags::empty(),
    };

    let fence_create_info = vk::FenceCreateInfo {
        s_type : vk::StructureType::FenceCreateInfo,
        p_next : ptr::null(),
        flags  : vk::FENCE_CREATE_SIGNALED_BIT,
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

pub fn create_vertex_buffer<T: Copy>(device: &ash::Device<V1_0>, device_memory_properties: &vk::PhysicalDeviceMemoryProperties, command_pool: vk::CommandPool, submit_queue: vk::Queue, data: &[T])
                                     -> (vk::Buffer, vk::DeviceMemory) {

    let buffer_size = ::std::mem::size_of_val(data) as vk::DeviceSize;;

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
        let mut vert_align = ash::util::Align::new(data_ptr, ::std::mem::align_of::<T>() as u64, buffer_size);
        vert_align.copy_from_slice(data);
        device.unmap_memory(staging_buffer_memory);
    }

    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        device,
        buffer_size,
        vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_VERTEX_BUFFER_BIT,
        vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
        &device_memory_properties,
    );

    copy_buffer(device, submit_queue, command_pool, staging_buffer, vertex_buffer, buffer_size);

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    (vertex_buffer, vertex_buffer_memory)
}

pub fn create_index_buffer(device: &ash::Device<V1_0>, device_memory_properties: &vk::PhysicalDeviceMemoryProperties, command_pool: vk::CommandPool, submit_queue: vk::Queue, data: &[uint32_t])
                           -> (vk::Buffer, vk::DeviceMemory) {

    let buffer_size = ::std::mem::size_of_val(data) as vk::DeviceSize;

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
        let mut vert_align = ash::util::Align::new(data_ptr, ::std::mem::align_of::<uint32_t>() as u64, buffer_size);
        vert_align.copy_from_slice(data);
        device.unmap_memory(staging_buffer_memory);
    }

    let (index_buffer, index_buffer_memory) = create_buffer(
        device,
        buffer_size,
        vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_INDEX_BUFFER_BIT,
        vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
        &device_memory_properties,
    );

    copy_buffer(device, submit_queue, command_pool, staging_buffer, index_buffer, buffer_size);

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    (index_buffer, index_buffer_memory)
}

pub fn create_descriptor_pool(device: &ash::Device<V1_0>, swapchain_images_size: usize) -> vk::DescriptorPool {

    let pool_sizes = [
        vk::DescriptorPoolSize {
            typ              : vk::DescriptorType::UniformBuffer,
            descriptor_count : swapchain_images_size as u32
        }
    ];

    let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
        s_type          : vk::StructureType::DescriptorPoolCreateInfo,
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

pub fn create_descriptor_sets(device: &ash::Device<V1_0>, descriptor_pool: vk::DescriptorPool, descriptor_set_layout: vk::DescriptorSetLayout, uniforms_buffers: &Vec<vk::Buffer>, swapchain_images_size: usize) -> Vec<vk::DescriptorSet> {

    let mut layouts: Vec<vk::DescriptorSetLayout> = vec![];
    for _ in 0..swapchain_images_size {
        layouts.push(descriptor_set_layout);
    }

    let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
        s_type               : vk::StructureType::DescriptorSetAllocateInfo,
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
        let descriptor_buffer_info = [
            vk::DescriptorBufferInfo {
                buffer : uniforms_buffers[i],
                offset : 0,
                range  : ::std::mem::size_of::<UniformBufferObject>() as u64,
            },
        ];

        let descriptor_write_sets = [
            vk::WriteDescriptorSet {
                s_type              : vk::StructureType::WriteDescriptorSet,
                p_next              : ptr::null(),
                dst_set             : descritptor_set,
                dst_binding         : 0,
                dst_array_element   : 0,
                descriptor_count    : 1,
                descriptor_type     : vk::DescriptorType::UniformBuffer,
                p_image_info        : ptr::null(),
                p_buffer_info       : descriptor_buffer_info.as_ptr(),
                p_texel_buffer_view : ptr::null(),
            },
        ];

        unsafe {
            device.update_descriptor_sets(&descriptor_write_sets, &[]);
        }
    }

    descriptor_sets
}

pub fn create_descriptor_set_layout(device: &ash::Device<V1_0>) -> vk::DescriptorSetLayout {

    let ubo_layout_bindings = [
        vk::DescriptorSetLayoutBinding {
            binding              : 0,
            descriptor_type      : vk::DescriptorType::UniformBuffer,
            descriptor_count     : 1,
            stage_flags          : vk::SHADER_STAGE_VERTEX_BIT,
            p_immutable_samplers : ptr::null(),
        }
    ];

    let ubo_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
        s_type        : vk::StructureType::DescriptorSetLayoutCreateInfo,
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

pub fn create_uniform_buffers(device: &ash::Device<V1_0>, device_memory_properties: &vk::PhysicalDeviceMemoryProperties, swapchain_image_count: usize) -> (Vec<vk::Buffer>, Vec<vk::DeviceMemory>) {

    let buffer_size = ::std::mem::size_of::<UniformBufferObject>();

    let mut uniform_buffers = vec![];
    let mut uniform_buffers_memory = vec![];

    for _ in 0..swapchain_image_count {
        let (uniform_buffer, uniform_buffer_memory) = create_buffer(
            device,
            buffer_size as u64,
            vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT,
            vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT,
            device_memory_properties,
        );
        uniform_buffers.push(uniform_buffer);
        uniform_buffers_memory.push(uniform_buffer_memory);
    }

    (uniform_buffers, uniform_buffers_memory)
}

pub fn create_image(device: &ash::Device<V1_0>, width: uint32_t, height: uint32_t, mip_levels: uint32_t, num_samples: vk::SampleCountFlags, format: vk::Format, tiling: vk::ImageTiling, usage: vk::ImageUsageFlags, required_memory_properties: vk::MemoryPropertyFlags, device_memory_properties: &vk::PhysicalDeviceMemoryProperties)
                    -> (vk::Image, vk::DeviceMemory) {

    let image_create_info = vk::ImageCreateInfo {
        s_type                   : vk::StructureType::ImageCreateInfo,
        p_next                   : ptr::null(),
        flags                    : vk::ImageCreateFlags::empty(),
        image_type               : vk::ImageType::Type2d,
        format,
        extent: vk::Extent3D {
            width,
            height,
            depth: 1,
        },
        mip_levels,
        array_layers             : 1,
        samples                  : num_samples,
        tiling,
        usage,
        sharing_mode             : vk::SharingMode::Exclusive,
        queue_family_index_count : 0,
        p_queue_family_indices   : ptr::null(),
        initial_layout           : vk::ImageLayout::Undefined,
    };

    let texture_image = unsafe {
        device.create_image(&image_create_info, None)
            .expect("Failed to create Texture Image!")
    };

    let image_memory_requirement = device.get_image_memory_requirements(texture_image);
    let memory_allocate_info = vk::MemoryAllocateInfo {
        s_type            : vk::StructureType::MemoryAllocateInfo,
        p_next            : ptr::null(),
        allocation_size   : image_memory_requirement.size,
        memory_type_index : find_memory_type(image_memory_requirement.memory_type_bits, required_memory_properties, device_memory_properties)
    };

    let texture_image_memory = unsafe {
        device.allocate_memory(&memory_allocate_info, None)
            .expect("Failed to allocate Texture Image memory!")
    };

    unsafe {
        device.bind_image_memory(texture_image, texture_image_memory, 0)
            .expect("Failed to bind Image Memmory!");
    }

    (texture_image, texture_image_memory)
}

pub fn transition_image_layout(device: &ash::Device<V1_0>, command_pool: vk::CommandPool, submit_queue: vk::Queue, image: vk::Image, format: vk::Format, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout, mip_levels: uint32_t) {

    let command_buffer = begin_single_time_command(device, command_pool);

    let src_access_mask;
    let dst_access_mask;
    let source_stage;
    let destination_stage;

    if old_layout == vk::ImageLayout::Undefined && new_layout == vk::ImageLayout::TransferDstOptimal {

        src_access_mask   = vk::AccessFlags::empty();
        dst_access_mask   = vk::ACCESS_TRANSFER_WRITE_BIT;
        source_stage      = vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT;
        destination_stage = vk::PIPELINE_STAGE_TRANSFER_BIT;
    } else if old_layout == vk::ImageLayout::TransferDstOptimal && new_layout == vk::ImageLayout::ShaderReadOnlyOptimal {

        src_access_mask   = vk::ACCESS_TRANSFER_WRITE_BIT;
        dst_access_mask   = vk::ACCESS_SHADER_READ_BIT;
        source_stage      = vk::PIPELINE_STAGE_TRANSFER_BIT;
        destination_stage = vk::PIPELINE_STAGE_FRAGMENT_SHADER_BIT;
    } else if old_layout == vk::ImageLayout::Undefined && new_layout == vk::ImageLayout::DepthStencilAttachmentOptimal {

        src_access_mask   = vk::AccessFlags::empty();
        dst_access_mask   = vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT | vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT;
        source_stage      = vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT;
        destination_stage = vk::PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT;
    } else if old_layout == vk::ImageLayout::Undefined && new_layout == vk::ImageLayout::ColorAttachmentOptimal {

        src_access_mask   = vk::AccessFlags::empty();
        dst_access_mask   = vk::ACCESS_COLOR_ATTACHMENT_READ_BIT | vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT;
        source_stage      = vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT;
        destination_stage = vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
    } else {
        panic!("Unsupported layout transition!")
    }

    let aspect_mask = if new_layout == vk::ImageLayout::DepthStencilAttachmentOptimal {
        if has_stencil_component(format) {
            vk::IMAGE_ASPECT_DEPTH_BIT | vk::IMAGE_ASPECT_STENCIL_BIT
        } else {
            vk::IMAGE_ASPECT_DEPTH_BIT
        }
    } else {
        vk::IMAGE_ASPECT_COLOR_BIT
    };

    let image_barriers = [
        vk::ImageMemoryBarrier {
            s_type: vk::StructureType::ImageMemoryBarrier,
            p_next: ptr::null(),
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
            src_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
            image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level   : 0,
                level_count      : mip_levels,
                base_array_layer : 0,
                layer_count      : 1,
            }
        },
    ];

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            source_stage, destination_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &image_barriers
        );
    }

    end_single_time_command(device, command_pool, submit_queue, command_buffer);
}


pub fn create_image_views(device: &ash::Device<V1_0>, surface_format: vk::Format, images: &Vec<vk::Image>) ->Vec<vk::ImageView> {

    let swapchain_imageviews: Vec<vk::ImageView> = images.iter().map(|&image| {
        create_image_view(device, image, surface_format, vk::IMAGE_ASPECT_COLOR_BIT, 1)
    }).collect();

    swapchain_imageviews
}

pub fn create_image_view(device: &ash::Device<V1_0>, image: vk::Image, format: vk::Format, aspect_flags: vk::ImageAspectFlags, mip_levels: uint32_t) -> vk::ImageView {

    let imageview_create_info = vk::ImageViewCreateInfo {
        s_type    : vk::StructureType::ImageViewCreateInfo,
        p_next    : ptr::null(),
        flags     : vk::ImageViewCreateFlags::empty(),
        view_type : vk::ImageViewType::Type2d,
        format,
        components: vk::ComponentMapping {
            r: vk::ComponentSwizzle::Identity,
            g: vk::ComponentSwizzle::Identity,
            b: vk::ComponentSwizzle::Identity,
            a: vk::ComponentSwizzle::Identity,
        },
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask      : aspect_flags,
            base_mip_level   : 0,
            level_count      : mip_levels,
            base_array_layer : 0,
            layer_count      : 1,
        },
        image,
    };

    unsafe {
        device.create_image_view(&imageview_create_info, None)
            .expect("Failed to create Image View!")
    }
}

pub fn create_texture_image_view(device: &ash::Device<V1_0>, texture_image: vk::Image, mip_levels: uint32_t) -> vk::ImageView {

    create_image_view(device, texture_image, vk::Format::R8g8b8a8Unorm, vk::IMAGE_ASPECT_COLOR_BIT, mip_levels)
}

pub fn create_texture_sampler(device: &ash::Device<V1_0>) -> vk::Sampler {

    let sampler_create_info = vk::SamplerCreateInfo {
        s_type                   : vk::StructureType::SamplerCreateInfo,
        p_next                   : ptr::null(),
        flags                    : vk::SamplerCreateFlags::empty(),
        mag_filter               : vk::Filter::Linear,
        min_filter               : vk::Filter::Linear,
        address_mode_u           : vk::SamplerAddressMode::Repeat,
        address_mode_v           : vk::SamplerAddressMode::Repeat,
        address_mode_w           : vk::SamplerAddressMode::Repeat,
        anisotropy_enable        : vk::VK_TRUE,
        max_anisotropy           : 16.0,
        compare_enable           : vk::VK_FALSE,
        compare_op               : vk::CompareOp::Always,
        mipmap_mode              : vk::SamplerMipmapMode::Linear,
        min_lod                  : 0.0,
        max_lod                  : 0.0,
        mip_lod_bias             : 0.0,
        border_color             : vk::BorderColor::IntOpaqueBlack,
        unnormalized_coordinates : vk::VK_FALSE,
    };

    unsafe {
        device.create_sampler(&sampler_create_info, None)
            .expect("Failed to create Sampler!")
    }
}

pub fn create_texture_image(device: &ash::Device<V1_0>, command_pool: vk::CommandPool, submit_queue: vk::Queue, device_memory_properties: &vk::PhysicalDeviceMemoryProperties, image_path: &Path) -> (vk::Image, vk::DeviceMemory) {

    let mut image_object = image::open(image_path).unwrap(); // this function is slow in debug mode.
    image_object = image_object.flipv();
    let (image_width, image_height) = (image_object.width(), image_object.height());
    let image_data = match &image_object {
        | image::DynamicImage::ImageLuma8(_)
        | image::DynamicImage::ImageRgb8(_) => image_object.to_rgba().into_raw(),
        | image::DynamicImage::ImageLumaA8(_)
        | image::DynamicImage::ImageRgba8(_) => image_object.raw_pixels(),
    };
    let image_size = (::std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;

    if image_size <= 0 {
        panic!("Failed to load texture image!")
    }

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        device,
        image_size,
        vk::BUFFER_USAGE_TRANSFER_SRC_BIT,
        vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT,
        device_memory_properties
    );

    unsafe {
        let data_ptr = device.map_memory(staging_buffer_memory, 0, image_size, vk::MemoryMapFlags::empty())
            .expect("Failed to Map Memory");
        let mut align = ash::util::Align::new(data_ptr, ::std::mem::align_of::<u8>() as u64, image_size);
        align.copy_from_slice(&image_data);
        device.unmap_memory(staging_buffer_memory);
    }

    let (texture_image, texture_image_memory) = create_image(
        device,
        image_width, image_height,
        1,
        vk::SAMPLE_COUNT_1_BIT,
        vk::Format::R8g8b8a8Unorm,
        vk::ImageTiling::Optimal,
        vk::IMAGE_USAGE_TRANSFER_DST_BIT | vk::IMAGE_USAGE_SAMPLED_BIT,
        vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
        device_memory_properties
    );

    transition_image_layout(device, command_pool, submit_queue, texture_image, vk::Format::R8g8b8a8Unorm, vk::ImageLayout::Undefined, vk::ImageLayout::TransferDstOptimal, 1);

    copy_buffer_to_image(device, command_pool, submit_queue, staging_buffer, texture_image, image_width, image_height);

    transition_image_layout(device, command_pool, submit_queue, texture_image, vk::Format::R8g8b8a8Unorm, vk::ImageLayout::TransferDstOptimal, vk::ImageLayout::ShaderReadOnlyOptimal, 1);

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    (texture_image, texture_image_memory)
}

pub fn create_depth_resources(instance: &ash::Instance<V1_0>, device: &ash::Device<V1_0>, physical_device: vk::PhysicalDevice, command_pool: vk::CommandPool, submit_queue: vk::Queue, swapchain_extent: vk::Extent2D, device_memory_properties: &vk::PhysicalDeviceMemoryProperties, msaa_samples: vk::SampleCountFlags) -> (vk::Image, vk::ImageView, vk::DeviceMemory) {

    let depth_format = find_depth_format(instance, physical_device);
    let (depth_image, depth_image_memory) = create_image(
        device,
        swapchain_extent.width, swapchain_extent.height,
        1,
        msaa_samples,
        depth_format,
        vk::ImageTiling::Optimal,
        vk::IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT,
        vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
        device_memory_properties
    );
    let depth_image_view = create_image_view(device, depth_image, depth_format, vk::IMAGE_ASPECT_DEPTH_BIT, 1);

    transition_image_layout(device, command_pool, submit_queue, depth_image, depth_format, vk::ImageLayout::Undefined, vk::ImageLayout::DepthStencilAttachmentOptimal, 1);

    (depth_image, depth_image_view, depth_image_memory)
}



pub fn generate_mipmaps(device: &ash::Device<V1_0>, command_pool: vk::CommandPool, submit_queue: vk::Queue, image: vk::Image, tex_width: u32, tex_height: u32, mip_levels: uint32_t) {

    let command_buffer = begin_single_time_command(device, command_pool);

    let mut image_barrier = vk::ImageMemoryBarrier {
        s_type                 : vk::StructureType::ImageMemoryBarrier,
        p_next                 : ptr::null(),
        src_access_mask        : vk::AccessFlags::empty(),
        dst_access_mask        : vk::AccessFlags::empty(),
        old_layout             : vk::ImageLayout::Undefined,
        new_layout             : vk::ImageLayout::Undefined,
        src_queue_family_index : vk::VK_QUEUE_FAMILY_IGNORED,
        dst_queue_family_index : vk::VK_QUEUE_FAMILY_IGNORED,
        image,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask      : vk::IMAGE_ASPECT_COLOR_BIT,
            base_mip_level   : 0,
            level_count      : 1,
            base_array_layer : 0,
            layer_count      : 1,
        }
    };

    let mut mip_width  = tex_width  as int32_t;
    let mut mip_height = tex_height as int32_t;

    for i in 1..mip_levels {

        image_barrier.subresource_range.base_mip_level = i - 1;
        image_barrier.old_layout      = vk::ImageLayout::TransferDstOptimal;
        image_barrier.new_layout      = vk::ImageLayout::TransferSrcOptimal;
        image_barrier.src_access_mask = vk::ACCESS_TRANSFER_WRITE_BIT;
        image_barrier.dst_access_mask = vk::ACCESS_TRANSFER_READ_BIT;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PIPELINE_STAGE_TRANSFER_BIT,
                vk::PIPELINE_STAGE_TRANSFER_BIT,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier.clone()],
            );
        }

        let blits = [
            vk::ImageBlit {
                src_subresource: vk::ImageSubresourceLayers {
                    aspect_mask      : vk::IMAGE_ASPECT_COLOR_BIT,
                    mip_level        : i - 1,
                    base_array_layer : 0,
                    layer_count      : 1,
                },
                src_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D { x: mip_width, y: mip_height, z: 1 },
                ],
                dst_subresource: vk::ImageSubresourceLayers {
                    aspect_mask      : vk::IMAGE_ASPECT_COLOR_BIT,
                    mip_level        : i,
                    base_array_layer : 0,
                    layer_count      : 1,
                },
                dst_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: max(mip_width  / 2, 1),
                        y: max(mip_height / 2, 1),
                        z: 1
                    },
                ],
            },
        ];

        unsafe {
            device.cmd_blit_image(
                command_buffer,
                image,
                vk::ImageLayout::TransferSrcOptimal,
                image,
                vk::ImageLayout::TransferDstOptimal,
                &blits,
                vk::Filter::Linear
            );
        }

        image_barrier.old_layout      = vk::ImageLayout::TransferSrcOptimal;
        image_barrier.new_layout      = vk::ImageLayout::ShaderReadOnlyOptimal;
        image_barrier.src_access_mask = vk::ACCESS_TRANSFER_READ_BIT;
        image_barrier.dst_access_mask = vk::ACCESS_SHADER_READ_BIT;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PIPELINE_STAGE_TRANSFER_BIT,
                vk::PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier.clone()],
            );
        }

        mip_width  = max(mip_width  / 2, 1);
        mip_height = max(mip_height / 2, 1);
    }


    image_barrier.subresource_range.base_mip_level = mip_levels - 1;
    image_barrier.old_layout      = vk::ImageLayout::TransferDstOptimal;
    image_barrier.new_layout      = vk::ImageLayout::ShaderReadOnlyOptimal;
    image_barrier.src_access_mask = vk::ACCESS_TRANSFER_WRITE_BIT;
    image_barrier.dst_access_mask = vk::ACCESS_SHADER_READ_BIT;

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PIPELINE_STAGE_TRANSFER_BIT,
            vk::PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[image_barrier.clone()],
        );
    }

    end_single_time_command(device, command_pool, submit_queue, command_buffer);
}

