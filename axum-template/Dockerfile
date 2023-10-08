FROM rust:1.72.0 as builder

# Create appuser
ENV USER=axum
ENV UID=1001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:latest

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

COPY --from=builder app/target/release/axum-template ./

#unprivileged user.
USER axum:axum

EXPOSE 8080
CMD ["./axum-template"]