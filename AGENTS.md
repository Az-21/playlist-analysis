# AGENTS.md

## Core Principles

Write idiomatic Rust, not C or C++ translated into Rust. The borrow checker, the type system, and iterators exist to make correct code the easy path. If the code feels like it is fighting the compiler, the design is usually the problem, not the compiler.

- Prefer correctness and clarity over cleverness.
- Prefer the type system over runtime checks and comments.
- Prefer small, composable, pure pieces over large stateful ones.
- Make illegal states unrepresentable.
- Keep changes minimal and focused on the task at hand.

## Required Checks Before Finishing a Task

Run these commands and fix everything they report before declaring a task done.

```sh
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

If `cargo nextest` is installed, prefer `cargo nextest run` for speed. Doc tests still need `cargo test --doc`. Never disable a failing test to get a green build; fix the code or the test.

## Project Layout and Modularity

Organize code by responsibility, not by type kind. Group things that change together, and keep unrelated concerns apart. Avoid giant `utils`, `helpers`, `common`, or `misc` modules; if a function has no natural home, that is a sign the domain model is missing a concept.

### Module Files Use mod.rs and tests.rs

Do not create `something.rs` next to a `something/` directory. Every module is a directory with a `mod.rs`, and its unit tests live in a sibling `tests.rs`.

```text
src/
  main.rs
  config/
    mod.rs
    tests.rs
  parser/
    mod.rs
    tests.rs
    lexer/
      mod.rs
      tests.rs
  pricing/
    mod.rs
    tests.rs
```

Wire the test module up at the bottom of `mod.rs`:

```rust
#[cfg(test)]
mod tests;
```

Inside `tests.rs`, import the parent with `use super::*;` and keep helpers private to that file. A leaf module that has no tests yet may start as a directory with only `mod.rs`, but still use the directory form so adding tests later does not move files.

### Slim Files with One Responsibility

Each file should do one thing and be easy to read in a single sitting.

- Aim for files under roughly 300 lines of non-test code. Treat 500 lines as a hard signal to split.
- Aim for functions under roughly 40 lines. Split earlier if a function mixes parsing, validation, and I/O.
- One primary type (plus its `impl` blocks and trait impls) per file is a good default. Small closely related helper types may share a file.
- `mod.rs` should mostly declare submodules and re-export the public surface (`pub use`). Avoid piling logic into `mod.rs`.
- Name files after the thing they contain (`retry_policy.rs`, not `stuff.rs`).

### Visibility and Public API

Default to the narrowest visibility that works: private, then `pub(super)`, then `pub(crate)`, and only then `pub`. Each module exposes a small, deliberate API through `mod.rs` re-exports, and callers should not reach into its internals. In library crates, treat every `pub` item as a promise to downstream users.

## Pure Functions and Separation of Concerns

Push side effects to the edges of the program and keep the core logic pure. A pure function depends only on its arguments, returns a value, and does not touch the filesystem, network, clock, environment, randomness, or global state.

- Split "decide" from "do". A function that computes what should happen is pure and trivially testable; a thin outer layer performs the I/O.
- Pass time, randomness, and configuration in as parameters (or behind a small trait) instead of calling `SystemTime::now()` or reading env vars deep inside logic.
- Prefer returning new values over mutating through `&mut` when the cost is small and the code gets clearer. Use `&mut self` when mutation is the natural model (builders, caches, state machines).
- Avoid hidden global state: no `static mut`, and no `lazy_static`/`OnceLock` singletons for things that should be passed explicitly.
- Prefer iterator chains and `match` expressions that produce values over loops that mutate accumulators, when they read better.

```rust
// Pure: easy to test, no I/O.
pub fn apply_discount(subtotal: Money, rule: &DiscountRule) -> Money {
    match rule {
        DiscountRule::Percent(p) => subtotal.scale_down(*p),
        DiscountRule::Flat(amount) => subtotal.saturating_sub(*amount),
        DiscountRule::None => subtotal,
    }
}

