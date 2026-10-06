#![allow(unexpected_cfgs)]
// Native builds keep the package inspectable; the plugin is built by cargo-skyline.
#[cfg(target_os = "switch")]
mod plugin;

pub mod bounds;
