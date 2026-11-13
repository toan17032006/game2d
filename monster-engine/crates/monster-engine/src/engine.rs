//! The engine facade: builder, run loop and the [`GameApp`] contract.

use std::time::Instant;

use monster_assets::Assets;
use monster_core::audio::AudioBackend;
use monster_core::events::{EngineEvent, EventBus};
use monster_core::graphics::{Camera2D, DrawOptions, Renderer};
use monster_core::input::InputState;
use monster_core::log_info;
use monster_core::time::FixedClock;
use monster_math::{Color, Rect, Vec2};
use monster_physics::world::PhysicsWorld2;
use monster_platform::{NullWindow, PlatformWindow, Window, WindowOptions};
use monster_render::Software2DRenderer;
use monster_scene::{sprite_draw_order, World};

/// Everything a game can touch during init.
pub struct InitContext<'a> {
    pub world: &'a mut World,
    pub assets: &'a mut Assets,
    pub audio: &'a mut dyn AudioBackend,
    pub camera: &'a mut Camera2D,
    /// Physics world when enabled via `EngineBuilder::with_physics`.
    pub physics: Option<&'a mut PhysicsWorld2>,
}

/// Per-fixed-step update context.
pub struct UpdateContext<'a> {
    pub world: &'a mut World,
    pub assets: &'a mut Assets,
    pub input: &'a InputState,
    pub audio: &'a mut dyn AudioBackend,
    pub camera: &'a mut Camera2D,
    /// Physics world when enabled via `EngineBuilder::with_physics`.
    pub physics: Option<&'a mut PhysicsWorld2>,
    /// Fixed delta seconds of this step.
    pub dt: f32,
    should_exit: &'a mut bool,
}

impl UpdateContext<'_> {
    /// Ask the engine to stop running after this step.
    pub fn exit(&mut self) {
        *self.should_exit = true;
    }
}

/// Per-frame render context (world is read-only here).
pub struct RenderContext<'a> {
    pub world: &'a World,
    pub renderer: &'a mut dyn Renderer,
    /// Clock interpolation fraction 0.0..1.0 for this frame.
    pub alpha: f32,
}

/// The game contract: plain trait, plain functions.
pub trait GameApp {
    /// Called once before the loop; load assets, spawn entities.
    fn init(&mut self, ctx: &mut InitContext<'_>);
    /// Fixed-timestep simulation step (`ctx.dt` is constant).
    fn update(&mut self, ctx: &mut UpdateContext<'_>);
    /// Per-frame rendering. The engine has already drawn the world's
    /// sprites (z-sorted); draw extra effects on top here.
    fn render(&mut self, ctx: &mut RenderContext<'_>);

    /// Called once after the loop ends.
    fn shutdown(&mut self) {
        let _ = ();
    }
}

/// Engine configuration collected by the builder.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// `Some` = real OS window; `None` = headless (NullWindow).
    pub window: Option<WindowOptions>,
    /// Fixed update rate (steps per second).
    pub fixed_fps: u32,
    /// Framebuffer clear color.
    pub clear_color: Color,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            window: None,
            fixed_fps: 60,
            clear_color: Color::opaque(24, 24, 32),
        }
    }
}

/// Fluent builder: configure subsystems without touching engine internals.
///
/// Adding a new backend later (GPU renderer, real audio device) is a new
/// `with_*` option here — nothing else in the engine changes.
pub struct EngineBuilder {
    config: EngineConfig,
    audio: Option<Box<dyn AudioBackend>>,
    /// Gravity for the physics world (None = physics disabled).
    physics_gravity: Option<Vec2>,
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineBuilder {
    pub fn new() -> EngineBuilder {
        EngineBuilder {
            config: EngineConfig::default(),
            audio: None,
            physics_gravity: None,
        }
    }

    /// Run with a real OS window.
    pub fn with_window(mut self, options: WindowOptions) -> EngineBuilder {
        self.config.window = Some(options);
        self
    }

    /// Run headless (no OS window; frames render into an offscreen buffer).
    pub fn headless(mut self) -> EngineBuilder {
        self.config.window = None;
        self
    }

    pub fn with_fixed_fps(mut self, fps: u32) -> EngineBuilder {
        self.config.fixed_fps = fps.max(1);
        self
    }

    pub fn with_clear_color(mut self, color: Color) -> EngineBuilder {
        self.config.clear_color = color;
        self
    }

    /// Swap the audio backend (default: `NullAudioBackend`).
    pub fn with_audio_backend(mut self, backend: Box<dyn AudioBackend>) -> EngineBuilder {
        self.audio = Some(backend);
        self
    }

    /// Enable the physics world with the given gravity vector
    /// (e.g. `Vec2::new(0.0, 980.0)` for +Y-down screens).
    pub fn with_physics(mut self, gravity: Vec2) -> EngineBuilder {
        self.physics_gravity = Some(gravity);
        self
    }

    /// Assemble the engine.
    pub fn build(self) -> Result<Engine, monster_platform::PlatformError> {
        let window: Box<dyn PlatformWindow> = match &self.config.window {
            Some(options) => Box::new(Window::new(options)?),
            None => Box::new(NullWindow::new()),
        };
        let (w, h) = window.size();
        let viewport = if w == 0 || h == 0 {
            (960.0, 540.0)
        } else {
            (w as f32, h as f32)
        };
        let renderer = Software2DRenderer::new(
            viewport.0 as u32,
            viewport.1 as u32,
            self.config.clear_color,
        );
        let camera = Camera2D::new(viewport.0, viewport.1);
        Ok(Engine {
            window,
            renderer,
            camera,
            assets: Assets::new(),
            audio: self
                .audio
                .unwrap_or_else(|| Box::new(monster_audio::NullAudioBackend::new())),
            physics: self.physics_gravity.map(PhysicsWorld2::new),
            events: EventBus::new(),
            world: World::new(),
            input: InputState::new(),
            clock: FixedClock::with_fps(self.config.fixed_fps),
            should_exit: false,
        })
    }
}

/// Summary of a finished run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineRunResult {
    /// Frames rendered.
    pub frames: u64,
    /// Fixed updates executed.
    pub updates: u64,
    /// Wall-clock seconds the loop ran.
    pub elapsed_secs: f32,
    /// Average frames per second over the run.
    pub avg_fps: f32,
}

