FROM rust:1.78.0-slim-bullseye as builder
WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/ecr-middleware /
CMD ["./ecr-middleware"]
