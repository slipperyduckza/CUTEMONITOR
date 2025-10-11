# Agent Instructions for Cutemonitor

## Build Commands
- Build: `cargo build`
- Release build: `cargo build --release`
- Run: `cargo run`

## Test Commands
- Run all tests: `cargo test`
- Run specific test: `cargo test <test_name>`
- Test with output: `cargo test -- --nocapture`

## Lint and Format
- Lint: `cargo clippy`
- Format: `cargo fmt`
- Check format: `cargo fmt --check`

## Code Style Guidelines

### Imports
- Group imports: std, external crates, then local modules
- Use explicit imports over glob imports
- Example: `use iced::{Application, Command, Element, Settings, Subscription, Theme};`

### Naming Conventions
- Functions/variables: snake_case
- Types/structs/enums: PascalCase
- Constants: SCREAMING_SNAKE_CASE
- Modules: snake_case

### Error Handling
- Use `Result<T, E>` for fallible operations
- Prefer `?` operator for error propagation
- Use `unwrap()` only when certain the operation won't fail
- Handle Windows API errors properly with `is_ok()` checks

### Types and Safety
- Use explicit types where clarity is needed
- Prefer safe APIs over unsafe when possible
- Use `unsafe` blocks only when necessary (e.g., Windows FFI)

### Formatting
- Use `cargo fmt` for consistent formatting
- Follow standard Rust formatting conventions
- Keep lines under 100 characters when possible

### Dependencies
- iced: GUI framework
- sysinfo: System information
- tokio: Async runtime
- windows: Windows API bindings