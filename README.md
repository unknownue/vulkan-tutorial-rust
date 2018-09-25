# vulkan-tutorial-rust

A Rust implementation of the [Vulkan Tutorial](https://vulkan-tutorial.com) based on [ash crate](https://crates.io/crates/ash).

[![Build Status](https://travis-ci.org/Usami-Renko/vulkan-tutorial-rust.svg?branch=master)](https://travis-ci.org/Usami-Renko/vulkan-tutorial-rust)[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

The codes have been tested on macOS and Windows.

The API of ash crate is more similar to C++ when compared with [Vulkano crate](https://crates.io/crates/vulkano).

If you prefer using vulkano, [vulkan-tutorial-rs](https://github.com/bwasty/vulkan-tutorial-rs) may be a better choose.

## Main Dependencies

- [Vulkan SDK](https://vulkan.lunarg.com/sdk/home)
- [ash](https://github.com/MaikKlein/ash)

## Configuration

The environment configuration is same as [ash](https://github.com/MaikKlein/ash#example).

It's recommended to compile the examples of ash first before compiling this repository.

## Status

Vulkan is quit hard. All examples on official website have been finished. :beers:

This repository will maintain when the website updates.

## Usage

Enter the project root directory and run any example with the following command:

```shell
$ cargo run --bin example_number
```

Here replace `example_number` with option in the following table:

| example_number | Code                             | Reference                                                    | Note                                |
| -------------- | -------------------------------- | ------------------------------------------------------------ | ----------------------------------- |
| 00             | 00_base_code.rs                  | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Base_code) |                                     |
| 01             | 01_instance_creation.rs          | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Instance) |                                     |
| 02             | 02_validation_layers.rs          | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Validation_layers) |                                     |
| 03             | 03_physical_device_selection.rs  | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Physical_devices_and_queue_families) |                                     |
| 04             | 04_logical_device.rs             | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Logical_device_and_queues) |                                     |
| 05             | 05_window_surface.rs             | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Window_surface) |                                     |
| 06             | 06_swap_chain_creation.rs        | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Swap_chain) |                                     |
| 07             | 07_image_view.rs                 | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Image_views) |                                     |
| 08             | 08_graphics_pipeline.rs          | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics) |                                     |
| 09             | 09_shader_modules.rs             | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Shader_modules) |                                     |
| 10             | 10_fixed_functions.rs            | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Fixed_functions) |                                     |
| 11             | 11_render_passes.rs              | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Render_passes) |                                     |
| 12             | 12_graphics_pipeline_complete.rs | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Conclusion) |                                     |
| 13             | 13_framebuffers.rs               | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Framebuffers) |                                     |
| 14             | 14_command_buffers.rs            | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Command_buffers) |                                     |
| 15             | 15_hello_triangle.rs             | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Rendering_and_presentation) |                                     |
| 16             | 16_swap_chain_recreation.rs      | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Swap_chain_recreation) |                                     |
| 17             | 17_vertex_input.rs               | [Link](https://vulkan-tutorial.com/Vertex_buffers/Vertex_input_description) | The Validation Layer will complain. |
| 18             | 18_vertex_buffer.rs              | [Link](https://vulkan-tutorial.com/Vertex_buffers/Vertex_buffer_creation) |                                     |
| 19             | 19_staging_buffer.rs             | [Link](https://vulkan-tutorial.com/Vertex_buffers/Staging_buffer) |                                     |
| 20             | 20_index_buffer.rs               | [Link](https://vulkan-tutorial.com/Vertex_buffers/Index_buffer) |                                     |
| 21             | 21_descriptor_layout.rs          | [Link](https://vulkan-tutorial.com/Uniform_buffers/Descriptor_layout_and_buffer) | The Validation Layer will complain. |
| 22             | 22_descriptor_sets.rs            | [Link](https://vulkan-tutorial.com/Uniform_buffers/Descriptor_pool_and_sets) |                                     |
| 23             | 23_texture_image.rs              | [Link](https://vulkan-tutorial.com/Texture_mapping/Images)   |                                     |
| 24             | 24_sampler.rs                    | [Link](https://vulkan-tutorial.com/Texture_mapping/Image_view_and_sampler) |                                     |
| 25             | 25_texture_mapping.rs            | [Link](https://vulkan-tutorial.com/Texture_mapping/Combined_image_sampler) |                                     |
| 26             | 26_depth_buffering.rs            | [Link](https://vulkan-tutorial.com/Depth_buffering)          |                                     |
| 27             | 27_model_loading.rs              | [Link](https://vulkan-tutorial.com/Loading_models)           | Test this example in release mode.  |
| 28             | 28_mipmapping.rs                 | [Link](https://vulkan-tutorial.com/Generating_Mipmaps)       | Test this example in release mode.  |
| 29             | 29_multisampling.rs              | [Link](https://vulkan-tutorial.com/Multisampling)            | Test this example in release mode.  |

### example usage

```
$ cargo run --bin 00
```

## Notices

- All textures and models using in the tutorials are not included in this repository. Please place those resources in the `assets` directory.
- Use `VK_FORMAT_R32G32B32A32_SFLOAT` instead of `VK_FORMAT_R32G32B32_SFLOAT`, since `VK_FORMAT_R32G32B32_SFLOAT` is not available on macOS.
- It's recommended to use `vk::PresentModeKHR::Fifo` as present mode instead of `vk::PresentModeKHR::Immediate`, if you want to keep 60.0 fps.

## Assets
https://vulkan-tutorial.com/resources/chalet.obj.zip

https://vulkan-tutorial.com/resources/chalet.jpg

https://pixabay.com/en/statue-sculpture-fig-historically-1275469/


## Screen Shot

![screenshot_all](./screenshots/screenshot_all.png)
