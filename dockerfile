FROM rust:latest as builder

WORKDIR /app
COPY src ./src
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN cargo build --release

FROM debian:stable-slim

WORKDIR /app
COPY --from=builder /app/target/release/kaibai_user_service . 
COPY ./run.sh .
# COPY .env .
EXPOSE 3000

CMD ["./kaibai_user_service"]