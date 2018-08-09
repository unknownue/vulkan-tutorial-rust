# vulkan-tutorial-rust

A Rust implementation of the [Vulkan Tutorial](https://vulkan-tutorial.com) based on [ash crate](https://crates.io/crates/ash).

The codes are only tested on macOS, but it should work on Windows or Linux in theory.

It's recommended to compile the examples of [ash](https://github.com/MaikKlein/ash) first before compiling this repository.

## Dependencies

- [Vulkan SDK](https://vulkan.lunarg.com/sdk/home)
- [ash](https://github.com/MaikKlein/ash)
- [winit](https://github.com/tomaka/winit)

## Status

Currently finish work on [20-index-buffer](https://vulkan-tutorial.com/Vertex_buffers/Index_buffer).

## Usage

Enter the project root directory and run any example like the following code:

```shell
$ cargo run --bin sample_name
```

Here replace `sample_name` with option in the following table:

| sample_name                   | Reference                                                    |
| ----------------------------- | ------------------------------------------------------------ |
| 00_base_code                  | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Base_code) |
| 01_instance_creation          | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Instance) |
| 02_validation_layers          | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Validation_layers) |
| 03_physical_device_selection  | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Physical_devices_and_queue_families) |
| 04_logical_device             | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Logical_device_and_queues) |
| 05_window_surface             | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Window_surface) |
| 06_swap_chain_creation        | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Swap_chain) |
| 07_image_view                 | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Image_views) |
| 08_graphics_pipeline          | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics) |
| 09_shader_modules             | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Shader_modules) |
| 10_fixed_functions            | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Fixed_functions) |
| 11_render_passes              | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Render_passes) |
| 12_graphics_pipeline_complete | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Conclusion) |
| 13_framebuffers               | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Framebuffers) |
| 14_command_buffers            | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Command_buffers) |
| 15_hello_triangle             | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Rendering_and_presentation) |
| 16_swap_chain_recreation      | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Swap_chain_recreation) |
| 17_vertex_input               | [Link](https://vulkan-tutorial.com/Vertex_buffers/Vertex_input_description) |
| 18_vertex_buffer              | [Link](https://vulkan-tutorial.com/Vertex_buffers/Vertex_buffer_creation) |
| 19_staging_buffer             | [Link](https://vulkan-tutorial.com/Vertex_buffers/Staging_buffer) |
| 20_index_buffer               | [Link](https://vulkan-tutorial.com/Vertex_buffers/Index_buffer) |

### example usage

```
$ cargo run --bin 00_base_code
```

## Notices

- Use `VK_FORMAT_R32G32B32A32_SFLOAT` instead of `VK_FORMAT_R32G32B32_SFLOAT`, since `VK_FORMAT_R32G32B32_SFLOAT` is not available on macOS.