#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod dock;
mod tab;
pub mod tasks;
pub use app::Graph2DataEguiApp;
type ImageBuf = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