// Impure edge: thin, delegates to pure logic.
pub async fn checkout(db: &Db, cart_id: CartId) -> Result<Receipt, CheckoutError> {
    let cart = db.load_cart(cart_id).await?;
    let total = apply_discount(cart.subtotal(), &cart.discount);
    db.save_receipt(cart_id, total).await
}
```

## Unit Testing

Every non-trivial piece of logic needs unit tests, and every bug fix needs a regression test that fails before the fix.

### Where Tests Live

- Unit tests go in the module's `tests.rs` file (see the layout rules above).
- Integration tests that exercise the public API go in the top-level `tests/` directory, one file per feature area.
- Examples in doc comments are compiled and run as doc tests; keep them correct and minimal.

### How to Write Tests

- Name tests after behavior, not implementation: `rejects_empty_input`, `retries_three_times_then_fails`.
- Follow arrange, act, assert, and keep one logical assertion focus per test.
- Test pure functions directly with plain inputs and outputs. If a test needs heavy mocking, the code under test probably mixes logic with I/O; split it.
- Cover the happy path, boundaries (empty, one, max, off-by-one), and error paths. Assert on specific error variants, not just `is_err()`.
- Prefer `assert_eq!` and `assert_matches!`-style checks with clear messages over `assert!(a == b)`.
- Use table-driven tests for many similar cases, and property-based tests (`proptest`) for parsers, serializers, and anything with invariants such as round trips.
- Use snapshot tests (`insta`) for large structured output, and review snapshot diffs by hand.
- Tests must be deterministic and independent: no reliance on test order, wall-clock time, real network, or shared mutable state. Use `tempfile` for filesystem tests.
- Tests may use `unwrap()` and `expect()`; production code may not (see error handling).

```rust
use super::*;

#[test]
fn percent_discount_rounds_down() {
    let total = apply_discount(Money::from_cents(999), &DiscountRule::Percent(10));
    assert_eq!(total, Money::from_cents(899));
}

#[test]
fn flat_discount_never_goes_negative() {
    let total = apply_discount(Money::from_cents(500), &DiscountRule::Flat(Money::from_cents(900)));
    assert_eq!(total, Money::ZERO);
}
```

## Think in Rust, Not in C or C++

Many common Rust mistakes come from carrying over habits from C and C++. Check your code against this list.

### Ownership and Cloning

- Do not call `.clone()` to silence the borrow checker. First ask: can this take a reference, can the data be restructured, or should ownership simply move?
- Accept the most general borrowed type in parameters: `&str` over `&String`, `&[T]` over `&Vec<T>`, `&Path` over `&PathBuf`.
- Accept `impl AsRef<str>`, `impl Into<String>`, or `impl IntoIterator<Item = T>` when it makes the call site nicer, but do not make every function generic for no reason.
- Take ownership (`String`, `Vec<T>`) only when the function needs to store or consume the value.
- Use `Cow<'_, str>` when a function sometimes needs to allocate and sometimes does not.
- Share data with `Arc<T>` (or `Rc<T>` in single-threaded code) when shared ownership is the real requirement, and clone the `Arc`, not the data inside it.
- Avoid `.to_string()`, `.to_owned()`, and `.collect::<Vec<_>>()` in hot paths and in the middle of iterator chains when borrowing or lazy iteration works.
- Be deliberate about `Copy` and `Clone` derives. Derive `Copy` only for small plain value types.

### Memory, Pointers, and Unsafe

- Do not use raw pointers, `unsafe`, `transmute`, or manual memory management unless there is a measured need and a documented safety argument. The default is `#![forbid(unsafe_code)]`.
- Do not emulate nullable pointers with sentinel values. Use `Option<T>`.
- Do not emulate error codes or out-parameters. Return `Result<T, E>` or a struct/tuple.
- Do not index with integers when iterators work. Prefer `for item in &items`, `.iter().enumerate()`, `.windows()`, `.chunks()`, and `.zip()` over `for i in 0..items.len()`.
- Do not build "classes with getters and setters" or inheritance-style hierarchies. Use plain structs with public fields where invariants allow, enums for variants, and traits for shared behavior.
- Do not reach for `Box<dyn Trait>` by default. Prefer generics or enums when the set of types is known; use trait objects when you truly need runtime polymorphism.
- Avoid `Rc<RefCell<T>>` and `Arc<Mutex<T>>` webs that recreate a pointer graph. Prefer clear ownership, indices into a `Vec`, or message passing.

### Error Handling

- Never use `unwrap()`, `expect()`, `panic!`, `unreachable!`, or unchecked indexing in non-test production code for recoverable situations. Propagate errors with `?`.
- `expect("reason")` is acceptable only for true invariants that cannot fail, with a message that states the invariant.
- In libraries, define precise error enums with `thiserror`. In binaries and glue code, `anyhow` (or `eyre`) with `.context("what was being attempted")` is fine.
- Do not stringly-type errors. Do not return `Box<dyn Error>` from library APIs.
- Do not swallow errors with `let _ = ...` or `.ok()` unless ignoring is intentional and commented.

### Types and Data Modeling

