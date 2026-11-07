# A dev shell for this folder, so ~/rust carries its own Rust rather than
# borrowing whatever modules/packages.nix happens to install today.
#
# Why bother when the system already has rustc?  Because this is the shape
# every real Rust project you write from here on will have, and it is worth
# being fluent in it before a project depends on it.  A dev shell pins the
# toolchain in flake.lock, so this folder keeps compiling with the same rustc
# in a year, and on another machine, and in CI -- none of which is true of a
# system profile that moves whenever you run `nh os switch`.
{
  description = "Sid's Rust learning workspace";

  # Pinned to the exact revision ~/nixos/flake.lock holds, not the floating
  # nixos-unstable branch.  Tracking the branch would resolve to whatever is
  # current the day this folder is first locked, which is almost never the
  # system's revision -- and a different revision means a second nixpkgs to
  # fetch and a second rustc/cargo closure in the store, for an identical
  # compiler.  Same rev, same store paths, nothing extra downloaded.
  #
  # Bump it by hand (then `nix flake update`) when ~/nixos moves and you want
  # this folder to follow.  A pinned rev stays portable: it resolves the same
  # way on any machine.
  inputs.nixpkgs.url = "github:nixos/nixpkgs/e7439b6b14ad3cc35d05608ebca9bce01a25f5f8";

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          rustc
          cargo
          clippy
          rustfmt
          rust-analyzer

          # Tools worth having in the shell but not in the system profile.
          bacon # live `cargo check`/clippy/test pane -- leave it running in a split
          cargo-expand # shows what a macro (incl. #[derive]) actually expanded to
          cargo-nextest # faster test runner with far more readable failure output

          # book-src/ is a real mdBook project, so the tool to build it belongs
          # in this folder's shell as well as the system profile -- otherwise
          # the folder only works on a machine whose ~/nixos matches.
          mdbook
        ];

        # nixpkgs' rustc has no lib/rustlib/src, so without this rust-analyzer
        # cannot step into the standard library and `impl Iterator for ...`
        # stays a black box.  Same variable home/common.nix sets globally;
        # repeated here so the shell stands on its own.
        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

        # Panics print a real stack trace instead of asking you to set this.
        RUST_BACKTRACE = "1";
      };
    };
}
