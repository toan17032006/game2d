# Traceability Matrix — monster-engine v0.1.0

- Owner: Product and Engineering
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Truy vết hai chiều: requirement → module hiện thực → test kiểm chứng → tài liệu. Mọi FR đều có test thật trong workspace (105 tests, toàn bộ PASS — xem `/.agent/tasks/TASK-20261113-008/logs/test.log`).

## Ma trận FR → Module → Test

| FR | Module (crate) | Test kiểm chứng | Test result |
| --- | --- | --- | --- |
| FR-001 | monster-math::vec | `vec::tests::vec2_ops`, `vec2_normalize`, `vec2_lerp_and_angle`, `vec3_cross_and_extend` | PASS |
| FR-002 | monster-math::mat4 | `mat4::tests::identity_and_associativity`, `orthographic_maps_corners`, `rotate_z_90_moves_x_to_y`, `view_2d_centers_camera_position`, `perspective_points_into_screen` | PASS |
| FR-003 | monster-math::quat | `quat::tests::identity_rotates_nothing`, `z_axis_90_maps_x_to_y`, `composition_order`, `normalize_unnormalized` | PASS |
| FR-004 | monster-core::events | `events::tests::fifo_order_and_drain` | PASS |
| FR-005 | monster-core::time | `time::tests::accumulates_into_steps`, `max_frame_delta_clamps_spiral`, `first_tick_is_zero_steps` | PASS |
| FR-006 | monster-core::input | `input::tests::press_hold_release_cycle`, `mouse_buttons_and_motion`, `dense_indices_unique` | PASS |
| FR-007 | monster-core::log | `log::tests::level_gating_and_parsing` | PASS |
| FR-008 | monster-core::handle | `handle::tests::handles_are_typed_and_copy` | PASS |
| FR-009 | monster-platform::window | `window::tests::wide_encoding_null_terminated` + compile-link user32 (window thật: chạy thủ công) | PASS |
| FR-010 | monster-platform::keymap | `keymap::tests::letters_and_digits_map`, `control_keys_map`, `unknown_falls_back` | PASS |
| FR-011 | monster-platform::window | `window::tests::null_window_counts_presents`, `null_window_bus_stays_empty` | PASS |
| FR-012 | monster-platform::window | compile-link gdi32 (present thật: verify trực quan window mode) | PASS |
| FR-013 | monster-render::framebuffer | `framebuffer::tests::blend_pixel_overwrite_and_skip`, `blend_pixel_half`, `fill_rect_clips`, `clear_and_read_pixels` | PASS |
| FR-014 | monster-render::framebuffer | `blit_opaque_sprite_covers_destination`, `blit_src_subrect_selects_region`, `blit_flip_mirrors`, `blit_tint_multiplies`, `blit_alpha_blends_over_background` | PASS |
| FR-015 | monster-core::graphics + monster-render::software | `graphics::tests::camera_roundtrip_and_zoom`, `software::tests::camera_transform_maps_world_to_screen`, `zoom_shrinks_draw_area` | PASS |
| FR-016 | monster-render::software | `software::tests::begin_clears_and_counts`, `end_presents_frame_to_target`, `sprite_via_camera_and_clipping_no_panic` | PASS |
| FR-017 | monster-assets::registry | `registry::tests::same_key_returns_stable_handle`, `insert_and_retrieve_procedural` | PASS |
| FR-018 | monster-assets::png | `png::tests::decodes_rgba_solid`, `decodes_palette_with_trns`, `decodes_gray_with_trns_color_key`, `defilter_sub_up_average_paeth`, `rejects_bad_signature`, `rejects_corrupt_crc`, `rejects_interlaced`, `real_asset_smoke_test` (PNG thật 192×256) | PASS |
| FR-019 | monster-assets::bmp | `bmp::tests::decodes_bottom_up_24bpp`, `decodes_top_down_negative_height`, `rejects_non_bmp_and_unsupported` | PASS |
| FR-020 | monster-assets::inflate | `inflate::tests::stored_block_roundtrip`, `two_stored_blocks`, `fixed_huffman_literals`, `fixed_huffman_back_reference`, `rejects_bad_zlib_and_adler` + `crc::tests::crc32_known_vector`, `adler32_known_vector` | PASS |
| FR-021 | monster-scene::world | `world::tests::spawn_get_and_alive`, `despawn_generations_guard_stale_ids`, `insert_replace_remove_and_mutate` | PASS |
| FR-022 | monster-scene::world | `iter_mut_touches_only_live`, `for_each2_joins_intersecting_components`, `dead_entities_never_join` | PASS |
| FR-023 | monster-scene::systems | `systems::tests::movement_integrates_positions`, `draw_order_sorts_by_z` | PASS |
| FR-024 | monster-physics | `physics::tests::intersects_variants`, `separation_picks_smallest_axis`, `separation_none_when_apart`, `collision_normal_is_unit_axis`, `bounce_reflects_and_restitution` | PASS |
| FR-025 | monster-physics | `physics::tests::broadphase_reports_pairs`, `integrate_and_gravity` | PASS |
| FR-026 | monster-audio::wav | `wav::tests::decodes_mono_pcm16`, `stereo_mixed_down_to_mono`, `rejects_non_riff_and_truncated`, `skips_extra_chunks_and_alignment` | PASS |
| FR-027 | monster-audio::backend | `backend::tests::play_stop_cycle`, `volumes_are_clamped` | PASS |
| FR-028 | monster-engine::engine | `engine_loop::tests::audio_backend_swaps_without_core_changes` | PASS |
| FR-029 | monster-engine::engine | `engine_loop::tests::headless_loop_runs_app_callbacks`, `exit_flag_stops_the_loop` | PASS |
| FR-030 | monster-engine::engine | `headless_loop_runs_app_callbacks` (assert draw_count ≥ 1 trong render) | PASS |
| FR-031 | monster-engine example | `logs/sandbox-headless.log`: 122 frames, exit 0 (TASK-20261113-008) | PASS |

