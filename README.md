# vulkan-tutorial-rust

A Rust implementation of the [Vulkan Tutorial](https://vulkan-tutorial.com) based on [ash crate](https://crates.io/crates/ash).

The codes are only tested on macOS.

It's recommended to first compile the examples of [ash](https://github.com/MaikKlein/ash) before compiling this repository.

## Status

Currently finish work on [02-validation-layers](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Validation_layers).

## Usage

Enter the project root directory and run any example like the following code:

```shell
$ cargo run --bin sample_name
```

Here replace `sample_name` with option in the following table:

| sample_name          | Reference                                                    |
| -------------------- | ------------------------------------------------------------ |
| 00_base_code         | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Base_code) |
| 01_instance_creation | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Instance) |
| 02_validation_layers | [Link](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Validation_layers) |

### example usage

```
$ cargo run --bin 00_base_code
```

