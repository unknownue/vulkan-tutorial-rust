
extern crate winit;

use winit::{ Event, WindowEvent, ControlFlow, VirtualKeyCode };

// Constants
const WINDOW_TITLE: &'static str = "00.Base Code";
const WINDOW_WIDTH:  u32 = 800;
const WINDOW_HEIGHT: u32 = 600;


struct VulkanApp {
    // winit stuff
    events_loop: winit::EventsLoop,
    window: Option<winit::Window>,
}

impl VulkanApp {

    pub fn new() -> VulkanApp {
        VulkanApp {
            events_loop: winit::EventsLoop::new(),
            window: None,
        }
    }

    pub fn run(&mut self) {

        self.init_window();
        self.init_vulkan();
        self.main_loop();
        self.cleanup();
    }

    fn init_window(&mut self) {

        let window = winit::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_dimensions((WINDOW_WIDTH, WINDOW_HEIGHT).into())
            .build(&self.events_loop)
            .expect("Failed to create window.");

        self.window = Some(window);
    }

    fn init_vulkan(&mut self) {

    }

    fn main_loop(&mut self) {

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

    fn cleanup(&mut self) {

    }
}

fn main() {

    let mut vulkan_app = VulkanApp::new();
    vulkan_app.run();
}
