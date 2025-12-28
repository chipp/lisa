# Gemini Guidelines

This document provides a concise summary of project-specific guidelines for Gemini to follow when working in the `lisa` repository.

## Project Overview
- `bin/`: Executable services (e.g., `alisa`, `elisa`, `elizabeth`, `isabel`, `elisheba`).
- `lib/`: Shared crates (e.g., `transport`, `alice`, `xiaomi`, `crypto`).
- `conf/`: Configuration files (Docker, MQTT).

## Common Commands
- **Build All:** `cargo build`
- **Run Specific Bin:** `cargo run --bin <bin_name>` (e.g., `cargo run --bin alisa`)
- **Test All:** `cargo test`
- **Format Code:** `cargo fmt`
- **Check Code:** `cargo check`
- **Makefile:** Use `make run_<bin_name>` for local runs (requires `op` CLI and secrets).

## Development Workflow
1. **Understand:** Use `search_file_content` and `read_file` to explore the codebase.
2. **Implement:** Follow `snake_case` for functions/modules, `CamelCase` for types. 4-space indentation.
3. **Verify:**
   - Run `cargo check` after changes.
   - Run `cargo fmt` before submitting.
   - Add/run unit tests using `cargo test`.
   - Do not commit until `cargo fmt`, `cargo check`, and `cargo test` have succeeded.
4. **Hardware Specifics:** Refer to `AGENTS.md` for Roborock, Inspinia, or Sonoff specific protocol details.

## Testing
- Tests are typically inline: `mod tests` at the bottom of the file.
- For manual MQTT testing, refer to the examples in `AGENTS.md`.

## Commits
- Use short, imperative sentences (e.g., "Add timeout to Roborock discovery").
- Style should match existing commit history (`git log -n 3`).
- Only commit after `cargo fmt`, `cargo check`, and `cargo test` have completed successfully.

## Pull Requests
- Update PR descriptions using `gh`.
