FROM librespace/gnuradio:latest

# Required for libudev-sys
RUN apt-get update && apt-get install -y \
    libudev-dev \
    pkg-config \
    curl

# Install Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

RUN cargo build --release

CMD ["./target/release/ground-station"]
