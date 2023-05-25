FROM rust:latest as builder
# 要求 Rust 程序在 scratch 镜像下运行，使用 MUSL C 作为 C 语言标准库实现
RUN rustup default nightly && rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

WORKDIR /backend
COPY . ./

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch

WORKDIR /backend
COPY --from=builder /backend/target/x86_64-unknown-linux-musl/release/db-midpj ./

EXPOSE 8080
ENTRYPOINT ["/backend/db-midpj"]