- Model state with enums, not with booleans and nullable fields. Combine them with `match` and let the compiler enforce exhaustiveness (avoid wildcard `_` arms on enums you own).
- Use newtypes for IDs, units, and validated values (`UserId(u64)`, `Email(String)`) so they cannot be mixed up.
- Use the builder pattern or struct update syntax (`..Default::default()`) for types with many optional fields instead of long argument lists.
- Parse, do not validate: convert raw input into a validated type once at the boundary, and use that type everywhere inside.
- Use `#[must_use]` on functions and types whose results should not be ignored, and `#[non_exhaustive]` on public enums and structs that may grow.
- Derive standard traits (`Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`) only where they are meaningful and used.

### Style Habits to Avoid

- Do not write `return x;` at the end of a function; use the tail expression.
- Do not write `if x == true`, `match` on a single pattern when `if let` or `let ... else` is clearer, or manual `loop` plus `break` when `while let` fits.
- Do not allocate a `Vec` just to iterate it once. Return `impl Iterator` instead.
- Do not use `String` for everything. Use enums, newtypes, and `&'static str` where they fit.
- Do not hand-roll what the standard library or a well-known crate already does (sorting, parsing, hashing, path joins, time math).

## Modern Rust Practices

Use current language and library features. Check the release notes when unsure whether something newer and better exists.

- Use `let ... else` for early returns, and `if let ... else` chains (let chains, where the edition supports them) to flatten nesting.
- Use inline format arguments: `format!("{name} has {count} items")` rather than `format!("{} has {} items", name, count)`.
- Use `async fn` in traits and `impl Trait` in argument and return positions where they simplify signatures.
- Use `std::sync::LazyLock` and `OnceLock` instead of `lazy_static` and `once_cell` for statics that truly must exist.
- Use `std::iter`, `Iterator::try_fold`, `array::map`, `slice::chunk_by`, `is_some_and`, `Option::zip`, `Result::inspect_err`, and similar combinators when they make the code shorter and clearer.
- Use `const` and `const fn` for values and functions that can be evaluated at compile time.
- Use `#[derive(Default)]` with `#[default]` on enum variants instead of manual `Default` impls.
- Use `std::path::Path` and `PathBuf` for paths, `std::time::Duration` for durations, and `std::num::NonZero*` where zero is invalid.
- Prefer `?` and `Result` chains over nested `match` pyramids.
- Write documentation comments (`///` and `//!`) for public items, with a short summary line, and `# Errors`, `# Panics`, and `# Safety` sections where relevant.
- Do not split sentence into multiple lines to honor a line character limit in code comments.

## Dependencies and Latest Packages

Prefer well-maintained, widely used crates and keep them current.

- Add dependencies with `cargo add <crate>` so you get the latest published version, instead of copying version numbers from memory, old blog posts, or old projects. Verify the current version and API on crates.io and docs.rs before using it.
- Read the crate's changelog when upgrading across a major version; APIs change (for example, `rand`, `axum`, `reqwest`, `clap`, and `sqlx` have all had breaking releases).
- Enable only the features you need (`default-features = false` where sensible) to keep compile times and binary sizes down.
- Prefer the standard library when it is enough. Do not add a crate for a ten-line helper.
- Share versions across a workspace with `[workspace.dependencies]`.
- Before adding a crate, check that it is actively maintained, has reasonable adoption, and has an acceptable license and no known advisories.
- Run `cargo update` periodically, and run `cargo audit` and `cargo deny check` (when configured) in CI.

## Concurrency and Async

- Do not block inside async code. Use async-aware APIs, and move blocking work to `tokio::task::spawn_blocking`.
- Do not hold a `std::sync::Mutex` guard across an `.await`.
- Prefer message passing (channels) and clear task ownership overshared mutable state.
- Make cancellation safe: assume any `.await` point may be the last thing a future does.
- Let the compiler enforce thread safety through `Send` and `Sync`; do not work around it with `unsafe impl`.

## Logging and Observability

- Use `tracing` with structured fields (`tracing::info!(user_id = %id, "login succeeded")`) instead of `println!` or `eprintln!` in application code.
- Do not log secrets, tokens, or personal data.
- Library crates should emit events but never install a global subscriber.

## Agent Checklist

Before finishing any task, confirm each item.

- The code compiles, `cargo fmt` is clean, and `cargo clippy -- -D warnings` passes.
- New and changed logic has unit tests in the module's `tests.rs`.
- New modules use the `name/mod.rs` plus `name/tests.rs` layout, never `name.rs`.
- Logic is in pure functions, and I/O sits at the edges.
- No unnecessary `.clone()`, `.unwrap()`, `unsafe`, or C-style index loops were introduced.
- Files are slim and each has a single responsibility.
- New dependencies were added with `cargo add`, with only the needed features enabled.
- Public items have doc comments, and no unrelated files were changed.
