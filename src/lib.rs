#[cfg(test)]
mod unit_tests;

pub mod core;
pub mod events;
pub mod examples;
pub mod input;
pub mod networking;
pub mod settings;
pub mod utils;
//#[allow(unused)] // Either rust is generating completely wrong warnings for this module or I'm just dumb, either way im putting this as a temporary fix
#[cfg(feature = "window")]
pub mod window;
pub mod world;
