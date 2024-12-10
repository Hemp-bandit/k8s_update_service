FROM rust:latest as builder

WORKDIR /app
COPY src ./src
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN cargo build --release --target x86_64-unknown-linux-gnu
FROM debian:stable-slim

WORKDIR /app
COPY --from=builder /app/target/release .
COPY ./run.sh .
RUN touch .env ;
EXPOSE 3000

CMD ["sh","run.sh"]