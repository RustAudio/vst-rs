# vst-rs
[![crates.io][crates-img]][crates-url]
[![dependency status](https://deps.rs/repo/github/rustaudio/vst-rs/status.svg)](https://deps.rs/repo/github/rustaudio/vst-rs)
[![Discord Chat][discord-img]][discord-url]
[![Discourse topics][dc-img]][dc-url]

`vst-rs` is a library for creating VST2 plugins in the Rust programming language.

This library is a work in progress, and as such it does not yet implement all
functionality. It can create basic VST plugins without an editor interface.

**Note:** If you are upgrading from a version prior to 0.2.0, you will need to update
your plugin code to be compatible with the new, thread-safe plugin API. See the
[`transfer_and_smooth`](examples/transfer_and_smooth.rs) example for a guide on how
to port your plugin.

## Library Documentation

Documentation for **released** versions can be found [here](https://docs.rs/vst/).

Development documentation (current `master` branch) can be found [here](https://rustaudio.github.io/vst-rs/vst/).

## Crate
This crate is available on [crates.io](https://crates.io/crates/vst).  If you prefer the bleeding-edge, you can also
include the crate directly from the official [Github repository](https://github.com/rustaudio/vst-rs).

```toml
# get from crates.io.
vst = "0.2.1"
```
```toml
# get directly from Github.  This might be unstable!
vst = { git = "https://github.com/rustaudio/vst-rs" }
```

## Usage
To create a plugin, simply create a type which implements the `Plugin` trait. Then call the `plugin_main` macro, which will export the necessary functions and handle dealing with the rest of the API.

## Example Plugin
A simple plugin that bears no functionality. The provided `Cargo.toml` has a
`crate-type` directive which builds a dynamic library, usable by any VST host.

`src/lib.rs`

```rust
#[macro_use]
extern crate vst;

use vst::plugin::{HostCallback, Info, Plugin};

#[derive(Default)]
struct BasicPlugin;

impl Plugin for BasicPlugin {
    fn new(_host: HostCallback) -> Self {
        BasicPlugin
    }

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
vst = { git = "https://github.com/rustaudio/vst-rs" }

[lib]
name = "basicvst"
crate-type = ["cdylib"]
```

[crates-img]: https://img.shields.io/crates/v/vst.svg
[crates-url]: https://crates.io/crates/vst
[discord-img]: https://img.shields.io/discord/590254806208217089.svg?label=Discord&logo=discord&color=blue
[discord-url]: https://discord.gg/QPdhk2u
[dc-img]: https://img.shields.io/discourse/https/rust-audio.discourse.group/topics.svg?logo=discourse&color=blue
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
