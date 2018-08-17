
extern crate vulkan_tutorial_rust;
use vulkan_tutorial_rust::{
    utility, // the mod define some fixed functions that have been learned before.
    utility::share,
    utility::debug::*,
    utility::constants::*,
};

extern crate winit;
extern crate ash;

use winit::{ Event, EventsLoop, WindowEvent, ControlFlow, VirtualKeyCode };
use ash::vk;
use ash::version::{ V1_0, InstanceV1_0 };
use ash::version::DeviceV1_0;

type EntryV1 = ash::Entry<V1_0>;

use std::ptr;

// Constants
const WINDOW_TITLE: &'static str = "07.Image View";

struct VulkanApp {
    // winit stuff
    events_loop          : EventsLoop,
    _window              : winit::Window,

    // vulkan stuff
    _entry               : EntryV1,
    instance             : ash::Instance<V1_0>,
    surface_loader       : ash::extensions::Surface,
    surface              : vk::SurfaceKHR,
    debug_report_loader  : ash::extensions::DebugReport,
    debug_callback       : vk::DebugReportCallbackEXT,

    _physical_device     : vk::PhysicalDevice,
    device               : ash::Device<V1_0>,

    _graphics_queue      : vk::Queue,
    _present_queue       : vk::Queue,

    swapchain_loader     : ash::extensions::Swapchain,
    swapchain            : vk::SwapchainKHR,
    _swapchain_images    : Vec<vk::Image>,
    _swapchain_format    : vk::Format,
    _swapchain_extent    : vk::Extent2D,
    swapchain_imageviews : Vec<vk::ImageView>,
}

impl VulkanApp {

    pub fn new() -> VulkanApp {

        // init window stuff
        let events_loop = EventsLoop::new();
        let window = utility::window::init_window(&events_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        // init vulkan stuff
        let entry = EntryV1::new().unwrap();
        let instance = share::create_instance(&entry, WINDOW_TITLE, VALIDATION.is_enable, &VALIDATION.required_validation_layers.to_vec());
        let surface_stuff = share::create_surface(&entry, &instance, &window, WINDOW_WIDTH, WINDOW_HEIGHT);
        let (debug_report_loader, debug_callback) = setup_debug_callback(VALIDATION.is_enable, &entry, &instance);
        let physical_device = share::pick_physical_device(&instance, &surface_stuff, &DEVICE_EXTENSIONS);
        let (device, family_indices) = share::create_logical_device(&instance, physical_device, &VALIDATION, &DEVICE_EXTENSIONS, &surface_stuff);
        let graphics_queue = unsafe { device.get_device_queue(family_indices.graphics_family as u32, 0) };
        let present_queue  = unsafe { device.get_device_queue(family_indices.present_family as u32, 0) };
        let swapchain_stuff = share::create_swapchain(&instance, &device, physical_device, &window, &surface_stuff, &family_indices);
        let swapchain_imageviews = VulkanApp::create_image_views(&device, swapchain_stuff.swapchain_format, &swapchain_stuff.swapchain_images);

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
            _present_queue : present_queue,

            swapchain_loader : swapchain_stuff.swapchain_loader,
            swapchain        : swapchain_stuff.swapchain,
            _swapchain_format: swapchain_stuff.swapchain_format,
            _swapchain_images: swapchain_stuff.swapchain_images,
            _swapchain_extent: swapchain_stuff.swapchain_extent,
            swapchain_imageviews,
        }
    }

    fn create_image_views(device: &ash::Device<V1_0>, surface_format: vk::Format, images: &Vec<vk::Image>) ->Vec<vk::ImageView> {

        let mut swapchain_imageviews = vec![];

        for &image in images.iter() {

            let imageview_create_info = vk::ImageViewCreateInfo {
                s_type     : vk::StructureType::ImageViewCreateInfo,
                p_next     : ptr::null(),
                flags      : vk::ImageViewCreateFlags::empty(),
                view_type  : vk::ImageViewType::Type2d,
                format     : surface_format,
                components : vk::ComponentMapping {
                    r: vk::ComponentSwizzle::Identity,
                    g: vk::ComponentSwizzle::Identity,
                    b: vk::ComponentSwizzle::Identity,
                    a: vk::ComponentSwizzle::Identity,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask      : vk::IMAGE_ASPECT_COLOR_BIT,
                    base_mip_level   : 0,
                    level_count      : 1,
                    base_array_layer : 0,
                    layer_count      : 1,
                },
                image,
            };

            let imageview = unsafe {
                device.create_image_view(&imageview_create_info, None)
                    .expect("Failed to create Image View!")
            };
            swapchain_imageviews.push(imageview);
        }

        swapchain_imageviews
    }
}

impl Drop for VulkanApp {

    fn drop(&mut self) {

        unsafe {

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
