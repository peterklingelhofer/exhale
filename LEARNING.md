# Learning Rust by reading exhale

This is a beginner's guide to the Rust port of exhale. It assumes **zero Rust knowledge**. By the end you should be able to open any `.rs` file in this repo and have a rough idea of what's going on.

If you've never read Rust before, the syntax looks intimidating. Mostly it's not. Rust is just a language that's a little more honest than most about who owns which piece of memory and what's allowed to share it. Once you internalize that one rule, 80% of what looks weird stops looking weird.

## What is Rust?

A compiled language (like C++, Go, or Swift, not like Python or JavaScript). Source code goes through a compiler and produces a native binary that runs directly on the CPU. Rust's pitch is: "all the speed of C++, without the crashes from forgetting how memory works."

The way Rust achieves that is a strict set of rules about who is allowed to read or modify a piece of data, enforced by the compiler. You'll bump into those rules constantly while learning. Don't fight them. The compiler errors are weirdly helpful (we'll get to those).

## The shape of this repo

```
rust/
├── Cargo.toml                  ← top-level "workspace" file, lists the three crates
├── crates/
│   ├── exhale-core/            ← settings, breathing math, no UI
│   ├── exhale-render/          ← GPU rendering (wgpu shaders)
│   └── exhale-app/             ← the actual app: window, tray, event loop
└── target/                     ← build output (gitignored)
```

A **crate** is Rust's word for a package. Think of it like one `.jar` in Java or one `node_modules/<pkg>` in Node. This repo is a **workspace**: multiple crates that build together.

The three crates have a layered relationship:

```
exhale-app  ─uses→  exhale-render  ─uses→  exhale-core
```

`exhale-core` knows nothing about windows or pixels. `exhale-render` knows how to draw, but nothing about windows. `exhale-app` glues them to a real OS window and tray icon. Each layer is independently testable.

## Setting up the docs

Rust generates HTML documentation from doc-comments in the source code. To browse it:

```sh
cd rust
cargo doc --no-deps --workspace --open
```

- `--no-deps` skips generating docs for every dependency (you'd get 200+ extra crates' worth, which is overwhelming on first read)
- `--workspace` includes all three of our crates
- `--open` pops the result in your browser

You'll land on a page showing the three crates. Click `exhale_core` first — it has the least going on, and the breathing-animation math is genuinely interesting.

The docs are most useful for the **type-level view**. For each struct (data shape) you'll see its fields and methods. Clicking a method jumps to its source code on the right side. Treat it like an interactive table of contents for the codebase.

## The 10 things you'll see in every Rust file

### 1. Variables: `let` and `let mut`

```rust
let x = 5;              // can't change x
let mut y = 5;          // can change y
y = 10;                 // fine
```

By default everything is **immutable**. If you want to be able to change a variable, you say `let mut` explicitly. This is the opposite of most languages and is annoying for the first day. It's a feature, not a bug — it means you can scan a file and instantly see which variables get mutated.

You'll see types annotated sometimes, often not:

```rust
let count: u32 = 5;     // u32 = unsigned 32-bit integer
let count    = 5u32;    // same thing, different syntax
let count    = 5;       // Rust infers u32 (or whatever fits)
```

### 2. Functions: `fn name(args) -> Return { body }`

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b   // no semicolon = "this is the return value"
}
```

The last expression in a block is automatically returned. If you put a semicolon at the end, it becomes a statement instead and returns the unit type `()` (pronounced "unit", basically `void` in C).

Methods (functions attached to a type) take `&self`, `&mut self`, or `self` as their first arg:

```rust
impl Settings {
    fn is_valid(&self) -> bool { ... }        // read-only borrow of self
    fn reset(&mut self)        { ... }        // mutable borrow
    fn consume(self)           { ... }        // takes ownership, self is gone after
}
```

### 3. Ownership and borrowing (the only rule that matters)

Every piece of data has exactly one **owner**. When the owner goes out of scope, the data is freed. This replaces garbage collection.

```rust
let s = String::from("hello");
// `s` owns the string. When this function ends, the string is freed.
```

You can let other code **borrow** the data temporarily without giving up ownership:

```rust
let s = String::from("hello");
let length = compute_length(&s);   // & = "borrow, don't take"
println!("{} is {} chars", s, length);   // s is still ours
```

Two flavors of borrows:
- `&T` — shared borrow. Many readers allowed, no writers.
- `&mut T` — exclusive borrow. One writer allowed, no other readers or writers.

That rule (one OR the other, never both) is what prevents data races at compile time. You'll see it everywhere. When the compiler refuses to compile something, it's usually because you tried to break this rule.

Look at `crates/exhale-core/src/controller.rs` and you'll see `&mut Settings` pop up — that's a function signature saying "give me exclusive write access to a Settings, just for this call."

### 4. `Option<T>` instead of `null`

Rust has no `null`. If a value might be absent, the type system says so:

```rust
let maybe_a_number: Option<u32> = Some(42);
let definitely_nothing: Option<u32> = None;
```

To use the value you have to handle both cases:

```rust
match maybe_a_number {
    Some(n) => println!("got {}", n),
    None    => println!("nothing here"),
}
```

This is verbose but rules out the entire category of "null pointer exception" bugs. You'll see `Option<&T>`, `Option<KeyboardShortcut>`, `Option<MenuItem>` all over this codebase.

### 5. `Result<T, E>` and the `?` operator for errors

Rust has no exceptions. Failures are values:

```rust
fn read_settings() -> Result<Settings, std::io::Error> {
    // ...
}
```

`Result<T, E>` means "either a `T` on success or an `E` on failure." To handle both:

```rust
match read_settings() {
    Ok(s)  => println!("loaded {:?}", s),
    Err(e) => eprintln!("oops: {}", e),
}
```

The `?` operator is shorthand for "if this is an error, return it from the current function immediately; otherwise unwrap the success value":

```rust
fn do_two_things() -> Result<(), Error> {
    let s = read_settings()?;       // if Err, bail out here
    save_to_disk(&s)?;              // if Err, bail out here
    Ok(())                          // both worked, return success
}
```

You'll see `?` constantly. It's the single biggest reason Rust code stays readable despite handling every possible failure.

### 6. `Arc<RwLock<T>>` for shared mutable state across threads

When two threads need to share a piece of mutable data, you wrap it like this:

```rust
let settings = Arc::new(RwLock::new(Settings::default()));
let settings_for_thread_2 = Arc::clone(&settings);
```

- `Arc<T>` = "atomically reference-counted." Multiple owners, freed when the last one drops. Cheap to clone.
- `RwLock<T>` = "read-write lock." Many readers OR one writer at a time.

To read or write through it:

```rust
let snapshot = settings.read().unwrap();          // shared read access
let mut writable = settings.write().unwrap();     // exclusive write
writable.is_paused = true;
```

The `.unwrap()` is "this returns a Result, give me the inner value or panic" — used here because we trust the lock won't be poisoned (a thread holding the lock didn't crash mid-write).

This pattern is everywhere in `crates/exhale-app/src/main.rs` — the settings, controller state, and tray menu all use it because the GUI thread and the controller thread both need access.

### 7. `match`: pattern matching that's worth knowing

```rust
match phase {
    BreathingPhase::Inhale       => "in",
    BreathingPhase::PostInhale   => "hold",
    BreathingPhase::Exhale       => "out",
    BreathingPhase::PostExhale   => "hold",
}
```

`match` is like a `switch` but the compiler checks that you handled every possible case. If `BreathingPhase` gains a fifth variant tomorrow, every `match` on it stops compiling until you add the new arm. That's a feature.

You can also destructure structs and tuples:

```rust
let (x, y) = (10, 20);
let Settings { is_paused, opacity, .. } = settings;
```

### 8. Traits and `impl` blocks

A **trait** is what other languages call an "interface" or "protocol." It's a set of methods that types can claim to implement.

```rust
trait Drawable {
    fn draw(&self);
}

