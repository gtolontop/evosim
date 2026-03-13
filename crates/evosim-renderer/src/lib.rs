//! # evosim-renderer
//!
//! Optional Bevy-based visualizer for the **evosim** evolution simulator.
//!
//! This crate reads [`evosim_core::World`] state and renders creatures,
//! the ground plane, and UI overlays using the Bevy game engine.

use bevy::prelude::*;

/// Bevy plugin that adds evosim rendering systems to an [`App`].
///
/// Insert this plugin into your Bevy app to visualize the simulation.
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
/// use evosim_renderer::RendererPlugin;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(RendererPlugin)
///     .run();
/// ```
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: register rendering systems
    }
}
