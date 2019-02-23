use winit::{ControlFlow, Event, EventsLoop, VirtualKeyCode, WindowEvent};

// Constants
const WINDOW_TITLE: &'static str = "00.Base Code";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

struct VulkanApp {
    // winit stuff
    events_loop: EventsLoop,
    _window: winit::Window,
}

impl VulkanApp {
    pub fn new() -> VulkanApp {
        let events_loop = EventsLoop::new();
        let window = VulkanApp::init_window(&events_loop);

        VulkanApp {
            events_loop,
            _window: window,
        }
    }

    fn init_window(events_loop: &EventsLoop) -> winit::Window {
        winit::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_dimensions((WINDOW_WIDTH, WINDOW_HEIGHT).into())
            .build(events_loop)
            .expect("Failed to create window.")
    }

    pub fn main_loop(&mut self) {
        self.events_loop.run_forever(|event| {
            match event {
                // handling keyboard event
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            ControlFlow::Break
                        } else {
                            ControlFlow::Continue
                        }
                    }
                    WindowEvent::CloseRequested => ControlFlow::Break,
                    _ => ControlFlow::Continue,
                },
                _ => ControlFlow::Continue,
            }
        });
    }
}

fn main() {
    let mut vulkan_app = VulkanApp::new();
    vulkan_app.main_loop();
}
