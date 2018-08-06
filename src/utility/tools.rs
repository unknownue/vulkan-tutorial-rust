
use std::path::Path;

#[allow(non_camel_case_types)]
type c_char = i8; // define in 'libc' crate

/// Helper function to convert [c_char; SIZE] to string
pub fn vk_to_string(raw_string_array: &[c_char]) -> String {
    let end = '\0' as u8;

    let mut content: Vec<u8> = vec![];

    for ch in raw_string_array.iter() {
        let ch = (*ch) as u8;

        if ch != end {
            content.push(ch);
        } else {
            break
        }
    }

    String::from_utf8(content)
        .expect("Failed to convert vulkan raw string")
}

/// Helper function to convert string to [c_char; SIZE]
///
/// size of 256 char would be adequade for most of time
pub fn vk_to_raw_string(string_to_converted: &str) -> [c_char; 256] {

    let end = '\0' as c_char;
    let mut content = [end; 256];

    let raw_bytes = string_to_converted.as_bytes();

    for (index, byte) in raw_bytes.iter().enumerate() {
        content[index] = (*byte) as c_char;
    }

    if raw_bytes.len() > 256 {
        println!("Rust string to Vulkan string out of size.");
    }

    content
}

pub fn read_shader_code(shader_path: &Path) -> Vec<u8> {
    use std::fs::File;
    use std::io::Read;

    let spv_file = File::open(shader_path)
        .expect(&format!("Failed to find spv file at {:?}", shader_path));
    let bytes_code: Vec<u8> = spv_file.bytes()
        .filter_map(|byte| byte.ok()).collect();

    bytes_code
}



