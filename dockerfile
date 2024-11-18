FROM rust:latest as builder

WORKDIR /app
COPY src ./src
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN cargo build --release

FROM debian:stable-slim

WORKDIR /app
COPY --from=builder /app/target/release/k8s_update_service . 
COPY ./run.sh .

EXPOSE 3001

CMD ["sh","run.sh"]