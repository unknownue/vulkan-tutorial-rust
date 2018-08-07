# vulkan-tutorial-rust

A Rust implementation of the [Vulkan Tutorial](https://vulkan-tutorial.com) based on [ash crate](https://crates.io/crates/ash).

The codes are only tested on macOS.

It's recommended to first compile the examples of [ash](https://github.com/MaikKlein/ash) before compiling this repository.

## Status

Currently finish work on [16-swap-chain-recreation](https://vulkan-tutorial.com/Drawing_a_triangle/Swap_chain_recreation).

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

### example usage

```
$ cargo run --bin 00_base_code
```

