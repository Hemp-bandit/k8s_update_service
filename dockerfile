FROM rust:latest as builder

WORKDIR /app
COPY src ./src
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN cargo build --release --target x86_64-unknown-linux-gnu

FROM debian:stable-slim

COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/kaibai_user_service /bin/kaibai_user_service
COPY ./run.sh .
COPY sources.list /etc/apt/sources.list
RUN apt update && apt  install -y openssl

RUN touch .env ;

EXPOSE 3000
CMD ["sh","run.sh"]