# Build in special rust container
FROM rust:latest as build

WORKDIR /app

COPY Cargo.toml ./
COPY Cargo.lock ./
COPY src ./src

RUN cargo build --release


# Transfer to debian container for production
FROM debian:buster-slim
RUN apt-get update && apt-get install -y openssl libssl-dev
WORKDIR /app

COPY --from=build /app/target/release/ .

# Set the entrypoint command for the container
CMD ["./Rustatoskr"]
