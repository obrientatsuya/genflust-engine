# Evidências (fase 1)

Onde está a prova de cada item da fase 1. Rode `cargo test --workspace`; fixtures golden regeneram com `GEN_FIXTURE=1`.

| Item | Evidência |
|---|---|
| ENG-01-03 workspace/CI | `.github/workflows/ci.yml` (fmt/clippy/test/arch-check/token-budget, ubuntu+windows) — verde no main |
| ENG-01-04-S01 tipos/erros | `crates/api/src/error.rs` + uso em todos os crates; `cargo build -p api` isolado |
| ENG-01-05-S01/S04 config/diagnóstico | `crates/cli/src/config.rs` (E020–E022 com `file`+`fix`); `crates/app/tests/cli.rs::missing_project_is_structured_error` |
| ENG-01-05-S02 caminhos | `crates/cli/src/project.rs::project_path` (rejeita `..`/absoluto, E023); `evidence.rs::path_escape_is_rejected` |
| ENG-01-05-S03 capacidades | `crates/cli/src/config.rs` (E024/E025 com `fix`); `evidence.rs::unavailable_capability_is_structured_error`, `unknown_capability_is_rejected` |
| ENG-01-06 identidade geracional | `crates/ecs/tests/world.rs` (barrier, obsoleto, slot reutilizado, tags/nomes) |
| ENG-01-07-S01 SoA | `crates/ecs/src/soa.rs` (contiguidade, ordem estável, remoção O(1)); `world.rs::component_counts` |
| ENG-01-07-S04 serialização | `crates/ecs/src/snapshot.rs` (ordem canônica, transitório fora, roundtrip TOML) |
| ENG-01-07-S02/S03 componentes | `world.rs` (Transform/Sprite/Velocity; `sprite_requires_transform`) |
| ENG-01-08 barreiras | `world.rs::spawn_activates_at_barrier`, `kill_at_barrier...`, `kill_pending_cancels_spawn_at_barrier`, `invalid_batch_leaves_no_partial_state` |
| ENG-01-09 cena | `crates/core/src/scene.rs` (validate prévio, loader com local, mounts); `evidence.rs` (hello com cena) |
| ENG-01-10-S02 RNG | `crates/core/src/rng.rs` (mesma seed; `state_restore_resumes_sequence`) |
| ENG-01-10-S01/S04 trace/relógio | `evidence.rs::tick_trace_shows_fixed_phase_order` (`barrier,update,draw,raster,hash`); `crates/core/src/clock.rs` |
| ENG-01-11 VM Luau | `evidence.rs` (budget adversarial, require bloqueado, falha contida); `crates/script/src/lib.rs` |
| ENG-01-12-S01/S02/S04 bindings | `evidence.rs::pending_kill_cancels_spawn_from_script`, `draw_cannot_mutate_world` (E016), `ent_pos_and_set_vel_roundtrip`, `pos_without_transform_is_error`, `set_vel_without_transform_is_error` |
| ENG-01-13 assets | `crates/assets/tests/registry.rs` (PNG real→paleta, corrompido=E100, placeholder estável/reusado) |
| ENG-01-14 raster | `crates/gfx/tests/golden.rs` (clear exato; retângulos clipping; fixture `fixtures/fase1.txt`) |
| ENG-01-15 sprites/fonte | `golden.rs` (flip/blit/fora da tela/sobreposição/empate estável; PNG vs LUT); `font.rs` (fallback de glifo desconhecido); `evidence.rs::camera_does_not_change_world_or_hash` |
| ENG-01-16 CLI/formatter | `crates/app/tests/cli.rs` (exit codes, stderr estruturado, artefatos); `formats.rs` (roundtrip TOON↔JSON) |
| ENG-01-17-S02/S03 check | `cli.rs::check_ok_on_example`, `corrupt_asset_is_effective_error_not_silence`, `script_syntax_error_is_effective_error` |
| ENG-01-18 headless/screenshot | `evidence.rs::screenshot_png_dimensions_and_hash_fixture` (+ `fixtures/hello_f60_seed42.txt`); `cli.rs::screenshot_writes_artifacts_where_told` |

## Limites declarados

- **#53 (ambiente strict):** `require`/imports **não existem** na fase 1 (sandbox bloqueia; proposta). A rejeição é provada; suporte a módulos entra com os kits (fase 3+).
- **#60 (resolução de nomes):** placeholders são resolvidos uma vez e reusados (sem criação por frame); benchmark formal fica com o `engine profile` (fase 2).
- **#58:** cobre pendentes e ausência de Transform; consulta de posição (`ent.pos`) ainda não existe (#59).
- **#20 (troca de backend):** bloqueado até as traits de fronteira existirem (ENG-01-04-S02, com o Taylor).
