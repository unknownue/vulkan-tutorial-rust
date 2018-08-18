
use winit;
use ash::vk;
use ash::version::{ EntryV1_0, InstanceV1_0 };

use ash::extensions::{ Surface, DebugReport };
#[cfg(target_os = "macos")]
use ash::extensions::MacOSSurface;
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::XlibSurface;
#[cfg(target_os = "windows")]
use ash::extensions::Win32Surface;

#[cfg(target_os = "macos")]
use metal_rs::CoreAnimationLayer;
#[cfg(target_os = "macos")]
use cocoa::base::id as cocoa_id;
#[cfg(target_os = "macos")]
use cocoa::appkit::{ NSView, NSWindow };
#[cfg(target_os = "macos")]
use objc::runtime::YES;

// required extension ------------------------------------------------------
#[cfg(target_os = "macos")]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        MacOSSurface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

#[cfg(all(windows))]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        XlibSurface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}
// ------------------------------------------------------------------------

// create surface ---------------------------------------------------------
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &winit::Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use winit::os::unix::WindowExt;
    use std::ptr;

    let x11_display = window.get_xlib_display().unwrap();
    let x11_window = window.get_xlib_window().unwrap();
    let x11_create_info = vk::XlibSurfaceCreateInfoKHR {
        s_type : vk::StructureType::XlibSurfaceCreateInfoKhr,
        p_next : ptr::null(),
        flags  : Default::default(),
        window : x11_window as vk::Window,
        dpy    : x11_display as *mut vk::Display,
    };
    let xlib_surface_loader =
        XlibSurface::new(entry, instance).expect("Unable to load xlib surface");
    xlib_surface_loader.create_xlib_surface_khr(&x11_create_info, None)
}

#[cfg(target_os = "macos")]
pub unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &winit::Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use winit::os::macos::WindowExt;
    use std::mem;
    use std::ptr;

    let wnd: cocoa_id = mem::transmute(window.get_nswindow());

    let layer = CoreAnimationLayer::new();

    layer.set_edge_antialiasing_mask(0);
    layer.set_presents_with_transaction(false);
    layer.remove_all_animations();

    let view = wnd.contentView();

    layer.set_contents_scale(view.backingScaleFactor());
    view.setLayer(mem::transmute(layer.as_ref()));
    view.setWantsLayer(YES);

    let create_info = vk::MacOSSurfaceCreateInfoMVK {
        s_type : vk::StructureType::MacOSSurfaceCreateInfoMvk,
        p_next : ptr::null(),
        flags  : Default::default(),
        p_view : window.get_nsview() as *const vk::types::c_void
    };

    let macos_surface_loader =
        MacOSSurface::new(entry, instance).expect("Unable to load macOS surface");
    macos_surface_loader.create_macos_surface_mvk(&create_info, None)
}

#[cfg(target_os = "windows")]
pub unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &winit::Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::GetWindow;
    use winit::os::windows::WindowExt;
    use std::ptr;

    let hwnd = window.get_hwnd() as HWND;
    let hinstance = GetWindow(hwnd, 0) as *const vk::c_void;
    let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
        s_type    : vk::StructureType::Win32SurfaceCreateInfoKhr,
        p_next    : ptr::null(),
        flags     : Default::default(),
        hinstance,
        hwnd      : hwnd as *const vk::c_void,
    };
    let win32_surface_loader =
        Win32Surface::new(entry, instance).expect("Unable to load win32 surface");
    win32_surface_loader.create_win32_surface_khr(&win32_create_info, None)
}
// ------------------------------------------------------------------------
