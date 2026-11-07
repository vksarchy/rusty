//! `storedup` -- find packages that exist in the Nix store under more than one
//! version.
//!
//! Duplicate versions are the usual reason a store grows: one old generation,
//! one package override, or one flake input pinning its own nixpkgs is enough
//! to keep a second copy of something alive.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;

const STORE: &str = "/nix/store";

/// Every Nix store hash is 32 characters of base32.
const HASH_LEN: usize = 32;

/// One directory in the Nix store, split into its parts.
///
/// `s5g2mz...nn6z-rustup-1.29.1` becomes
/// `{ name: "rustup", version: Some("1.29.1"), output: None }`.
///
/// `derive` writes the boilerplate impls for us: `Debug` so `{:?}` and
/// `assert_eq!` can print one, `PartialEq`/`Eq` so two can be compared.
/// Put the cursor on `derive` and press <leader>ce to see what it generated.
#[derive(Debug, PartialEq, Eq)]
struct StorePath {
    name: String,
    version: Option<String>,
    /// A non-default output, as in `rustc-1.98.1-doc`.
    output: Option<String>,
}

impl StorePath {
    /// Split a store directory name into its parts.
    ///
    /// Returns `None` for anything that is not a store path -- `.links`, a
    /// `.drv` file, a stray file someone dropped in there. The caller decides
    /// what to do about that, which is the whole point of `Option`.
    fn parse(dir_name: &str) -> Option<Self> {
        // `split_once` returns Option<(&str, &str)>; `?` gives up and returns
        // None from this function if there is no '-' at all.
        let (hash, rest) = dir_name.split_once('-')?;
        if hash.len() != HASH_LEN {
            return None;
        }

        let parts: Vec<&str> = rest.split('-').collect();

        // The version is the first part starting with a digit. Everything
        // before it is the name, anything after it is an output like `doc`.
        // `let ... else` handles "there isn't one" without nesting.
        let Some(i) = parts.iter().position(|p| starts_with_digit(p)) else {
            return Some(Self::unversioned(rest));
        };
        if i == 0 {
            // Something like `<hash>-1.0`, with no name at all.
            return Some(Self::unversioned(rest));
        }

        Some(Self {
            name: parts[..i].join("-"),
            version: Some(parts[i].to_owned()),
            output: if i + 1 < parts.len() {
                Some(parts[i + 1..].join("-"))
            } else {
                None
            },
        })
    }

    /// An associated function used as a small constructor, to avoid writing the
    /// same three fields twice above.
    fn unversioned(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            version: None,
            output: None,
        }
    }
}

fn starts_with_digit(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_ascii_digit())
}

/// `storedup [N]` -- how many packages to list. Defaults to 20.
fn parse_limit() -> usize {
    std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(20)
}

/// Returning `io::Result` lets `?` propagate a failure out of `main`, which
/// then prints the error and exits non-zero. Try it: change STORE to a path
/// that does not exist.
fn main() -> io::Result<()> {
    let limit = parse_limit();

    // name -> the set of versions seen for it. BTreeMap and BTreeSet keep
    // their keys sorted, so the report comes out in order with no sort call.
    let mut versions: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut scanned: usize = 0;

    for entry in fs::read_dir(STORE)? {
        let entry = entry?;
        let file_name = entry.file_name();

        // A filename on disk is bytes, not guaranteed UTF-8, so this can fail.
        // Skipping is better than unwrapping and crashing on one odd path.
        let Some(name) = file_name.to_str() else {
            continue;
        };
        // `.ends_with(".drv")` is the obvious way and clippy::pedantic will
        // tell you off for it: it is case-sensitive, so a `.DRV` would slip
        // through. Path::extension is the comparison that cannot get that
        // wrong. Good first lesson in why pedantic is worth reading.
        if std::path::Path::new(name)
            .extension()
            .is_some_and(|ext| ext == "drv")
        {
            continue;
        }
        let Some(path) = StorePath::parse(name) else {
            continue;
        };

        scanned += 1;
        if let Some(version) = path.version {
            // `entry().or_default()` inserts an empty BTreeSet the first time
            // we see a name, then hands us &mut to it either way.
            versions.entry(path.name).or_default().insert(version);
        }
    }

    let mut dupes: Vec<(String, BTreeSet<String>)> = versions
        .into_iter()
        .filter(|(_, seen)| seen.len() > 1)
        .collect();

    // Most versions first. sort_by_key is stable, so names stay alphabetical
    // within one count -- that is why the BTreeMap above was worth using.
    dupes.sort_by_key(|(_, seen)| std::cmp::Reverse(seen.len()));

    let extra: usize = dupes.iter().map(|(_, seen)| seen.len() - 1).sum();
    let total = dupes.len();

    println!("Scanned {scanned} store paths in {STORE}.");
    println!("{total} packages have more than one version ({extra} extra copies).\n");

    for (name, seen) in dupes.iter().take(limit) {
        let count = seen.len();
        let list = seen
            .iter()
            .map(String::as_str)
            .collect::<Vec<&str>>()
            .join(", ");
        println!("  {name:<34} {count:>2}  {list}");
    }

    if total > limit {
        println!(
            "\n  ... and {} more. Pass a number to see them.",
            total - limit
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real 32-character store hash, so the length check passes.
    const H: &str = "s5g2mz2zr526hfvzf7si9ghsd221nn6z";

    fn parse(name: &str) -> StorePath {
        StorePath::parse(name).expect("should be a valid store path")
    }

    #[test]
    fn splits_name_and_version() {
        let p = parse(&format!("{H}-rustup-1.29.1"));
        assert_eq!(p.name, "rustup");
        assert_eq!(p.version.as_deref(), Some("1.29.1"));
        assert_eq!(p.output, None);
    }

    #[test]
    fn keeps_hyphenated_names_together() {
        let p = parse(&format!("{H}-rustc-wrapper-1.98.1"));
        assert_eq!(p.name, "rustc-wrapper");
        assert_eq!(p.version.as_deref(), Some("1.98.1"));
    }

    #[test]
    fn recognises_an_output_suffix() {
        let p = parse(&format!("{H}-rustc-1.98.1-doc"));
        assert_eq!(p.name, "rustc");
        assert_eq!(p.version.as_deref(), Some("1.98.1"));
        assert_eq!(p.output.as_deref(), Some("doc"));
    }

    #[test]
    fn versionless_paths_keep_their_whole_name() {
        let p = parse(&format!("{H}-rust-lib-src"));
        assert_eq!(p.name, "rust-lib-src");
        assert_eq!(p.version, None);
    }

    #[test]
    fn rejects_things_that_are_not_store_paths() {
        assert_eq!(StorePath::parse(".links"), None);
        assert_eq!(StorePath::parse("too-short-hash-1.0"), None);
    }
}
