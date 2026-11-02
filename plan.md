1. **Explore & Analyze the Code:**
    - Read `lib/middleware/request_tracker.rb`.
    - Read `lib/presence_channel.rb`.
    - Read `lib/middleware/anonymous_cache.rb`.
    - Understand the logic and dependencies.

2. **Translate to Rust:**
    - Implement `RequestTracker` logic in `crates/discourse_core/src/middleware_presence/request_tracker.rs`. Use appropriate traits/structs for dependencies.
    - Implement `PresenceChannel` logic in `crates/discourse_core/src/middleware_presence/presence_channel.rs`. Use mocks or traits for Redis and MessageBus.
    - Implement `AnonymousCache` logic in `crates/discourse_core/src/middleware_presence/anonymous_cache.rs`.

3. **Add `middleware_presence` to `lib.rs`:**
    - Update `crates/discourse_core/src/lib.rs` to include `pub mod middleware_presence;`.
    - Export submodules in `crates/discourse_core/src/middleware_presence/mod.rs`.

4. **Write Unit Tests:**
    - Write comprehensive unit tests in each `*.rs` file in a `#[cfg(test)] mod tests` block, validating behavior identically to the Ruby original.

5. **Ensure Compilation and Tests Pass:**
    - Run `cd crates/discourse_core && cargo test` to verify.

6. **Delete Original Ruby Files:**
    - Remove `lib/middleware/request_tracker.rb`, `lib/presence_channel.rb`, and `lib/middleware/anonymous_cache.rb`.

7. **Pre-commit Steps:**
    - Call `pre_commit_instructions` to ensure proper testing, verification, review, and reflection are done.

8. **Submit:**
    - Open a pull request against the `rust-migration` branch.
