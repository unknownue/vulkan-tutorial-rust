#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

#[macro_use]
extern crate ash;
extern crate winit;
extern crate num;
extern crate cgmath;
#[macro_use]
extern crate memoffset;
extern crate image;
extern crate tobj;


#[cfg(target_os = "macos")]
extern crate metal_rs;
#[cfg(target_os = "macos")]
extern crate cocoa;
#[cfg(target_os = "macos")]
extern crate objc;

pub mod utility;