impl Drawable for Circle {
    fn draw(&self) { /* draw a circle */ }
}

impl Drawable for Rectangle {
    fn draw(&self) { /* draw a rectangle */ }
}
```

In exhale you'll see traits used heavily by external libraries. For example `winit::application::ApplicationHandler` is a trait — `crates/exhale-app/src/main.rs` implements it for the `App` struct, which tells winit "here's how to forward window events to me."

Some traits are special. `Send` means "safe to move between threads." `Sync` means "safe to share between threads." You'll see them as bounds: `T: Send + Sync` means "T must be both."

### 9. Generics: `<T>` is "type-to-be-decided-later"

```rust
fn first<T>(items: &[T]) -> &T {
    &items[0]
}
```

`T` is a placeholder. The compiler stamps out a concrete copy of `first` for each type you actually use it with (`first<i32>`, `first<String>`, etc.).

You can constrain `T` with **bounds**:

```rust
fn print_all<T: std::fmt::Display>(items: &[T]) {
    for item in items {
        println!("{}", item);
    }
}
```

That `T: std::fmt::Display` means "T must implement the Display trait" — only types that know how to print themselves are accepted. wgpu types use bounds heavily (`T: bytemuck::Pod` etc.).

### 10. `unsafe` blocks

```rust
unsafe {
    let title: *mut AnyObject = msg_send![ns_string, stringWithUTF8String: c"exhale".as_ptr()];
}
```

`unsafe` is a promise from you (the programmer) to the compiler: "I know this could break the borrow checker's rules, and I've thought about it." Inside `unsafe { }`, you can dereference raw pointers and call C functions.

We use it only where we have to: talking to Apple's Objective-C APIs (`crates/exhale-app/src/platform/mac.rs`), calling Win32 APIs, or loading X11 functions on Linux. About 1% of the codebase. Everything else is "safe Rust" — the compiler proves it can't crash on memory safety issues.

### Bonus: `#[cfg(...)]` for conditional compilation

