ARG RUST_VERSION=1.66.0

FROM rust:${RUST_VERSION}

ARG BUILD_USER=builder
ARG UID=1000
ARG GID=1000

RUN groupadd -g ${GID} builder
RUN useradd -m -u ${UID} -g ${GID} ${BUILD_USER}

RUN apt update
RUN apt install -y libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

USER ${BUILD_USER}
WORKDIR /home/${BUILD_USER}/

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk
RUN cargo install wasm-bindgen-cli
RUN cargo install tauri-cli
