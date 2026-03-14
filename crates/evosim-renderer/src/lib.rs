//! # evosim-renderer
//!
//! Bevy-based real-time visualizer for the **evosim** evolution simulator.
//!
//! Load a saved champion, build a creature, and call [`run_renderer`] to open
//! an interactive window showing the creature moving in real time.

pub mod app;
mod background;
mod camera;
mod hud;
mod input;
mod render;
mod simulation;
mod trails;

pub use app::run_renderer;
