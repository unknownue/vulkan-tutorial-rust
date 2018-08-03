
use winit;
use winit::EventsLoop;

pub fn init_window(events_loop: &EventsLoop, title: &str, width: u32, height: u32) -> winit::Window {

    winit::WindowBuilder::new()
        .with_title(title)
        .with_dimensions((width, height).into())
        .build(events_loop)
        .expect("Failed to create window.")
}