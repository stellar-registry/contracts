set dotenv-load := true

export PATH := './target/bin:' + env_var('PATH')
export CONFIG_DIR := 'target/'
export CI_BUILD := env_var_or_default('CI_BUILD', '')

[private]
path:
    just --list

s +args:
    @stellar {{ args }}

stellar +args:
    @stellar {{ args }}

build_contract p:
    stellar contract build --profile contracts --package {{ p }}

# Build all contracts with the size-optimized profile
build:
    stellar contract build --profile contracts

# Setup git hooks and pin the stellar-cli version
setup:
    git config core.hooksPath .githooks
    -cargo binstall -y stellar-cli --version 26.0.0 --force --install-path ./target/bin

# Tests import compiled fixture wasm via `contractimport!`, so build first
test: build
    cargo t

clippy *args:
    cargo clippy --all {{ args }} \
    -- -Dclippy::pedantic -Aclippy::must_use_candidate -Aclippy::missing_errors_doc -Aclippy::missing_panics_doc

clippy-test:
    just clippy --tests --all-features
