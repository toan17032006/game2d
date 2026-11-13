# Specification

## Inputs

- `origin/main` @ 764b60d: physics taxonomy (monster-physics: body/shape/collision/joint/particles/raycast/vehicle/world) + `physics_bridge.rs` + engine tích hợp (`with_physics`, `ctx.physics`, sync tự động mỗi fixed step).
- `monster-core` `InputState::is_down(KeyCode)`; KeyCode có W/A/S/D và Left/Right/Up/Down.
- `PhysicsBody2`: `new(kind, position, ColliderShape2)`, `.with_velocity`, `.with_gravity_scale`, `.with_material`, `.with_tag`; `PhysicsWorld2::new(gravity)`, `.add_body`, `.step`, `.bodies` public mutable.
- Bridge: physics-owned kinds (Rigid/Character/Bone/Vehicle/Articulated) → physics drives `Transform`; authored kinds (Static/Kinematic/Animatable/Area) → scene drives body. Engine tự chạy `sync_static_to_physics → step → sync_dynamic_to_scene` TRƯỚC `app.update` mỗi fixed step.

## Confirmed facts

- Nhánh `game` = cda2ed8 = fast-forward-able đến 764b60d (không diverge).
- Commit 764b60d chứa `.agent/tasks/TASK-20261114-001/` → xung đột untracked với task local cùng tên.
- Engine step physics trước app.update → velocity set trong update có độ trễ 1 step, chấp nhận.
- `InputState` không có setter public cho held keys → để test headless được pipeline input, game cần mode tự inject input (`PlayerInput` struct tách khỏi `InputState`).

## Inferences

- Camera follow không cần thiết: world trùng viewport 960×540, biên world = mép màn hình.

## Unknowns

- Không có.

## Constraints

- Dependency: chỉ `monster-engine` (path) + std.
- Gravity (0,0) — world kiểu top-down; Character body dừng bằng cách set velocity = 0 khi không có input.
- Collider player nhỏ hơn sprite (bàn chân), không phải full 192px.

## Decisions

- D1: Player = 1 hàng `player` trong `scene.csv` (kind mới), dùng def từ `units.csv` (blue:warrior, speed 120).
- D2: Di chuyển qua physics Character body (dùng thành phần mới nhất của main) thay vì ECS Velocity — NPCs giữ nguyên ECS Velocity.
- D3: `PlayerInput` struct (left/right/up/down) dựng từ `InputState` mỗi update; `--autodrive` thay input bằng kịch bản thời gian → testable headless.
- D4: Biên world = 4 static bodies (không link entity); buildings (house1, tower) = static bodies + `PhysicsLink` (authored: scene → physics).
- D5: Đổi tên task local 001 → 002 và cập nhật chuỗi Task ID trong artifacts của nó.

## Output contract

- `game/src/player.rs` mới: `Player`, `PlayerInput`, `player_system`, `player_input_from_state`, autodrive pattern; `game.rs` tích hợp (with_physics, spawn bodies, update); `defs.rs` thêm `SceneKind::Player`; `scene.csv` hàng player; tests.
- `--autodrive` flag mới cho bin; log xác minh vị trí.

## Test Command

`powershell -NoProfile -ExecutionPolicy Bypass -File .agent/tasks/TASK-20261114-003/scripts/run-headless-autodrive.ps1`

## References

- `/AGENTS.md`
- `/monster-engine/crates/monster-engine/src/physics_bridge.rs`
- `/monster-engine/crates/monster-physics/src/body.rs`
- `/monster-engine/crates/monster-physics/src/world.rs`
