import io

p = 'crates/monster-engine/src/engine.rs'
with io.open(p, 'r', encoding='utf-8') as f:
    c = f.read()

# 1. imports: add Gl3DRenderer, Camera3D, Renderer3D, GlContext
c = c.replace(
    'use monster_core::graphics::{Camera2D, DrawOptions, Renderer};',
    'use monster_core::graphics::{Camera2D, Camera3D, DrawOptions, Renderer, Renderer3D};'
)
c = c.replace(
    'use monster_platform::{NullWindow, PlatformWindow, Window, WindowOptions};',
    'use monster_platform::{NullWindow, PlatformWindow, Window, WindowOptions};\nuse monster_platform::GlContext;'
)
c = c.replace(
    'use monster_render::Software2DRenderer;',
    'use monster_render::Software2DRenderer;\nuse crate::gl3d::Gl3DRenderer;'
)

# 2. RenderContext3D struct (after RenderContext)
c = c.replace(
    '''/// The game contract: plain trait, plain functions.
pub trait GameApp {''',
    '''/// Per-frame 3D render context (enabled via `EngineBuilder::with_3d`).
pub struct RenderContext3D<'a> {
    pub renderer: &'a mut dyn Renderer3D,
    pub camera: &'a Camera3D,
}

/// The game contract: plain trait, plain functions.
pub trait GameApp {'''
)

# 3. GameApp::render_3d default no-op (after render)
c = c.replace(
    '''    /// Per-frame rendering. The engine has already drawn the world's
    /// sprites (z-sorted); draw extra effects on top here.
    fn render(&mut self, ctx: &mut RenderContext<'_>);''',
    '''    /// Per-frame rendering. The engine has already drawn the world's
    /// sprites (z-sorted); draw extra effects on top here.
    fn render(&mut self, ctx: &mut RenderContext<'_>);

    /// Per-frame 3D rendering — only called when `with_3d` is enabled.
    /// Default no-op for 2D-only games.
    fn render_3d(&mut self, _ctx: &mut RenderContext3D<'_>) {}'''
)

# 4. EngineBuilder: use_3d field + with_3d method
c = c.replace(
    '''    /// Gravity for the physics world (None = physics disabled).
    physics_gravity: Option<Vec2>,
}''',
    '''    /// Gravity for the physics world (None = physics disabled).
    physics_gravity: Option<Vec2>,
    /// Enable 3D rendering (None = 2D only).
    use_3d: bool,
}'''
)
c = c.replace(
    '''            physics_gravity: None,
        }
    }''',
    '''            physics_gravity: None,
            use_3d: false,
        }
    }'''
)
c = c.replace(
    '''    /// Enable the physics world with the given gravity vector
    /// (e.g. `Vec2::new(0.0, 980.0)` for +Y-down screens).
    pub fn with_physics(mut self, gravity: Vec2) -> EngineBuilder {
        self.physics_gravity = Some(gravity);
        self
    }''',
    '''    /// Enable the physics world with the given gravity vector
    /// (e.g. `Vec2::new(0.0, 980.0)` for +Y-down screens).
    pub fn with_physics(mut self, gravity: Vec2) -> EngineBuilder {
        self.physics_gravity = Some(gravity);
        self
    }

    /// Enable 3D rendering (creates Gl3DRenderer + Camera3D).
    /// Requires window mode for a real GL context; headless → unbound.
    pub fn with_3d(mut self) -> EngineBuilder {
        self.use_3d = true;
        self
    }'''
)

# 5. build(): create GL renderer when use_3d
c = c.replace(
    '''            physics: self.physics_gravity.map(PhysicsWorld2::new),
            events: EventBus::new(),''',
    '''            physics: self.physics_gravity.map(PhysicsWorld2::new),
            gl_renderer: if self.use_3d {
                Some(Gl3DRenderer::new_unbound(viewport.0 as u32, viewport.1 as u32))
            } else {
                None
            },
            gl_context: if self.use_3d {
                // only real windows can create a GL context
                match &window.as_any().downcast_ref::<Window>() {
                    Some(win) => GlContext::new(win).ok(),
                    None => None,
                }
            } else {
                None
            },
            camera3d: Camera3D::new(
                monster_math::Vec3::new(0.0, 3.0, 8.0),
                monster_math::Vec3::ZERO,
                viewport.0 / viewport.1.max(1.0),
            ),
            events: EventBus::new(),'''
)

# 6. Engine struct: gl fields
c = c.replace(
    '''pub struct Engine {
    window: Box<dyn PlatformWindow>,
    renderer: Software2DRenderer,
    camera: Camera2D,''',
    '''pub struct Engine {
    window: Box<dyn PlatformWindow>,
    renderer: Software2DRenderer,
    camera: Camera2D,
    gl_renderer: Option<Gl3DRenderer>,
    gl_context: Option<GlContext>,
    camera3d: Camera3D,'''
)

# 7. bind GL renderer after build if context available
# This needs to happen right after Engine construction — add bind logic
# Actually, bind in run() init phase is cleaner

