FROM yanwk/comfyui-boot:cu129-slim

RUN pip install maturin && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y