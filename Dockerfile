FROM rust:1.88-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --workspace

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/api-server /app/api-server
EXPOSE 8080
CMD ["/app/api-server"]
