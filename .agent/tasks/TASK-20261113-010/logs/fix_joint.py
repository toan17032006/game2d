import io

p = 'crates/monster-physics/src/joint.rs'
with io.open(p, 'r', encoding='utf-8') as f:
    c = f.read()

# 1. rewrite solve_distance as impulse + Baumgarte
c = c.replace('''fn solve_distance(
    world: &mut PhysicsWorld2,
    a: usize,
    b: usize,
    anchor_a: Vec2,
    anchor_b: Vec2,
    length: f32,
) {
    let Some(wa) = world_anchor(world, a, anchor_a) else { return };
    let Some(wb) = world_anchor(world, b, anchor_b) else { return };
    let (inv_a, inv_b) = {
        let (ba, bb) = (&world.bodies[a], &world.bodies[b]);
        (ba.inv_mass(), bb.inv_mass())
    };
    if inv_a + inv_b <= 0.0 {
        return;
    }
    let delta = wb - wa;
    let dist = delta.length();
    if dist <= 1.0e-9 {
        return;
    }
    let diff = dist - length;
    let axis = delta / dist;
    // positional correction split by inverse mass
    let total = inv_a + inv_b;
    let ba = &mut world.bodies[a];
    ba.transform.position -= axis * (diff * inv_a / total);
    let bb = &mut world.bodies[b];
    bb.transform.position += axis * (diff * inv_b / total);
    // velocity: remove relative motion along the axis
    let rv = world.bodies[b].velocity - world.bodies[a].velocity;
    let vel_n = rv.dot(axis) - diff / (1.0 / 60.0); // damp approach speed
    let _ = vel_n;
    let rv_along = rv.dot(axis);
    let ba = &mut world.bodies[a];
    ba.velocity += axis * (rv_along * inv_a / total);
    let bb = &mut world.bodies[b];
    bb.velocity -= axis * (rv_along * inv_b / total);
}''', '''/// Impulse-based distance constraint with Baumgarte bias (stable).
fn solve_distance(
    world: &mut PhysicsWorld2,
    a: usize,
    b: usize,
    anchor_a: Vec2,
    anchor_b: Vec2,
    length: f32,
) {
    let Some(wa) = world_anchor(world, a, anchor_a) else { return };
    let Some(wb) = world_anchor(world, b, anchor_b) else { return };
    let (inv_a, inv_b) = {
        let (ba, bb) = (&world.bodies[a], &world.bodies[b]);
        (ba.inv_mass(), bb.inv_mass())
    };
    let total = inv_a + inv_b;
    if total <= 0.0 {
        return;
    }
    let delta = wb - wa;
    let dist = delta.length();
    if dist <= 1.0e-9 {
        return;
    }
    let axis = delta / dist;
    let diff = dist - length; // C: > 0 stretched, < 0 compressed
    // Cdot = axis·(vB - vA); impulse = -(Cdot + beta*C/dt) / (invA + invB)
    let rv = world.bodies[b].velocity - world.bodies[a].velocity;
    let vel_n = rv.dot(axis);
    let beta = 0.3;
    let bias = (diff / dt * beta).clamp(-2000.0, 2000.0);
    let lambda = (-(vel_n + bias) / total).clamp(-5000.0, 5000.0);
    let ba = &mut world.bodies[a];
    ba.velocity -= axis * (lambda * inv_a);
    let bb = &mut world.bodies[b];
    bb.velocity += axis * (lambda * inv_b);
    // small positional stabilization to kill residual drift
    let pos_fix = diff * 0.2;
    let ba = &mut world.bodies[a];
    ba.transform.position -= axis * (pos_fix * inv_a / total);
    let bb = &mut world.bodies[b];
    bb.transform.position += axis * (pos_fix * inv_b / total);
}

/// 2D point constraint (Pin): full-vector impulse form.
fn solve_pin_vector(world: &mut PhysicsWorld2, a: usize, b: usize, anchor_a: Vec2, anchor_b: Vec2) {
    let Some(wa) = world_anchor(world, a, anchor_a) else { return };
    let Some(wb) = world_anchor(world, b, anchor_b) else { return };
    let (inv_a, inv_b) = {
        let (ba, bb) = (&world.bodies[a], &world.bodies[b]);
        (ba.inv_mass(), bb.inv_mass())
    };
    let total = inv_a + inv_b;
    if total <= 0.0 {
        return;
    }
    let cv = wb - wa; // constraint violation vector
    let rv = world.bodies[b].velocity - world.bodies[a].velocity;
    let beta = 0.3;
    let bias = cv * (beta / dt.max(1.0e-4));
    let lambda = -(rv + bias) * (1.0 / total);
    let mag = lambda.length();
    let lambda = if mag > 5000.0 { lambda * (5000.0 / mag) } else { lambda };
    let ba = &mut world.bodies[a];
    ba.velocity -= lambda * inv_a;
    let bb = &mut world.bodies[b];
    bb.velocity += lambda * inv_b;
    // positional stabilization (percent)
    let fix = cv * 0.2;
    let ba = &mut world.bodies[a];
    ba.transform.position -= fix * (inv_a / total);
    let bb = &mut world.bodies[b];
    bb.transform.position += fix * (inv_b / total);
}''')

