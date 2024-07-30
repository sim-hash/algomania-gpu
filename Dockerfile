FROM nvidia/cuda:12.2.0-runtime-ubuntu22.04

ENV RUST_VERSION=1.74.1

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION}

ENV PATH="/root/.cargo/bin:${PATH}"

COPY . .

RUN ln -s /usr/local/cuda-12.2/targets/x86_64-linux/lib/libOpenCL.so.1 /usr/lib/libOpenCL.so
RUN cargo build --release

CMD ["./target/release/algomania-gpu"]