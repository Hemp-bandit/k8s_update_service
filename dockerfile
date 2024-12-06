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
RUN touch .env
EXPOSE 3000

CMD ["sh","run.sh"]