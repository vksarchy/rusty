# Rust

Learning workspace. Everything here is reproducible from the flake; nothing
depends on a `cargo install` or a `rustup` toolchain living in `$HOME`.

## What is installed, and where it comes from

The toolchain is declarative, in `~/nixos`:

| Thing | Where it is declared |
| --- | --- |
| `rustc` `cargo` `clippy` `rustfmt` `rust-analyzer` `rustlings` `bacon` `mdbook` | `modules/packages.nix` → `rustPackages` |
| `RUST_SRC_PATH` (so the LSP can see into `std`) | `home/common.nix` → `sessionVariables` |
| VSCodium's rust-analyzer + CodeLLDB, and their settings | `home/vscodium.nix` |
| `rustbook` / `ruststd` / `rustex` / `rustdocs` aliases | `home/zsh.nix` |

`rustup` was deliberately removed. It downloads prebuilt, dynamically linked
toolchains that cannot run against this system's `stub-ld`, and it kept the
live compiler outside the flake. The trade is that `cargo +nightly` and
`rustup component add miri` are gone; when a project needs either, it pins its
own toolchain in a `devShell` — which is the reproducible way regardless.

**Do not `cargo install` anything.** It writes to `~/.cargo/bin`, which the
flake does not know about and `nh os switch` will never update. If a tool is
missing, add it to `rustPackages` or to this folder's `flake.nix`.

## Reading

```
rustbook     # The Book, offline, version-locked to the installed rustc
ruststd      # std API docs — you will open this several times an hour
rustex       # Rust by Example
rustdocs     # index of all of it: Reference, Nomicon, Cargo book, Clippy book
```

Prefer these to the website. They match the compiler actually installed, which
docs.rust-lang.org cannot promise, and they work offline.

`book-src/the-rust-book/` is the Book's *source* — a real mdBook project. Two
uses: its `listings/` directory holds every code example as a compiling crate
(far better than copying out of a browser), and it is a non-trivial mdBook to
practise `mdbook serve` against.

## The path

Do not read the Book cover to cover and then start writing. Alternate.

**1 — Mechanics (ch. 1–6, 8).** Syntax, `cargo`, structs, enums, `match`,
`Vec`/`String`/`HashMap`. Fast. Run `rustlings` alongside it.

**2 — Ownership (ch. 4, 10, 15).** The only genuinely new idea in the language.
Borrowing, lifetimes, `Box`/`Rc`/`RefCell`. Most people bounce here, reach for
`.clone()` everywhere, and plateau. Don't rush: this chapter is the language.

**3 — Errors and traits (ch. 9, 10, 13, 17).** `Result`, `?`, `thiserror` /
`anyhow`, trait bounds, iterators and closures. Idiomatic Rust is mostly
iterator chains and `?`; if you are writing `for` loops with index variables
and `unwrap()`, you are writing C in Rust.

**4 — Build something real.** The jump that actually matters. Concrete
suggestion given what you already run: pick one of the shell aliases in
`home/zsh.nix` and rewrite it as a Rust CLI with `clap`. Small, genuinely
useful, and it forces you through args, IO, errors and exit codes at once.

**5 — Concurrency (ch. 16) and then async.** Threads, `Send`/`Sync`, channels
first. `tokio` only after that — async in Rust is much harder to learn if
ownership and traits are not already solid.

**6 — Idiom.** Turn on `clippy::pedantic` (already on in VSCodium) and actually
read what it says; it is the single best idiom tutor available. Then read the
Rust API Guidelines, and start reading `std`'s own source — `ruststd` has a
`[src]` link on every item.

## The working loop

```sh
cd ~/rust && direnv allow        # once; afterwards the shell loads on cd
cargo new projects/thing && cd projects/thing
bacon clippy                     # leave running in a split: live lints on save
cargo nextest run                # tests, with readable failures
```

Three habits worth forming early:

- **Read the whole compiler error.** Rust's errors include the fix surprisingly
  often. Skimming to the red line and guessing is the slowest way to learn.
- **Let the types tell you.** Inlay hints are on in VSCodium — the inferred
  type of every binding is written into the buffer. Watch them while you edit.
- **Reach for `clone()` consciously.** It is not forbidden, but each one is a
  place you have not yet understood the ownership. Note them, revisit later.

## Layout

```
flake.nix          dev shell for this folder (pinned toolchain + extra tools)
.envrc             direnv hook; `direnv allow` once
book-src/          the Book's mdBook source + every listing as a real crate
exercises/         rustlings lives here
projects/          your own crates
```
