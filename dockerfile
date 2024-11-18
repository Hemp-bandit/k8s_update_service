FROM rust:latest as builder

WORKDIR /app
COPY src ./src
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN cargo build --release


# FROM debian:bullseye-slim
FROM rust:latest

COPY --from=builder /app/target/release/k8s_update_service /bin/k8s_update_service
EXPOSE 3001
CMD ["k8s_update_service"]