/// The assembled engine. Owns every subsystem; game code reaches them only
/// through the contexts handed to [`GameApp`].
pub struct Engine {
    window: Box<dyn PlatformWindow>,
    renderer: Software2DRenderer,
    camera: Camera2D,
    assets: Assets,
    audio: Box<dyn AudioBackend>,
    physics: Option<PhysicsWorld2>,
    events: EventBus,
    world: World,
    input: InputState,
    clock: FixedClock,
    should_exit: bool,
}

impl Engine {
    /// Direct read-only world access for tools/tests.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Direct read-only asset access for tools/tests.
    pub fn assets(&self) -> &Assets {
        &self.assets
    }

    /// Read-only physics access (None when physics is disabled).
    pub fn physics(&self) -> Option<&PhysicsWorld2> {
        self.physics.as_ref()
    }

    /// Run the game until the window closes or `UpdateContext::exit` fires.
    pub fn run(&mut self, app: &mut dyn GameApp) -> EngineRunResult {
        let started = Instant::now();
        let mut frames: u64 = 0;
        let mut updates: u64 = 0;

        {
            let mut ctx = InitContext {
                world: &mut self.world,
                assets: &mut self.assets,
                audio: &mut *self.audio,
                camera: &mut self.camera,
                physics: self.physics.as_mut(),
            };
            app.init(&mut ctx);
        }

        while self.window.is_open() && !self.should_exit {
            // 1. Pump OS messages into the bus.
            self.window.poll_events(&mut self.events);
            if !self.window.is_open() {
                break;
            }

            // 2. Route events: window lifecycle handled here, input events
            //    forwarded into the InputState snapshot.
            let mut input_events: Vec<EngineEvent> = Vec::new();
            for event in self.events.drain() {
                match event {
                    EngineEvent::WindowResized { width, height } => {
                        self.camera.viewport = (width as f32, height as f32);
                        self.renderer.resize(width, height);
                    }
                    EngineEvent::WindowClosed => self.should_exit = true,
                    other => input_events.push(other),
                }
            }
            for event in input_events {
                self.events.push(event);
            }
            self.input.begin_frame(&mut self.events);

            // 3. Fixed-step simulation.
            let steps = self.clock.tick(Instant::now());
            for _ in 0..steps {
                let dt = self.clock.fixed_step_secs();
                // physics: authored bodies follow the scene, simulate, then
                // physics-owned bodies drive the scene back
                if self.physics.is_some() {
                    crate::physics_bridge::sync_static_to_physics(
                        &self.world,
                        self.physics.as_mut().unwrap(),
                    );
                    self.physics.as_mut().unwrap().step(dt);
                    crate::physics_bridge::sync_dynamic_to_scene(
                        self.physics.as_ref().unwrap(),
                        &mut self.world,
                    );
                }
                let mut ctx = UpdateContext {
                    world: &mut self.world,
                    assets: &mut self.assets,
                    input: &self.input,
                    audio: &mut *self.audio,
                    camera: &mut self.camera,
                    physics: self.physics.as_mut(),
                    dt,
                    should_exit: &mut self.should_exit,
                };
                app.update(&mut ctx);
                updates += 1;
                if self.should_exit {
                    break;
                }
            }
            if self.should_exit {
                break;
            }

            // 4. Render: engine draws world sprites (z-sorted), then the
            //    app draws on top, then present.
            let camera = self.camera.clone();
            self.renderer.begin(&camera);
            for draw in sprite_draw_order(&self.world) {
                let Some(texture) = self.assets.texture(draw.texture) else {
                    continue;
                };
                let dst = Rect::from_center(draw.position, draw.size * 0.5);
                self.renderer
                    .draw_sprite(texture, dst, draw.src, DrawOptions::default());
            }
            {
                let mut ctx = RenderContext {
                    world: &self.world,
                    renderer: &mut self.renderer,
                    alpha: self.clock.alpha(),
                };
                app.render(&mut ctx);
            }
            self.renderer.end(self.window.as_mut());
            frames += 1;
        }

        app.shutdown();

        let elapsed_secs = started.elapsed().as_secs_f32();
        let avg_fps = if elapsed_secs > 0.0 {
            frames as f32 / elapsed_secs
        } else {
            0.0
        };
        let result = EngineRunResult {
            frames,
            updates,
            elapsed_secs,
            avg_fps,
        };
        log_info!(
            "engine run finished: {} frames, {} updates, {:.1}s, avg {:.1} fps",
            result.frames,
            result.updates,
            result.elapsed_secs,
            result.avg_fps
        );
        result
    }
}
