//! Integration tests: the physics bridge (ECS ↔ PhysicsWorld2) driven by the
//! real engine loop — entities with `PhysicsLink` follow their bodies.

use monster_engine::prelude::*;

struct BridgeTestApp {
    ball_entity: Option<EntityId>,
    ball_body: Option<usize>,
    ground_entity: Option<EntityId>,
    updates: u32,
}

impl GameApp for BridgeTestApp {
    fn init(&mut self, ctx: &mut InitContext<'_>) {
        let Some(physics) = ctx.physics.as_mut() else {
            return;
        };
        // static ground
        let ground = physics.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(0.0, 200.0),
            ColliderShape2::Box {
                half_extents: Vec2::new(500.0, 10.0),
            },
        ));
        let ground_entity = ctx
            .world
            .spawn()
            .with(Transform::at(Vec2::new(0.0, 200.0)))
            .build();
        physics_attach(&mut ctx.world, ground_entity, ground);
        self.ground_entity = Some(ground_entity);

        // falling ball (restitution 0, lands and stays)
        let ball = physics.add_body(
            PhysicsBody2::new(
                BodyKind2::Rigid,
                Vec2::new(0.0, 50.0),
                ColliderShape2::Circle { radius: 10.0 },
            )
            .with_material(0.0, 0.5),
        );
        let ball_entity = ctx
            .world
            .spawn()
            .with(Transform::at(Vec2::new(0.0, 50.0)))
            .build();
        physics_attach(&mut ctx.world, ball_entity, ball);
        self.ball_entity = Some(ball_entity);
        self.ball_body = Some(ball);
    }

    fn update(&mut self, ctx: &mut UpdateContext<'_>) {
        if ctx.input.was_pressed(KeyCode::Escape) {
            ctx.exit();
        }
        // let the ball settle: exit after ~3 simulated seconds
        self.updates += 1;
        if self.updates >= 180 {
            ctx.exit();
        }
    }

    fn render(&mut self, _ctx: &mut RenderContext<'_>) {}
}

#[test]
fn physics_bridge_entity_follows_body() {
    let mut engine = EngineBuilder::new()
        .headless()
        .with_fixed_fps(60)
        .with_physics(Vec2::new(0.0, 980.0))
        .build()
        .unwrap();
    let mut app = BridgeTestApp {
        ball_entity: None,
        ball_body: None,
        ground_entity: None,
        updates: 0,
    };
    engine.run(&mut app);

    let ball_entity = app.ball_entity.expect("spawned ball entity");
    let ball_body = app.ball_body.expect("created ball body");

    // physics body rested on the ground (surface y=190, ball radius 10)
    let body_y = engine.physics().unwrap().bodies[ball_body]
        .transform
        .position
        .y;
    assert!(
        body_y > 170.0 && body_y < 186.0,
        "ball body rested at y={}",
        body_y
    );
    // scene Transform follows the body exactly
    let entity_y = engine
        .world()
        .get::<Transform>(ball_entity)
        .unwrap()
        .position
        .y;
    assert!(
        (entity_y - body_y).abs() < 1e-3,
        "entity y={} body y={}",
        entity_y,
        body_y
    );
    // ground stayed where the scene placed it (authored body follows scene)
    let ground_entity = app.ground_entity.unwrap();
    let ground_y = engine.physics().unwrap().bodies[0].transform.position.y;
    assert_eq!(ground_y, 200.0);
    let _ = ground_entity;
}

#[test]
fn engine_without_physics_leaves_context_none() {
    struct Probe {
        saw_physics: bool,
        ran: bool,
    }
    impl GameApp for Probe {
        fn init(&mut self, _ctx: &mut InitContext<'_>) {}
        fn update(&mut self, ctx: &mut UpdateContext<'_>) {
            self.saw_physics = ctx.physics.is_some();
            self.ran = true;
            ctx.exit();
        }
        fn render(&mut self, _ctx: &mut RenderContext<'_>) {}
    }
    let mut engine = EngineBuilder::new().headless().build().unwrap();
    let mut app = Probe {
        saw_physics: false,
        ran: false,
    };
    engine.run(&mut app);
    assert!(app.ran);
    assert!(!app.saw_physics, "physics is None when not enabled");
}
