# Repository Guidelines

## Project Structure & Module Organization
- `bin/` contains the executable services (e.g., `alisa`, `elisa`, `elizabeth`, `isabel`, `elisheba`), each with its own `src/` and Dockerfiles.
- `lib/` hosts shared crates (e.g., `transport`, `alice`, `xiaomi`, `crypto`) used by the binaries.
- `conf/` holds Docker and deployment configuration (for example, `conf/arm64.Dockerfile`).
- `build/` stores built artifacts copied from Docker images; `target/` is the Cargo build output.

## Build, Test, and Development Commands
- `cargo build` builds the full workspace.
- `cargo run --bin alisa` runs a specific service locally; replace `alisa` with another binary name.
- `make run_alisa` (and `make run_elisa`, `make run_elizabeth`, etc.) runs a service with expected environment variables pre-set.
- `make build_alisa` builds a Docker image and exports the binary to `build/` (repeat for other services).
- `cargo test` runs unit tests across the workspace; `make test` runs Docker-based test builds.
- Local builds that touch MQTT require CMake (`paho-mqtt-sys`); install with `brew install cmake` on macOS.

## Coding Style & Naming Conventions
- Rust code follows standard formatting (4-space indentation); use `cargo fmt` before submitting.
- Names: `snake_case` for modules/functions, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Keep module boundaries clean: shared logic goes to `lib/*`, service-specific logic stays in `bin/*`.

## Testing Guidelines
- Tests are inline in modules (`mod tests`) under `bin/*` and `lib/*`.
- After each change, run `cargo check`.
- Run `cargo fmt` after each change.
- After implementing a task or fixing a bug, run `cargo test` to ensure no regressions.
- Do not commit until `cargo fmt`, `cargo check`, and `cargo test` have been run successfully.
- Before marking a todo list step complete, run `cargo fmt`, `cargo check`, and `cargo test`, then stop for review and commit the changes to the branch.
- Run all tests with `cargo test`; for Docker build validation use `make test`.
- Add tests alongside the module you are changing to keep coverage close to the code.
- Manual `elisa` MQTT checks: publish to `action/request` and `state/request` with MQTT v5 `ResponseTopic` set; responses arrive on `action/response/<uuid>` or `state/response/<uuid>`.
- Example action request (cleanup + speed in one payload):
  - `mosquitto_pub -h localhost -p 1883 -u elisa -P 123mqtt -t action/request -V mqttv5 -D publish response-topic action/response/TEST-UUID -m '{"actions":[{"elisa":[{"set_cleanup_mode":"dry_cleaning"},"ID-1"]},{"elisa":[{"set_work_speed":"turbo"},"ID-2"]}]}'`
- Example state request:
  - `mosquitto_pub -h localhost -p 1883 -u elisa -P 123mqtt -t state/request -V mqttv5 -D publish response-topic state/response/TEST-UUID -m '{"device_ids":["vacuum_cleaner/living_room"]}'`

## Commit & Pull Request Guidelines
- Commit messages are short, imperative sentences (e.g., “Increase refresh token expiration duration”).
- Only commit after `cargo fmt`, `cargo check`, and `cargo test` have completed successfully.
- PRs should explain the change, link related issues, and include notes on config/env changes.
- If the change affects runtime behavior, add a brief manual test note or log snippet.
- Use `gt` (Graphite) for managing PRs in this project.
- For multi-step plans, create a draft PR when starting implementation and update the PR after each completed step.
- After plan completion update the PR description using `gh`, then publish it.

## Security & Configuration Tips
- Several run targets expect secrets from 1Password (`op read ...`) and MQTT credentials; avoid hard-coding secrets.
- Keep local config in env vars and document new required variables in the PR description.

## Roborock Local Notes
- `lib/roborock` supports L01 local protocol only; V1/A01/B01/MAP are not implemented.
- Local RPC responses can arrive as `GeneralResponse` (5) or `RpcResponse` (102); handle both.
- RPC payload format: outer `{"dps":{"101":"<inner_json>"},"t":<unix_timestamp>}`, inner `{"id":<request_id>,"method":"<method>","params":<params>}`.
- `set_custom_mode` uses params list: `[<fan_speed_code>]`.
- `set_water_box_custom_mode` uses params object: `{"water_box_mode": <code>}`.
- `app_segment_clean` uses params list: `[{ "segments": [<segment_id>], "repeat": 1 }]`.
- When stopping `elisa` locally, prefer `pkill -f target/debug/elisa` over `pgrep`.

## Rust Toolchain & Base Builder Tags
- `rust-toolchain.toml` pins the local Rust toolchain (e.g., `channel = "1.92.0"`).
- `.rust-version` tracks the base builder image tag and uses the `_N` revision suffix (e.g., `1.92.0_1`).
- PR CI checks that `.rust-version` matches the toolchain version when the suffix is stripped.
