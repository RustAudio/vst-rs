# rust-vst2
A library to help facilitate creating VST plugins in rust.

This library is a work in progress and as such does not yet implement all opcodes. It is enough to create a basic VST plugins without an editor interface.

### TODO
  - Editor support
  - Implement all opcodes

## Usage
To create a plugin, simply create a type which implements `Vst` and `std::default::Default`. Then call the macro `vst_main!`, which will export the necessary functions and handle dealing with the rest of the API.

### Example plugin
A simple plugin that bears no functionality.
###### lib.rs

```rust
#[macro_use]
extern crate vst2;
use std::default::Default;

use vst2::{Vst, Info};

struct BasicVst {
    info: Info
}

impl Vst for BasicVst {
    fn get_info(&mut self) -> &mut Info {
        &mut self.info
    }
}

impl Default for BasicVst {
    fn default() -> BasicVst {
        BasicVst {
            info: Info {
                name: "BasicVst".to_string(),

                ..Default::default()
            }
        }
    }
}

vst_main!(BasicVst); //Important!
```

###### Cargo.toml

```toml
[package]
name = "basic_vst"
version = "0.0.1"
authors = ["Author <author@example.com>"]

[dependencies.vst2]
git = "https://github.com/overdrivenpotato/rust-vst2"

[lib]
name = "basicvst"
crate-type = ["dylib"]
```
