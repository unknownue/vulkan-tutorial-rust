
extern crate vulkan_tutorial_rust;
use vulkan_tutorial_rust::{
    utility, // the mod define some fixed functions that have been learned before.
    utility::debug::*,
    utility::vulkan::*,
    utility::constants::*,
};

extern crate winit;
extern crate ash;
extern crate num;

use winit::{ Event, EventsLoop, WindowEvent, ControlFlow, VirtualKeyCode };
use ash::vk;
use ash::version::{ V1_0, InstanceV1_0 };
use ash::version::DeviceV1_0;

type EntryV1 = ash::Entry<V1_0>;

use std::ptr;

// Constants
const WINDOW_TITLE: &'static str = "15.Hello Triangle";
const MAX_FRAMES_IN_FLIGHT: usize = 2;

struct SyncObjects {
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    inflight_fences: Vec<vk::Fence>,
}

struct VulkanApp {

    // vulkan stuff
    _entry: EntryV1,
    instance: ash::Instance<V1_0>,
    surface_loader: ash::extensions::Surface,
    surface: vk::SurfaceKHR,
    debug_report_loader: ash::extensions::DebugReport,
    debug_callback: vk::DebugReportCallbackEXT,

    _physical_device: vk::PhysicalDevice,
    device: ash::Device<V1_0>,

    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    swapchain_loader: ash::extensions::Swapchain,
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
    command_buffers: Vec<vk::CommandBuffer>,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
}

impl VulkanApp {

    pub fn new(window: &winit::Window) -> VulkanApp {

        // init vulkan stuff
        let entry = EntryV1::new().unwrap();
        let instance = create_instance(&entry, WINDOW_TITLE, VALIDATION.is_enable, &VALIDATION.required_validation_layers.to_vec());
        let surface_stuff = create_surface(&entry, &instance, &window, WINDOW_WIDTH, WINDOW_HEIGHT);
        let (debug_report_loader, debug_callback) = setup_debug_callback( VALIDATION.is_enable, &entry, &instance);
        let physical_device = pick_physical_device(&instance, &surface_stuff, &DEVICE_EXTENSIONS);
        let (device, family_indices) = create_logical_device(&instance, physical_device, &VALIDATION, &DEVICE_EXTENSIONS, &surface_stuff);
        let graphics_queue = unsafe { device.get_device_queue(family_indices.graphics_family as u32, 0) };
        let present_queue  = unsafe { device.get_device_queue(family_indices.present_family as u32, 0) };
        let swapchain_stuff = create_swapchain(&instance, &device, physical_device, &window, &surface_stuff, &family_indices);
        let swapchain_imageviews = create_image_view(&device, swapchain_stuff.swapchain_format, &swapchain_stuff.swapchain_images);
        let render_pass = create_render_pass(&device, swapchain_stuff.swapchain_format);
        let (graphics_pipeline, pipeline_layout) = create_graphics_pipeline(&device, render_pass, swapchain_stuff.swapchain_extent);
        let swapchain_framebuffers = create_framebuffers(&device, render_pass, &swapchain_imageviews, swapchain_stuff.swapchain_extent);
        let command_pool = create_command_pool(&device, &family_indices);
        let command_buffers = create_command_buffers(&device, command_pool, graphics_pipeline, &swapchain_framebuffers, render_pass, swapchain_stuff.swapchain_extent);
        let sync_ojbects = VulkanApp::create_sync_objects(&device);

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp {

            // vulkan stuff
            _entry: entry,
            instance,
            surface: surface_stuff.surface,
            surface_loader: surface_stuff.surface_loader,
            debug_report_loader,
            debug_callback,

            _physical_device: physical_device,
            device,

            graphics_queue,
            present_queue,

            swapchain_loader:  swapchain_stuff.swapchain_loader,
            swapchain:         swapchain_stuff.swapchain,
            _swapchain_format: swapchain_stuff.swapchain_format,
            _swapchain_images: swapchain_stuff.swapchain_images,
            _swapchain_extent: swapchain_stuff.swapchain_extent,
            swapchain_imageviews,
            swapchain_framebuffers,

            pipeline_layout,
            render_pass,
            graphics_pipeline,

            command_pool,
            command_buffers,

            image_available_semaphores: sync_ojbects.image_available_semaphores,
            render_finished_semaphores: sync_ojbects.render_finished_semaphores,
            in_flight_fences:           sync_ojbects.inflight_fences,
            current_frame: 0,
        }
    }

    fn draw_frame(&mut self) {

        let image_index;
        let wait_fences = [
            self.in_flight_fences[self.current_frame]
        ];

        unsafe {
            self.device.wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
            self.device.reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            image_index = self.swapchain_loader.acquire_next_image_khr(self.swapchain, std::u64::MAX, self.image_available_semaphores[self.current_frame], vk::Fence::null())
                .expect("Failed to acquire next image.");
        }

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

        unsafe {
            self.swapchain_loader.queue_present_khr(self.present_queue, &present_info)
                .expect("Failed to execute queue present.");
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn create_sync_objects(device: &ash::Device<V1_0>) -> SyncObjects {

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

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
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
}

impl Drop for VulkanApp {

    fn drop(&mut self) {

        unsafe {

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device.destroy_semaphore(self.image_available_semaphores[i], None);
                self.device.destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.in_flight_fences[i], None);
            }

            self.device.destroy_command_pool(self.command_pool, None);

            for &framebuffer in self.swapchain_framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }

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
struct ProgramProc {
    // winit stuff
    events_loop: EventsLoop,
    window: winit::Window,
}

impl ProgramProc {

    fn new() -> ProgramProc {

        // init window stuff
        let events_loop = EventsLoop::new();
        let window = utility::window::init_window(&events_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        ProgramProc {
            events_loop,
            window,
        }
    }

    fn main_loop(&mut self, vulkan_app: &mut VulkanApp) {

        self.events_loop.run_forever(|event| {

            match event {
                // handling keyboard event
                | Event::WindowEvent { event, .. } => match event {
                    | WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            return ControlFlow::Break
                        }
                    }
                    | WindowEvent::CloseRequested => return ControlFlow::Break,
                    | _ => (),
                },
                | _ => (),
            }

            vulkan_app.draw_frame();
            ControlFlow::Continue
        });

        vulkan_app.device.device_wait_idle()
            .expect("Failed to wait device idle!");
    }
}

fn main() {

    let mut program_proc = ProgramProc::new();
    let mut vulkan_app = VulkanApp::new(&program_proc.window);

    program_proc.main_loop(&mut vulkan_app);
}
// -------------------------------------------------------------------------------------------