## Ma trận NFR → Bằng chứng

| NFR | Bằng chứng |
| --- | --- |
| NFR-01 zero-dep | 9 `Cargo.toml` chỉ có path deps; commit `e1aff19` |
| NFR-02 headless | `cargo test --workspace` 105 PASS / 0 FAIL (`logs/test.log`) |
| NFR-03 hiệu năng | `logs/sandbox-headless.log`: avg ~660 fps debug |
| NFR-04 deterministic | test FixedClock accumulator chính xác đến ms |
| NFR-05 safety | biên `unsafe` trong `monster-platform/src/ffi.rs` + callers |
| NFR-06 extensibility | `audio_backend_swaps_without_core_changes` PASS |
| NFR-07 maintainability | biểu đồ crate trong `container-and-component.md` |
| NFR-09 gates | preflight + final gates PASS (TASK-20261113-008) |
| NFR-10 docs | rustdoc mọi public item; bộ docs `/docs/` tiếng Việt |

## Ma trận AC (task) → FR

| AC của TASK-20261113-008 | FR liên quan |
| --- | --- |
| AC-002 (build zero-dep) | NFR-01 |
| AC-003 (trait-based) | FR-027, FR-028, NFR-06 |
| AC-004 (tests subsystems) | FR-001..FR-027 |
| AC-005 (sandbox headless) | FR-031 |
| AC-006 (ADR draft) | ADR-001 |

## Ma trận FR Physics v2 (TASK-20261113-010)

