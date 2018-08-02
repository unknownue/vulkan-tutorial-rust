
extern crate winit;

use winit::{ Event, WindowEvent, ControlFlow, VirtualKeyCode };

const WINDOW_TITLE: &'static str = "00.Base Code";
const WINDOW_WIDTH:  u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn main() {
    let mut events_loop = winit::EventsLoop::new();

    let _window = winit::WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_dimensions((WINDOW_WIDTH, WINDOW_HEIGHT).into())
        .build(&events_loop)
        .expect("Failed to create window.");

    events_loop.run_forever(|event| {

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
                | WindowEvent::CloseRequested => winit::ControlFlow::Break,
                | _ => ControlFlow::Continue,
            },
            | _ => winit::ControlFlow::Continue,
        }
    });
}
