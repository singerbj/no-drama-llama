#!/usr/bin/env bash
# Runs the same checks as the pull request CI (.github/workflows/ci.yml and site.yml).
# The pre-commit hook (.githooks/pre-commit) runs it; you can also run it by hand.
#
#   scripts/check.sh              everything; the docs site only if site/, design/, docs/ or CHANGELOG.md changed
#   scripts/check.sh --all        also build the docs site regardless
#   scripts/check.sh --staged     (the hook) decide about the docs site from the staged files
#
# Checks that need Windows (the Windows tests and the release exe) run only on Windows, as on CI.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

mode="${1:-}"
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) windows=1 ;;
  *) windows=0 ;;
esac

step() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }
fail() {
  printf '\n\033[1;31m%s\033[0m\n' "$*" >&2
  exit 1
}
need() { command -v "$1" >/dev/null 2>&1 || fail "$1 is missing. $2"; }

need npm "Install Node.js 22 or later."
need cargo "Install Rust from https://rustup.rs (rust-toolchain.toml picks the version)."

# ---------------------------------------------------------------- settings window (ui/)

step "UI: install dependencies"
(cd ui && npm ci --no-audit --no-fund --loglevel=error)

step "UI: audit, format, lint, typecheck, test, build"
(cd ui && npm run --silent check)

# ---------------------------------------------------------------- Rust

step "Rust: format"
cargo fmt --check

step "Rust: clippy (this OS)"
cargo clippy --all-targets -- -D warnings

step "Rust: clippy (Windows target)"
if [ "$windows" = 0 ] && ! command -v llvm-rc >/dev/null 2>&1; then
  fail "llvm-rc is missing: checking the Windows target from $(uname -s) needs it to build the exe's
resources. Install LLVM (Debian/Ubuntu: sudo apt-get install llvm; macOS: brew install llvm and
put its bin/ on PATH), or make llvm-rc point at llvm-rc-<version>."
fi
cargo clippy --all-targets --target x86_64-pc-windows-msvc -- -D warnings

step "Rust: tests"
cargo build --examples
PROPTEST_CASES=1000 cargo test --all-features -- --include-ignored

step "Rust: dependency advisories"
need cargo-deny "Install it with: cargo install cargo-deny --locked"
cargo deny check advisories

if [ "$windows" = 1 ]; then
  step "Windows: release build and smoke test"
  cargo build --release --locked
  exe=target/release/no-drama-llama.exe
  "$exe" version >/dev/null || fail "'version' failed"
  "$exe" --help >/dev/null || fail "'--help' failed"
  set +e
  "$exe" bogus-command >/dev/null 2>&1
  [ $? = 2 ] || fail "a bad command must exit with 2"
  set -e
fi

# ---------------------------------------------------------------- docs site (site/)

site_paths='^(site/|design/|docs/|CHANGELOG\.md$|\.github/workflows/site\.yml$)'
case "$mode" in
  --all) build_site=1 ;;
  --staged) git diff --cached --name-only | grep -Eq "$site_paths" && build_site=1 || build_site=0 ;;
  *) { git diff --name-only HEAD; git diff --name-only "$(git merge-base HEAD origin/main 2>/dev/null || echo HEAD)" HEAD; } |
    grep -Eq "$site_paths" && build_site=1 || build_site=0 ;;
esac
if [ "$build_site" = 1 ] && [ -d site ]; then
  step "Docs site: type-check and build"
  (cd site && npm ci --no-audit --no-fund --loglevel=error && npm run build)
fi

printf '\n\033[1;32mAll checks passed.\033[0m\n'
