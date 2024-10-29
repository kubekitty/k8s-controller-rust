FROM rust:1.82.0-slim-bullseye as builder

WORKDIR /app
COPY . .
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /app
RUN apt-get update && \
    apt-get install -y ca-certificates libssl1.1 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/k8s-controller-rust /app/
COPY rules /etc/kubekitty/rules
ENV RUST_LOG=info
EXPOSE 8080

ENTRYPOINT ["/app/k8s-controller-rust"]