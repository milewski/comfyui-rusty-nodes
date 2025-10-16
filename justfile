publish:
    ./.venv/bin/comfy node publish
    rm node.zip

test:
    cargo test --no-default-features

build:
    maturin build --release --features extension-module