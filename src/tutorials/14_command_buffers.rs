use vulkan_tutorial_rust::{
    utility, // the mod define some fixed functions that have been learned before.
    utility::constants::*,
    utility::debug::*,
    utility::share,
    utility::structures::*,
};

use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;
use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};

use std::ptr;

// Constants
const WINDOW_TITLE: &'static str = "14.Command Buffers";

struct VulkanApp {

    _entry: ash::Entry,
    instance: ash::Instance,
    surface_loader: ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_merssager: vk::DebugUtilsMessengerEXT,

    _physical_device: vk::PhysicalDevice,
    device: ash::Device,

    _graphics_queue: vk::Queue,
    _present_queue: vk::Queue,

    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    _swapchain_images: Vec<vk::Image>,
    _swapchain_format: vk::Format,
    _swapchain_extent: vk::Extent2D,
    swapchain_imageviews: Vec<vk::ImageView>,
    swapchain_framebuffers: Vec<vk::Framebuffer>,

    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,

    command_pool: vk::CommandPool,
    _command_buffers: Vec<vk::CommandBuffer>,
}

impl VulkanApp {
    pub fn new(window: &winit::window::Window) -> VulkanApp {
       
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
        let (device, family_indices) = share::create_logical_device(
            &instance,
            physical_device,
            &VALIDATION,
            &DEVICE_EXTENSIONS,
            &surface_stuff,
        );
        let graphics_queue =
            unsafe { device.get_device_queue(family_indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(family_indices.present_family.unwrap(), 0) };
        let swapchain_stuff = share::create_swapchain(
            &instance,
            &device,
            physical_device,
            &window,
            &surface_stuff,
            &family_indices,
        );
        let swapchain_imageviews = share::v1::create_image_views(
            &device,
            swapchain_stuff.swapchain_format,
            &swapchain_stuff.swapchain_images,
        );
        let render_pass = share::v1::create_render_pass(&device, swapchain_stuff.swapchain_format);
        let (graphics_pipeline, pipeline_layout) = share::v1::create_graphics_pipeline(
            &device,
            render_pass,
            swapchain_stuff.swapchain_extent,
        );
        let swapchain_framebuffers = share::v1::create_framebuffers(
            &device,
            render_pass,
            &swapchain_imageviews,
            swapchain_stuff.swapchain_extent,
        );
        let command_pool = VulkanApp::create_command_pool(&device, &family_indices);
        let command_buffers = VulkanApp::create_command_buffers(
            &device,
            command_pool,
            graphics_pipeline,
            &swapchain_framebuffers,
            render_pass,
            swapchain_stuff.swapchain_extent,
        );

        // cleanup(); the 'drop' function will take care of it.
         VulkanApp {
            _entry: entry,
            instance,
            surface: surface_stuff.surface,
            surface_loader: surface_stuff.surface_loader,
            debug_utils_loader,
            debug_merssager,

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
            swapchain_framebuffers,

            pipeline_layout,
            render_pass,
            graphics_pipeline,

            command_pool,
            _command_buffers: command_buffers,
        }
    }

    fn create_command_pool(
        device: &ash::Device,
        queue_families: &QueueFamilyIndices,
    ) -> vk::CommandPool {
        let command_pool_create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::CommandPoolCreateFlags::empty(),
            queue_family_index: queue_families.graphics_family.unwrap(),
        };

        unsafe {
            device
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create Command Pool!")
        }
    }

    fn create_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        graphics_pipeline: vk::Pipeline,
        framebuffers: &Vec<vk::Framebuffer>,
        render_pass: vk::RenderPass,
        surface_extent: vk::Extent2D,
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

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }];

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
                device.cmd_draw(command_buffer, 3, 1, 0, 0);

                device.cmd_end_render_pass(command_buffer);

                device
                    .end_command_buffer(command_buffer)
                    .expect("Failed to record Command Buffer at Ending!");
            }
        }

        command_buffers
    }

    fn draw_frame(&mut self) {
        // Drawing will be here
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);

            for &framebuffer in self.swapchain_framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }

            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);

            for &imageview in self.swapchain_imageviews.iter() {
                self.device.destroy_image_view(imageview, None);
            }

            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
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

// Fix content -------------------------------------------------------------------------------
impl VulkanApp {
    pub fn main_loop(mut self, event_loop: EventLoop<()>, window: winit::window::Window) {

         event_loop.run(move |event, _, control_flow| {

            match event {
                | Event::WindowEvent { event, .. } => {
                    match event {
                        | WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit
                        },
                        | WindowEvent::KeyboardInput { input, .. } => {
                            match input {
                                | KeyboardInput { virtual_keycode, state, .. } => {
                                    match (virtual_keycode, state) {
                                        | (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                            *control_flow = ControlFlow::Exit
                                        },
                                        | _ => {},
                                    }
                                },
                            }
                        },
                        | _ => {},
                    }
                },
                | Event::MainEventsCleared => {
                    window.request_redraw();
                },
                | Event::RedrawRequested(_window_id) => {
                    self.draw_frame();
                },
                _ => (),
            }

        })
    }
}


fn main() {
    let event_loop = EventLoop::new();
    let window = utility::window::init_window(&event_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

    let vulkan_app = VulkanApp::new(&window);
    vulkan_app.main_loop(event_loop, window);
}
// -------------------------------------------------------------------------------------------
