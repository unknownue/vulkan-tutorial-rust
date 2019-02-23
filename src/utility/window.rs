
use winit::{ Event, EventsLoop, WindowEvent, VirtualKeyCode };

const IS_PAINT_FPS_COUNTER: bool = true;

pub fn init_window(events_loop: &EventsLoop, title: &str, width: u32, height: u32) -> winit::Window {

    winit::WindowBuilder::new()
        .with_title(title)
        .with_dimensions((width, height).into())
        .build(events_loop)
        .expect("Failed to create window.")
}


pub trait VulkanApp {

    fn draw_frame(&mut self, delta_time: f32);
    fn recreate_swapchain(&mut self);
    fn cleanup_swapchain(&self);
    fn wait_device_idle(&self);
    fn resize_framebuffer(&mut self);
}

pub struct ProgramProc {

    pub events_loop: EventsLoop,
}

impl ProgramProc {

    pub fn new() -> ProgramProc {

        // init window stuff
        let events_loop = EventsLoop::new();

        ProgramProc {
            events_loop,
        }
    }

    pub fn main_loop(&mut self, vulkan_app: &mut impl VulkanApp) {

        let mut is_first_toggle_resize = true;
        let mut tick_counter = super::fps_limiter::FPSLimiter::new();
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
                                vulkan_app.resize_framebuffer();
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

            let delta_time = tick_counter.delta_time();
            vulkan_app.draw_frame(delta_time);

            tick_counter.tick_frame();
            if IS_PAINT_FPS_COUNTER {
                print!("FPS: {}\r", tick_counter.fps());
            }

            if is_running == false {
                break 'mainloop
            }
        }

        vulkan_app.wait_device_idle();
    }
}
