# Rust Code Refactoring Plan

## Overview

This plan focuses on fixing bugs, improving error handling with `thiserror`, and replacing unsafe `unwrap()` calls throughout the codebase. All changes maintain API compatibility and require no new dependencies.

## Issues to Address

### 1. Fix Duplicate Configuration Loading in main.rs

**File**: [src/main.rs](src/main.rs)

**Problem**: `settings::try_setup()` is called twice (lines 16 and 31), causing unnecessary duplicate work and potential inconsistencies.

**Solution**:

- Remove the first call (lines 16-26)
- Keep only the second call that includes `trace_log()`
- Update logging setup to use the single configuration instance

**Changes**:

- Remove lines 16-26 (first `try_setup()` call)
- Keep lines 31-44 but adjust to handle initial logging setup
- Update line 27-29 to work with the single config instance

---

### 2. Replace Unsafe unwrap() Calls with Proper Error Handling

#### 2.1 main.rs - RwLock Poisoning

**File**: [src/main.rs](src/main.rs)

**Problem**: Multiple `unwrap()` calls on `RwLock::read()` that could panic if the lock is poisoned.

**Lines affected**: 28, 45

**Solution**:

- Create a new error type using `thiserror` for lock poisoning
- Replace `unwrap()` with proper error handling
- Propagate errors up to `main()` which already returns `Result<(), String>`

