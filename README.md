# rust-vst
<!--- [![Travis Build][trav-img]][trav-url] --->
<!--- [![Appveyor Build][appv-img]][appv-url] --->
<!--- [![crates.io][crates-img]][crates-url] --->

A library to help facilitate creating VST plugins in rust.

This library is a work in progress and as such does not yet implement all
opcodes. It is enough to create basic VST plugins without an editor interface.

## Library Documentation
  * https://rust-dsp.github.io/rust-vst

## TODO
  - Implement all opcodes
  - Proper editor support
  - Write more tests
  - Provide better examples

## Usage
To create a plugin, simply create a type which implements `plugin::Plugin` and
`std::default::Default`. Then call the macro `plugin_main!`, which will export
the necessary functions and handle dealing with the rest of the API.

## Example Plugin
A simple plugin that bears no functionality. The provided Cargo.toml has a
crate-type directive which builds a dynamic library, usable by any VST host.

`src/lib.rs`

```rust
#[macro_use]
extern crate vst2;

use vst2::plugin::{Info, Plugin};

#[derive(Default)]
struct BasicPlugin;

impl Plugin for BasicPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: "Basic Plugin".to_string(),
            unique_id: 1357, // Used by hosts to differentiate between plugins.

            ..Default::default()
        }
    }
}

plugin_main!(BasicPlugin); // Important!
```

`Cargo.toml`

```toml
[package]
name = "basic_vst"
version = "0.0.1"
authors = ["Author <author@example.com>"]

[dependencies]
vst2 = "0.0.1"

[lib]
name = "basicvst"
crate-type = ["cdylib"]
```

[trav-img]: https://travis-ci.org/rust-dsp/rust-vst.svg?branch=master
[trav-url]: https://travis-ci.org/rust-dsp/rust-vst
[appv-img]: https://ci.appveyor.com/api/projects/status/x25bmbwxqnsvy3ql?svg=true
[appv-url]: https://ci.appveyor.com/project/rustdsp/rust-vst
[crates-img]: https://img.shields.io/crates/v/vst2.svg
[crates-url]: https://crates.io/crates/vst2

#### Packaging on OS X

On OS X VST plugins are packaged inside of loadable bundles. 
To package your VST as a loadable bundle you may use the `osx_vst_bundler.sh` script this library provides. 

Example: 

```
./osx_vst_bundler.sh Plugin target/release/plugin.dylib
Creates a Plugin.vst bundle
```

## Special Thanks
[Marko Mijalkovic](https://github.com/overdrivenpotato) for [initiating this project](https://github.com/overdrivenpotato/rust-vst2)