# 2. Pin/Weld arms use the vector solver
c = c.replace('''                Joint2::Pin { a, b, anchor_a, anchor_b } => {
                    solve_distance(world, *a, *b, *anchor_a, *anchor_b, 0.0);
                    match_anchor_velocities(world, *a, *b, *anchor_a, *anchor_b);
                }
                Joint2::Weld { a, b, anchor_a, anchor_b } => {
                    solve_distance(world, *a, *b, *anchor_a, *anchor_b, 0.0);
                    match_anchor_velocities(world, *a, *b, *anchor_a, *anchor_b);
                    lock_angles(world, *a, *b);
                }''', '''                Joint2::Pin { a, b, anchor_a, anchor_b } => {
                    solve_pin_vector(world, *a, *b, *anchor_a, *anchor_b);
                }
                Joint2::Weld { a, b, anchor_a, anchor_b } => {
                    solve_pin_vector(world, *a, *b, *anchor_a, *anchor_b);
                    lock_angles(world, *a, *b);
                }''')

# 3. remove unused match_anchor_velocities
c = c.replace('''fn match_anchor_velocities(
    world: &mut PhysicsWorld2,
    a: usize,
    b: usize,
    anchor_a: Vec2,
    anchor_b: Vec2,
) {
    let (Some(wa), Some(wb)) = (world_anchor(world, a, anchor_a), world_anchor(world, b, anchor_b))
    else {
        return;
    };
    let (inv_a, inv_b) = {
        let (ba, bb) = (&world.bodies[a], &world.bodies[b]);
        (ba.inv_mass(), bb.inv_mass())
    };
    let total = inv_a + inv_b;
    if total <= 0.0 {
        return;
    }
    let rv = world.bodies[b].velocity - world.bodies[a].velocity;
    // move velocities towards each other weighted by inverse mass
    let ba = &mut world.bodies[a];
    ba.velocity += rv * (inv_a / total) * 0.5;
    let bb = &mut world.bodies[b];
    bb.velocity -= rv * (inv_b / total) * 0.5;
    let _ = (wa, wb);
}

''', '')

# 4. lock_angles: gentler positional correction
c = c.replace('''    let angle_diff = world.bodies[a].transform.rotation - world.bodies[b].transform.rotation;
    let ba = &mut world.bodies[a];
    ba.transform.rotation -= angle_diff * 0.5 * (inv_a / total);
    let bb = &mut world.bodies[b];
    bb.transform.rotation += angle_diff * 0.5 * (inv_b / total);''', '''    let angle_diff = world.bodies[a].transform.rotation - world.bodies[b].transform.rotation;
    let ba = &mut world.bodies[a];
    ba.transform.rotation -= angle_diff * 0.2 * (inv_a / total);
    let bb = &mut world.bodies[b];
    bb.transform.rotation += angle_diff * 0.2 * (inv_b / total);''')

with io.open(p, 'w', encoding='utf-8') as f:
    f.write(c)
print('joint.rs rewritten:', 'solve_pin_vector' in c, 'match_anchor_velocities' not in c)
