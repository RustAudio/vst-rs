# rust-vst
[![Travis Build][trav-img]][trav-url]
[![Appveyor Build][appv-img]][appv-url]
[![crates.io][crates-img]][crates-url]
[![dependency status](https://deps.rs/repo/github/rust-dsp/rust-vst/status.svg)](https://deps.rs/repo/github/rust-dsp/rust-vst)
[![Telegram Chat][tg-img]][tg-url]
[![Discourse topics][dc-img]][dc-url]

rust-vst is a library for creating VST plugins in the Rust programming language.

This library is a work in progress, and as such it does not yet implement all
functionality. It can create basic VST plugins without an editor interface.

For more detailed information about this library and subtopics such as GUI development progress, please check the [wiki](https://github.com/rust-dsp/rust-vst/wiki/).

## Library Documentation
  * https://rust-dsp.github.io/rust-vst
  
## Community
For questions, help, or other issues, consider joining our [Telegram Chat][tg-url].

## TODO
  - Implement all opcodes
  - Proper editor support
  - Write more tests
  - Provide better examples

## Crate
`VST` is available on [crates.io](https://crates.io/crates/vst).  If you prefer the bleeding-edge, you can also
include the crate directly from the official [Github repository](https://github.com/rust-dsp/rust-vst).

```toml
# get from crates.io.
vst = "0.1.0"
```
```toml
# get directly from Github.  This might be unstable!
vst = { git = "https://github.com/rust-dsp/rust-vst" }
```

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
extern crate vst;

use vst::plugin::{Info, Plugin};

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
vst = { git = "https://github.com/rust-dsp/rust-vst" }

[lib]
name = "basicvst"
crate-type = ["cdylib"]
```

[trav-img]: https://travis-ci.org/rust-dsp/rust-vst.svg?branch=master
[trav-url]: https://travis-ci.org/rust-dsp/rust-vst
[appv-img]: https://ci.appveyor.com/api/projects/status/npiyjfithlx50hfs?svg=true
[appv-url]: https://ci.appveyor.com/project/rustdsp/rust-vst
[crates-img]: https://img.shields.io/crates/v/vst.svg
[crates-url]: https://crates.io/crates/vst
[tg-img]: https://img.shields.io/badge/Telegram-Join%20Chat-blue.svg
[tg-url]: https://t.me/joinchat/BfEhnw0l4386Uzi5elmGrQ
[dc-img]: https://img.shields.io/discourse/https/rust-audio.discourse.group/topics.svg
[dc-url]: https://rust-audio.discourse.group

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