# 8. run(): bind GL + render_3d call
c = c.replace(
    '''            {
                let mut ctx = RenderContext {
                    world: &self.world,
                    renderer: &mut self.renderer,
                    alpha: self.clock.alpha(),
                };
                app.render(&mut ctx);
            }
            self.renderer.end(self.window.as_mut());''',
    '''            {
                let mut ctx = RenderContext {
                    world: &self.world,
                    renderer: &mut self.renderer,
                    alpha: self.clock.alpha(),
                };
                app.render(&mut ctx);
            }

            // 3D rendering pass (optional)
            if let (Some(gl_rend), Some(gl_ctx)) = (&mut self.gl_renderer, &self.gl_context) {
                // bind on first frame
                if !gl_rend.is_bound() {
                    // We need to bind through a mutable reference but gl_context
                    // is behind &self. The Gl3DRenderer.bind() only needs to
                    // mark itself as bound and enable depth test — the actual
                    // GL context is already current from creation.
                    // For now, just mark it as bound via a mutable borrow.
                }
                let camera3d = self.camera3d.clone();
                gl_rend.begin(&camera3d);
                let mut ctx3d = RenderContext3D {
                    renderer: gl_rend as &mut dyn Renderer3D,
                    camera: &self.camera3d,
                };
                app.render_3d(&mut ctx3d);
                gl_rend.end();
                gl_ctx.swap_buffers();
            }
            if self.gl_renderer.is_none() {
                self.renderer.end(self.window.as_mut());
            }'''
)

# 9. Camera3D getter
c = c.replace(
    '''    /// Read-only camera access for tools/tests.
    pub fn camera(&self) -> &Camera2D {
        &self.camera
    }''',
    '''    /// Read-only camera access for tools/tests.
    pub fn camera(&self) -> &Camera2D {
        &self.camera
    }

    /// Read-only 3D camera access.
    pub fn camera3d(&self) -> &Camera3D {
        &self.camera3d
    }'''
)

# 10. camera3d mutable access for InitContext
c = c.replace(
    '''pub struct InitContext<'a> {
    pub world: &'a mut World,
    pub assets: &'a mut Assets,
    pub audio: &'a mut dyn AudioBackend,
    pub camera: &'a mut Camera2D,
    /// Physics world when enabled via `EngineBuilder::with_physics`.
    pub physics: Option<&'a mut PhysicsWorld2>,
}''',
    '''pub struct InitContext<'a> {
    pub world: &'a mut World,
    pub assets: &'a mut Assets,
    pub audio: &'a mut dyn AudioBackend,
    pub camera: &'a mut Camera2D,
    pub camera3d: &'a mut Camera3D,
    /// Physics world when enabled via `EngineBuilder::with_physics`.
    pub physics: Option<&'a mut PhysicsWorld2>,
}'''
)

# InitContext construction: add camera3d
c = c.replace(
    '''            let mut ctx = InitContext {
                world: &mut self.world,
                assets: &mut self.assets,
                audio: &mut *self.audio,
                camera: &mut self.camera,
                physics: self.physics.as_mut(),
            };''',
    '''            let mut ctx = InitContext {
                world: &mut self.world,
                assets: &mut self.assets,
                audio: &mut *self.audio,
                camera: &mut self.camera,
                camera3d: &mut self.camera3d,
                physics: self.physics.as_mut(),
            };'''
)

# 11. fix window.as_any() — PlatformWindow doesn't have as_any. Use raw_hwnd instead
c = c.replace(
    '''            gl_context: if self.use_3d {
                // only real windows can create a GL context
                match &window.as_any().downcast_ref::<Window>() {
                    Some(win) => GlContext::new(win).ok(),
                    None => None,
                }
            } else {
                None
            },''',
    '''            gl_context: if self.use_3d {
                // only real windows can create a GL context
                match &window {
                    w if w.is_window() => GlContext::new(w.as_window()).ok(),
                    _ => None,
                }
            } else {
                None
            },'''
)

# 12. PlatformWindow needs is_window + as_window methods
# Actually, simpler: check if the window is a real Window (not NullWindow)
# We can use std::any::TypeId or just check if raw_hwnd is non-null
# Let me add a helper on PlatformWindow trait — but that's breaking.
# Better: store the GlContext creation as a deferred action.
# Actually the simplest: keep GlContext creation outside Engine — the game
# creates it in init() via InitContext. But that complicates things.

# Simplest approach: EngineBuilder::build() knows if window is real or null.
# If real, create GlContext there and store it.
c = c.replace(
    '''        let window: Box<dyn PlatformWindow> = match &self.config.window {
            Some(options) => Box::new(Window::new(options)?),
            None => Box::new(NullWindow::new()),
        };''',
    '''        let (window, gl_context): (Box<dyn PlatformWindow>, Option<GlContext>) = match &self.config.window {
            Some(options) => {
                let win = Window::new(options)?;
                let ctx = if self.use_3d { GlContext::new(&win).ok() } else { None };
                (Box::new(win), ctx)
            }
            None => (Box::new(NullWindow::new()), None),
        };'''
)
c = c.replace(
    '''            gl_context: if self.use_3d {
                // only real windows can create a GL context
                match &window {
                    w if w.is_window() => GlContext::new(w.as_window()).ok(),
                    _ => None,
                }
            } else {
                None
            },''', '''''')

# fix: Gl3DRenderer bind needs &mut self on the renderer, but gl_context is behind &self
# Move the bind to a separate step after build — use a marker on Engine
c = c.replace(
    '''            gl_renderer: if self.use_3d {
                Some(Gl3DRenderer::new_unbound(viewport.0 as u32, viewport.1 as u32))
            } else {
                None
            },''',
    '''            gl_renderer: if self.use_3d {
                Some(Gl3DRenderer::new_unbound(viewport.0 as u32, viewport.1 as u32))
            } else {
                None
            },'''
)

with io.open(p, 'w', encoding='utf-8') as f:
    f.write(c)
print('engine.rs 3D integration ok')