| FR | Module (crate) | Test kiểm chứng | Test result |
| --- | --- | --- | --- |
| FR-032 | monster-physics::world | `world::tests::rigid_falls_and_lands_on_static_ground`, `bouncing_ball_with_restitution` | PASS |
| FR-033 | monster-physics::shape + collision | `collision::tests::{circle_circle_contact_and_separation, circle_box_contact_normal_flips_with_order, box_box_sat, circle_plane_from_free_side, circle_convex_polygon_and_cone, concave_polygon_via_parts, heightfield_smooth_ground, terrain_blocky_columns, stadium_stadium_and_plane, compound_vs_circle_deepest_contact, wheel_behaves_like_circle}`, `shape::tests::convex_polygon_validates`, `contains_point_shapes`, `plane_solid_side`, `heightfield_and_terrain_containment`, `compound_bound_and_contains` | PASS |
| FR-034 | monster-physics::world | `world::tests::static_body_infinite_mass_no_response` (body), `rigid_falls_and_lands_on_static_ground` | PASS |
| FR-035 | monster-physics::world | `world::tests::kinematic_pushes_dynamic`, `animatable_pushes_and_ignores_gravity` | PASS |
| FR-036 | monster-physics::world | `world::tests::area_trigger_events_fire_and_exit`, `area_is_trigger_by_default` | PASS |
| FR-037 | monster-physics::world | `raycast::tests::character_move_and_slide_walks_on_floor`, `character_stops_at_wall` | PASS |
| FR-038 | monster-physics::joint | `joint::tests::bone_body_uses_stadium` | PASS |
| FR-039 | monster-physics::joint | `joint::tests::articulated_chain_hangs_together` | PASS |
| FR-040 | monster-physics::particles | `particles::tests::soft_blob_falls_stays_connected` | PASS |
| FR-041 | monster-physics::particles | `particles::tests::cloth_pinned_top_hangs` | PASS |
| FR-042 | monster-physics::vehicle | `vehicle::tests::{vehicle_settles_on_suspension, vehicle_drives_forward, suspension_compression_tracks_ground}` | PASS |
| FR-043 | monster-physics::particles | `particles::tests::particle_cloud_lands_on_ground` | PASS |
| FR-044 | monster-physics::particles | `particles::tests::fluid_stays_on_ground_and_spreads` | PASS |
| FR-045 | monster-physics::particles | `particles::tests::deformable_squashes_then_recovers_shape` | PASS |
| FR-046 | monster-physics::joint | `joint::tests::{distance_joint_holds_length_under_gravity, pin_joint_keeps_anchors_together, weld_locks_relative_angle, spring_pulls_towards_rest_length}` | PASS |
| FR-047 | monster-physics::raycast | `raycast::tests::{ray_hits_circle, ray_misses_when_too_short, ray_hits_box_face, ray_hits_heightfield, ray_nearest_hit_wins}` | PASS |
| FR-048 | monster-physics::shape + collision | `collision::tests::custom_shape_trait_narrowphase` | PASS |

## Ma trận FR Physics Bridge (TASK-20261114-001)

| FR | Module (crate) | Test kiểm chứng | Test result |
| --- | --- | --- | --- |
| FR-049 | monster-engine::physics_bridge | `physics_bridge::tests::{dynamic_body_drives_scene_transform, static_scene_transform_drives_body, unlinked_entities_are_untouched}`, `tests/physics_bridge.rs::{physics_bridge_entity_follows_body, engine_without_physics_leaves_context_none}`, sandbox `--physics` (logs) | PASS |

## Ma trận FR Camera follow + M3 slice (TASK-20261114-002)

| FR | Module (crate) | Test kiểm chứng | Test result |
| --- | --- | --- | --- |
| FR-050 | monster-scene::systems | `systems::tests::camera_follow_approaches_target_smoothly`, `camera_follow_offset_and_missing_target` | PASS |
| FR-051 | monster-math + monster-core | `mat4::tests::look_at_maps_camera_axis_to_view_axis`, `graphics.rs::camera3d_tests::{camera3d_view_projection_combines, camera3d_defaults_sane}`, `gl3d::tests::renderer3d_contract_with_mock` | PASS |
| FR-052 | monster-platform::gl + monster-engine::gl3d | `gl::tests::{pixel_format_descriptor_layout, gl_constants_match_registry}` + `gl3d::tests::{unbound_renderer_drives_full_frame_protocol, camera3d_drives_begin_state, cube_rotation_is_accepted_parameter}` + build-link opengl32 (GL runtime: demo thủ công) | PASS |

