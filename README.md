# vulkan-tutorial-rust

A Rust implementation of the [Vulkan Tutorial](https://vulkan-tutorial.com) based on [ash](https://crates.io/crates/ash) crate. The codes are only tested on macOS.

## Status

Currently finish work on [01-instance-creation](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Instance).

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

### example usage

```
$ cargo run --bin 00_base_code
```

