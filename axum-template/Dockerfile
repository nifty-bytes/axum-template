FROM rust:1.72.0 as builder
WORKDIR /usr/src/axum-template
COPY . .
RUN cargo install --path .
FROM rust:1.72.0-slim
COPY --from=builder /usr/local/cargo/bin/axum-template /usr/local/bin/axum-template
EXPOSE 8080
CMD ["axum-template"]