## Ma trận FR Animation + Mesh + Game slice (TASK-20261114-003)

| FR | Module (crate) | Test kiểm chứng | Test result |
| --- | --- | --- | --- |
| FR-053 | monster-scene::animation | `animation::tests::{from_grid_slices_row_major, source_rect_clamps, looping_animation_wraps, non_looping_clamps_and_stops, pause_freezes_the_frame}` | PASS |
| FR-054 | monster-core::mesh + gl3d | `mesh::tests::{cube_mesh_shape, obj_parses_vertices_and_faces, obj_negative_and_relative_indices, obj_errors_are_descriptive, cube_first_triangle_front_face}`, `gl3d::tests::draw_mesh_counts_in_unbound_mode` | PASS |
| FR-055 | monster-engine::examples | `logs/sandbox-game.log`: exit 0 — player_x 497.7 (moved), anim_frames_seen > 2, score = 3, camera_followed; regression `logs/sandbox-physics.log` + `logs/sandbox-classic.log` | PASS |

## Ma trận FR M4 slice 2 (TASK-20261114-004)

| FR | Module (crate) | Test kiểm chứng | Test result |
| --- | --- | --- | --- |
| FR-056 | monster-render::text | `text::tests::{glyphs_are_three_wide_five_tall, text_metrics, draw_text_writes_pixels, draw_text_respects_scale_and_advances}` | PASS |
| FR-057 | monster-scene::tilemap | `tilemap::tests::{spawns_solids_skips_empty, all_empty_spawns_nothing}` + sandbox --game (11 walls) | PASS |
| FR-058 | monster-scene::particles2d | `particles2d::tests::{burst_spawns_and_particles_move_and_die, continuous_rate_spawns_over_time, color_lerps_over_lifetime}` + sandbox coin sparks | PASS |
| FR-059 | monster-platform::winmm_audio | `winmm_audio::tests::{construction_reports_device_presence, play_stop_state_machine_with_or_without_device, waveformatex_layout}` + sandbox --game audio blip (winmm/Null fallback) | PASS |

## Ma trận FR M4 slice 3 + M3 slice 3 (TASK-20261114-005)

| FR | Module (crate) | Test kiểm chứng | Test result |
| --- | --- | --- | --- |
| FR-060 | monster-scene::systems + engine render | `systems::tests::sprite_culling_by_camera_viewport` + engine render pass dùng sprite_in_view (margin 64) | PASS |
| FR-061 | monster-scene::tilemap | `tilemap::tests::multi_layer_spawn_coexists` | PASS |
| FR-062 | monster-platform::gl + monster-engine::gl3d | `gl.rs::modern_tests::{extension_loader_without_context_falls_back, shader_sources_are_glsl_120_pass_through, compile_program_requires_modern_gl}` + `gl3d::tests` (unbound recorder shader_program=0 fallback) | PASS |
| FR-062a | monster-audio | `wav::tests::load_wav_file_roundtrip` | PASS |

## Ma trận FR Kiến trúc tách game (TASK-20261114-007)

| FR | Module (crate) | Test kiểm chứng | Test result |
| --- | --- | --- | --- |
| FR-066 | /game/ + /monster-engine/ | `game/` cargo build exit 0 + `--headless 120 --game` exit 0 + engine tests 205 PASS không regression | PASS |

## Quy tắc bảo trì

- Mỗi FR mới MUST có ít nhất một test thật trước khi đánh dấu ✅.
- Đổi tên/di chuyển test MUST cập nhật ma trận này trong cùng commit.
- FR bị loại bỏ MUST được ghi lại (không xóa hàng — chuyển sang `Deprecated` kèm lý do).

## References

- `/docs/02-requirements/functional-requirements.md`
- `/docs/02-requirements/non-functional-requirements.md`
- `/docs/05-quality/quality-strategy.md`
- `/monster-engine/crates/` (nguồn test thật)
