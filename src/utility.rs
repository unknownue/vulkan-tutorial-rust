
use ash::extensions::{ Surface, MacOSSurface, DebugReport };

// ------------------------------------------------------------------------
#[cfg(target_os = "macos")]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        MacOSSurface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

//#[cfg(all(windows))]
//pub fn required_extension_names() -> Vec<*const i8> {
//    vec![
//        Surface::name().as_ptr(),
//        Win32Surface::name().as_ptr(),
//        DebugReport::name().as_ptr(),
//    ]
//}

//#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
//pub fn required_extension_names() -> Vec<*const i8> {
//    vec![
//        Surface::name().as_ptr(),
//        XlibSurface::name().as_ptr(),
//        DebugReport::name().as_ptr(),
//    ]
//}
// ------------------------------------------------------------------------
