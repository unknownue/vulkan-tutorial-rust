
#[allow(non_camel_case_types)]
type c_char = i8; // define in 'libc' crate

/// Helper function to convert [c_char; SIZE] to string
pub fn convert_string(raw_string_array: &[c_char]) -> String {
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
