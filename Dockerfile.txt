FROM rust:1.80 AS builder
WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY templates ./templates
COPY static ./static

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/webx-metrics-pro /usr/local/bin/webx-metrics-pro
COPY --from=builder /usr/src/app/templates ./templates
COPY --from=builder /usr/src/app/static ./static

USER nonroot
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/webx-metrics-pro"]
