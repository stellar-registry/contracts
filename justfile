set dotenv-load := true

export PATH := './target/bin:' + env_var('PATH')
export CONFIG_DIR := 'target/'
export CI_BUILD := env_var_or_default('CI_BUILD', '')
# Stage built wasm under target/stellar/local/ — the path the registry tests'
# `contractimport!` and the manager's `import_contract_client!` resolve against.
export STELLAR_NETWORK := env_var_or_default('STELLAR_NETWORK', 'local')

[private]
path:
    just --list

s +args:
    @stellar {{ args }}

stellar +args:
    @stellar {{ args }}

build_contract p:
    stellar contract build --package {{ p }}

# Build all contracts with the size-optimized profile. Uses `stellar scaffold
# build` (not plain `stellar contract build`) so wasm is staged to
# target/stellar/<network>/, which the registry tests' `contractimport!` and
# registry-tansu-manager's `import_contract_client!(tansu_stub)` both resolve
# against. STELLAR_NETWORK defaults to `local` (see stellar-build), matching the
# `target/stellar/local/...` paths the tests import from.
build:
    stellar scaffold build

# Setup git hooks and pin the CLI versions
setup:
    git config core.hooksPath .githooks
    -cargo binstall -y stellar-cli --version 26.0.0 --force --install-path ./target/bin
    -cargo binstall -y stellar-scaffold-cli --version 0.0.24 --force --install-path ./target/bin

# Tests import compiled fixture wasm via `contractimport!`, so build first
test: build
    cargo t

clippy *args:
    cargo clippy --all {{ args }} \
    -- -Dclippy::pedantic -Aclippy::must_use_candidate -Aclippy::missing_errors_doc -Aclippy::missing_panics_doc

clippy-test:
    just clippy --tests --all-features

# Flux refinement verification of the registry contract (github.com/flux-rs/flux
# via the workspace flux-rs dep; toolchain setup: `just flux-setup` in
# nidohq/soroban-flux). Attributes are no-ops in normal builds. Overflow
# checking is per-item (#[opts(check_overflow = "strict")]) because the
# soroban-sdk-tools scerr macro generates arithmetic that can't be annotated.
flux:
    PATH="$HOME/.cargo/bin:$PATH" RUSTUP_TOOLCHAIN=nightly-2026-02-05 FLUXFLAGS="-Fpointer-width=32 -Fcache=target/flux-cache" cargo flux -p registry
