//! `monster-engine` — the facade crate: everything a game needs in one
//! dependency.
//!
//! Games implement [`GameApp`] (init/update/render), configure the engine
//! through [`EngineBuilder`], and call [`Engine::run`]. The engine owns the
//! window, input, fixed-timestep clock, renderer, assets, audio and the
//! ECS world; game code stays plain functions over components.
//!
//! ```no_run
//! use monster_engine::prelude::*;
//!
//! struct MyGame { spawned: bool }
//!
//! impl GameApp for MyGame {
//!     fn init(&mut self, ctx: &mut InitContext) {
//!         ctx.assets.insert_rgba("white", 1, 1, vec![255, 255, 255, 255]);
//!         ctx.camera.zoom = 1.0;
//!     }
//!     fn update(&mut self, ctx: &mut UpdateContext<'_>) {
//!         if ctx.input.was_pressed(KeyCode::Escape) {
//!             ctx.exit();
//!         }
//!     }
//!     fn render(&mut self, _ctx: &mut RenderContext<'_>) {}
//! }
//!
//! let mut engine = EngineBuilder::new().headless().build().unwrap();
//! let mut game = MyGame { spawned: false };
//! engine.run(&mut game);
//! ```

pub mod engine;
pub mod physics_bridge;

pub use engine::{
    Engine, EngineBuilder, EngineConfig, EngineRunResult, GameApp, InitContext, RenderContext,
    UpdateContext,
};
pub use physics_bridge::{
    attach as physics_attach, sync_dynamic_to_scene, sync_static_to_physics, PhysicsLink,
};

/// Convenience re-export of the whole public API surface.
pub mod prelude {
    pub use monster_assets::{AssetError, Assets, ImageData};
    pub use monster_audio::NullAudioBackend;
    pub use monster_core::audio::{AudioBackend, AudioClip};
    pub use monster_core::events::{EngineEvent, EventBus};
    pub use monster_core::graphics::{Camera2D, DrawOptions, PresentTarget, Renderer, Texture};
    pub use monster_core::handle::Handle;
    pub use monster_core::input::{InputState, KeyCode, MouseButton};
    pub use monster_core::log::{set_max_level, Level};
    pub use monster_core::time::{FixedClock, Stopwatch};
    pub use monster_core::{log_debug, log_error, log_info, log_trace, log_warn};
    pub use monster_math::{Color, Mat4, Quat, Rect, Vec2, Vec3, Vec4};
    pub use monster_physics::{
        bounce, find_collisions, integrate, Aabb2, BodyKind2, ColliderShape2, CollisionPair,
        PhysicsBody2, PhysicsWorld2,
    };
    pub use monster_platform::{NullWindow, PlatformWindow, Window, WindowOptions};
    pub use monster_render::Software2DRenderer;
    pub use monster_scene::{
        movement_system, sprite_draw_order, EntityId, Sprite, Transform, Velocity, World, ZOrder,
    };

    pub use crate::engine::{
        Engine, EngineBuilder, EngineConfig, EngineRunResult, GameApp, InitContext, RenderContext,
        UpdateContext,
    };
    pub use crate::physics_attach;
    pub use crate::physics_bridge::PhysicsLink;
}