**New error type** (add to a new `src/errors.rs` or existing error module):

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),
    // ... other errors
}
```

#### 2.2 runner.rs - Process Exit Code Handling

**File**: [src/cmd/runner.rs](src/cmd/runner.rs)

**Problem**: Line 123 uses `wait_for_child.code().unwrap()` which panics if the process was terminated by a signal (returns `None`).

**Solution**:

- Handle `None` case explicitly
- Return an appropriate exit code (e.g., 130 for SIGINT, 143 for SIGTERM, or 128 for unknown signal)
- Or add to `CommandError` enum

#### 2.3 runner.rs - Stdin Write Error Handling

**File**: [src/cmd/runner.rs](src/cmd/runner.rs)

**Problem**: Lines 111-112 use `unwrap_or_default()` which silently ignores stdin write errors.

**Solution**:

- Replace with proper error handling
- Return `CommandError` variant for stdin write failures
- Add new error variant to `CommandError` enum in [src/cmd/errors.rs](src/cmd/errors.rs)

**New error variant**:

```rust
#[error("could not write to command {command:?} stdin: {message}")]
WriteToCommandStdin {
    command: PathBuf,
    message: io::Error,
},
```

Note: This error variant is already commented out in the file (lines 30-35) - uncomment and use it.

#### 2.4 http.rs - RwLock Access

**File**: [src/http.rs](src/http.rs)

**Problem**: Many `unwrap()` calls on `RwLock::read()` and `RwLock::write()` throughout the file.

**Solution**:

- Create HTTP-specific error type using `thiserror`
- Replace `unwrap()` calls with proper error handling
- Propagate errors through handler functions

**New error type** (add to `http.rs` or create `src/http/errors.rs`):

```rust
#[derive(Error, Debug)]
pub enum HTTPInternalError {
    #[error("Configuration lock poisoned: {0}")]
    ConfigLockPoisoned(String),
    #[error("Commands lock poisoned: {0}")]
    CommandsLockPoisoned(String),
    #[error("Tokens lock poisoned: {0}")]
    TokensLockPoisoned(String),
    #[error("CAPTCHA lock poisoned: {0}")]
    CaptchaLockPoisoned(String),
}
```

Convert these to `HTTPError::API` or handle appropriately in each handler.

#### 2.5 settings.rs - Path and String Conversions

**File**: [src/settings.rs](src/settings.rs)

**Problem**: Multiple `unwrap()` calls on `PathBuf::to_str()` and `current_dir()` that could fail.

**Solution**:

- Add error variants to `CfgError` for these cases
- Replace `unwrap()` with proper error handling
- Handle cases where paths contain invalid UTF-8

---

### 3. Improve Error Type Consistency

**Problem**: Some functions return `Result<T, String>` instead of using proper error types with `thiserror`.

**Files affected**:

- [src/main.rs](src/main.rs) - `main()` returns `Result<(), String>`
- [src/cmd/mod.rs](src/cmd/mod.rs) - `check_input()` returns `Result<CommandInput, String>`
- Various other locations

**Solution**:

- Create a top-level `AppError` type using `thiserror` that can wrap other errors
- Convert `Result<T, String>` to use proper error types where it makes sense
- For `main()`, keep `Result<(), String>` but convert internal errors to strings properly

---

### 4. Fix Potential Panics in Tree Operations

**File**: [src/cmd/tree.rs](src/cmd/tree.rs)

**Problem**: Multiple `unwrap()` calls on `file_name()`, `to_str()`, and `strip_prefix()` that could panic.

**Lines affected**: 186, 190, 196, 218, 240, 242, 246, 276, 278

**Solution**:

- Add error handling for cases where:
  - File names are not valid UTF-8
  - Path operations fail
- Return appropriate `CommandError` variants
- Handle edge cases gracefully

---

### 5. Improve Error Messages

**Problem**: Some error messages could be more descriptive.

**Solution**:

- Review error messages in existing `thiserror` types
- Ensure all error variants provide useful context
- Add field information where helpful

---

## Implementation Order

1. **Phase 1: Critical Bug Fixes**

   - Fix duplicate `try_setup()` in `main.rs`
   - Fix stdin write error handling in `runner.rs`
   - Fix process exit code handling in `runner.rs`

2. **Phase 2: Error Type Improvements**

   - Create/update error types using `thiserror`
   - Add missing error variants (uncomment existing ones where appropriate)

3. **Phase 3: Replace unwrap() Calls**

   - Replace `RwLock` unwrap calls in `main.rs` and `http.rs`
   - Replace path/string conversion unwraps in `settings.rs` and `tree.rs`
   - Replace other critical unwrap calls

4. **Phase 4: Error Propagation**

   - Update function signatures to use proper error types
   - Ensure errors propagate correctly through call chains

---

## 6. Lock Usage Analysis and Potential Removal

**Analysis of current lock usage:**

1. **Configuration (`Arc<RwLock<Cfg>>`)**: 

   - Mostly read-only (19 read operations)
   - Only 1 write operation: password update in `try_set_password()` (line 1051)
   - Used across multiple concurrent HTTP request handlers

2. **Commands (`Arc<RwLock<Command>>`)**: 

   - Many read operations (command tree traversal)
   - Write operation: `reload()` called on every GET `/api/commands` request (line 504)
   - Used across multiple concurrent HTTP request handlers

3. **Tokens (`Arc<RwLock<HashMap<String, usize>>>`)**: 

   - Concurrent read/write for token validation and insertion
   - Must remain locked for thread safety

4. **CAPTCHA (`Option<Arc<RwLock<captcha::Captcha>>>`)**: 

   - Concurrent read/write for generation and validation
   - Must remain locked for thread safety

**Conclusion**:

Since rouille is multi-threaded and handles requests concurrently, we need synchronization for shared mutable state.

**Potential lock removal options:**

- **Option A (Requires API changes)**: Convert `cfg` and `commands` to `Arc<Cfg>` and `Arc<Command>` (immutable), and handle updates by cloning and replacing the Arc. However, this requires atomic replacement which would need a dependency like `arc-swap` (not allowed) OR accepting potential race conditions.

- **Option B (No API changes)**: Keep locks but optimize usage - use `Arc<Mutex<>>` instead of `Arc<RwLock<>>` for cases with infrequent writes (like config password updates), which is simpler and slightly more efficient for write-heavy operations.

**Recommendation**: Keep the locks as-is. The current `RwLock` usage is appropriate for the read-heavy, occasionally-written patterns. Removing locks would require either new dependencies (not allowed) or significant API changes that could break compatibility.

**Action**: No lock removal step will be added to the plan, as it's not feasible within the constraints.

---

## Notes

- All changes maintain API compatibility (same function signatures, same behavior)
- No new dependencies will be added
- No async code will be introduced
- Functions will not be split unless logic is duplicated 3+ times
- Large functions are acceptable per project preferences
- Error handling improvements focus on safety, not performance optimization
- Locks will remain as `Arc<RwLock<>>` - removal not feasible without new dependencies or breaking API changes