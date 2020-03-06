FROM rustlang/rust:nightly

WORKDIR /
RUN USER=root cargo new --bin rone
WORKDIR /rone

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN cargo build
RUN rm -rf src/*

COPY ./src ./src

RUN rm ./target/debug/deps/rone*
RUN rm ./target/release/deps/rone*
RUN cargo test

RUN cargo clippy

RUN cargo build --release
ENV SECRET_KEY=1
CMD ["./target/release/rone"]