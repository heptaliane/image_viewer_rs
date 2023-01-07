ARG RUST_VERSION=1.66.0

FROM rust:${RUST_VERSION}

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

RUN apt update
RUN apt install -y libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
