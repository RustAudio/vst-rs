#![allow(clippy::mut_from_ref)]
#![deny(missing_docs, unused_imports)]

//! A rust implementation of the VST2.4 API.
//!
//! The VST API is multi-threaded. A VST host calls into a plugin generally from two threads -
//! the *processing* thread and the *UI* thread. The organization of this crate reflects this
//! structure to ensure that the threading assumptions of Safe Rust are fulfilled and data
//! races are avoided.
//!
//! # Plugins
//! All Plugins must implement the `Plugin` trait and `std::default::Default`.
//! The `plugin_main!` macro must also be called in order to export the necessary functions
//! for the plugin to function.
//!
//! ## `Plugin` Trait
//! All methods in this trait have a default implementation except for the `get_info` method which
//! must be implemented by the plugin. Any of the default implementations may be overriden for
//! custom functionality; the defaults do nothing on their own.
//!
//! ## `PluginParameters` Trait
//! The methods in this trait handle access to plugin parameters. Since the host may call these
//! methods concurrently with audio processing, it needs to be separate from the main `Plugin`
//! trait.
//!
//! To support parameters, a plugin must provide an implementation of the `PluginParameters`
//! trait, wrap it in an `Arc` (so it can be accessed from both threads) and
//! return a reference to it from the `get_parameter_object` method in the `Plugin`.
//!
//! ## `plugin_main!` macro
//! `plugin_main!` will export the necessary functions to create a proper VST plugin. This must be
//! called with your VST plugin struct name in order for the vst to work.
//!
//! ## Example plugin
//! A barebones VST plugin:
//!
//! ```no_run
//! #[macro_use]
//! extern crate vst;
//!
//! use vst::plugin::{HostCallback, Info, Plugin};
//!
//! struct BasicPlugin;
//!
//! impl Plugin for BasicPlugin {
//!     fn new(_host: HostCallback) -> Self {
//!         BasicPlugin
//!     }
//!
//!     fn get_info(&self) -> Info {
//!         Info {
//!             name: "Basic Plugin".to_string(),
//!             unique_id: 1357, // Used by hosts to differentiate between plugins.
//!
//!             ..Default::default()
//!         }
//!     }
//! }
//!
//! plugin_main!(BasicPlugin); // Important!
//! # fn main() {} // For `extern crate vst`
//! ```
//!
//! # Hosts
//!
//! ## `Host` Trait
//! All hosts must implement the [`Host` trait](host/trait.Host.html). To load a VST plugin, you
//! need to wrap your host in an `Arc<Mutex<T>>` wrapper for thread safety reasons. Along with the
//! plugin path, this can be passed to the [`PluginLoader::load`] method to create a plugin loader
//! which can spawn plugin instances.
//!
//! ## Example Host
//! ```no_run
//! extern crate vst;
//!
//! use std::sync::{Arc, Mutex};
//! use std::path::Path;
//!
//! use vst::host::{Host, PluginLoader};
//! use vst::plugin::Plugin;
//!
//! struct SampleHost;
//!
//! impl Host for SampleHost {
//!     fn automate(&self, index: i32, value: f32) {
//!         println!("Parameter {} had its value changed to {}", index, value);
//!     }
//! }
//!
//! fn main() {
//!     let host = Arc::new(Mutex::new(SampleHost));
//!     let path = Path::new("/path/to/vst");
//!
//!     let mut loader = PluginLoader::load(path, host.clone()).unwrap();
//!     let mut instance = loader.instance().unwrap();
//!
//!     println!("Loaded {}", instance.get_info().name);
//!
//!     instance.init();
//!     println!("Initialized instance!");
//!
//!     println!("Closing instance...");
//!     // Not necessary as the instance is shut down when it goes out of scope anyway.
//!     // drop(instance);
//! }
//!
//! ```
//!
//! [`PluginLoader::load`]: host/struct.PluginLoader.html#method.load
//!

extern crate libc;
extern crate libloading;
extern crate num_traits;
#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;

#[macro_use]
mod macros;
mod cache;
mod interfaces;

pub mod api;
pub mod buffer;
pub mod channels;
pub mod editor;
pub mod event;
pub mod host;
pub mod init;
pub mod plugin;

pub mod util;