```rust
#[cfg(target_os = "macos")]
fn dock_only_visibility() { /* macOS implementation */ }

#[cfg(target_os = "windows")]
fn dock_only_visibility() { /* Windows implementation */ }
```

`#[cfg(...)]` lines compile their item only when the condition matches. This is how we ship one source tree that produces three different platform binaries. You'll see `#[cfg(feature = "global-hotkeys")]` too — that gates code on a Cargo feature flag (the Mac App Store build disables it because Apple's sandbox blocks global hotkeys).

### Bonus: modules and `use`

```rust
mod settings;                            // there's a file settings.rs next to me
mod settings_window;                     // there's a file settings_window.rs

use std::sync::Arc;                      // bring Arc into scope
use exhale_core::Settings;               // bring our Settings into scope
use crate::overlay::OverlayManager;      // `crate::` means "starting from this crate's root"
```

`mod` declares a submodule (basically "this file is part of this crate"). `use` is the equivalent of `import` in Python or `using` in C#.

## A reading path: follow Start end-to-end

Pick one user action and trace what happens. Best one for this repo: clicking the "Start Animation" button.

1. `crates/exhale-app/src/settings_window.rs` — the button is drawn here, and clicking it sends an `AppEvent::StartAnimation` via `proxy.send_event(...)`. Search for `StartAnimation` to find the click handler.
2. `crates/exhale-app/src/main.rs` — `App::user_event(...)` matches `AppEvent::StartAnimation` and calls `self.do_start()`. Search for `fn do_start`.
3. `do_start` flips `settings.is_animating = true` and calls `controller.restart()`.
4. `crates/exhale-core/src/controller.rs` — `restart()` flips a flag and unparks the controller thread. That thread is running `tick()` in a loop. Search for `fn tick`.
5. `tick()` computes the current frame's `BreathingState` and tells the renderer to draw it.
6. `crates/exhale-render/src/renderer.rs` — the renderer's `render()` method uploads uniforms to the GPU and submits a draw call. That's what you see on screen.

Open all four files side by side and walk through it. That's enough to understand the architecture without reading 50 other things first.

## How to read Rust compiler errors

The Rust compiler is the best teacher you'll have. Its errors look intimidating but they're structured:

```
error[E0382]: borrow of moved value: `settings`
   --> src/main.rs:42:13
    |
40  | let snapshot = settings;
    |                -------- value moved here
42  | println!("{:?}", settings);
    |             ^^^^^^^^ value borrowed here after move
    |
help: consider cloning the value if the performance cost is acceptable
    |
40  | let snapshot = settings.clone();
    |                        ++++++++
```

Three parts:
- **Error code + message** (`E0382: borrow of moved value`)
- **Where it happened** (with source snippets and arrows pointing at the lines)
- **A suggested fix**

Read every error from top to bottom and look for the `help:` block. About 80% of the time, the suggested fix is correct. The remaining 20% the suggestion is a hint that you've designed something wrong at a higher level.

`rustc --explain E0382` will give you the full essay version of any error code. Try it on a few — it's a free crash course in why Rust's rules exist.

## What to skip on first read

- **Lifetimes** (`'a`, `'static`, etc.). They show up in a few advanced spots and the compiler usually elides them for you. Cross that bridge later.
- **Macros**. You'll see `println!`, `format!`, `vec!` (note the `!`). They're like functions but operate on syntax. Using them is easy; writing them is advanced.
- **Async / await**. Almost not used in this codebase. The threading model is plain OS threads.
- **Procedural macros** like `#[derive(Debug, Clone)]`. Just trust that they generate boilerplate at compile time. The `derive(Debug)` one is what makes `{:?}` printing work.

## When you get stuck

- Read the compiler error twice. Don't skim it.
- `rustc --explain E0XXX` for the long version.
- Open the source of the third-party type the compiler is complaining about (the docs link goes there, or `cargo doc` includes it).
- Sometimes the right move is to print the type: `let _: () = the_thing;` makes the compiler say "expected `()`, found `<actual type>`" which tells you what the actual type is.
- The Rust book at https://doc.rust-lang.org/book/ is the canonical free reference. Chapters 4 (ownership) and 9 (error handling) are the load-bearing ones.

## What you DON'T need to learn to read this codebase

- Async/await
- Pin/Unpin
- Macros 2.0
- Custom procedural macros
- `Pin<Box<dyn Future>>`
- `'static` lifetime details
- Variance and HRTB

If you see any of those in the wild, just trust they do what they look like they're doing.

## Try it

Open `crates/exhale-core/src/controller.rs` in your editor. Skim from the top. When you hit something that looks weird, check this guide. When you don't see it in this guide, run `cargo doc --no-deps --workspace --open` and look up the type.

After 30 minutes you'll have read most of the controller. After an hour you'll have a real intuition for what the whole repo is doing. That's faster than reading any Rust book front-to-back.
