# Build in special rust container
FROM rust:latest as build

WORKDIR /app

COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release


# Transfer to debian container for production
FROM alpine:latest
RUN apk update && apk add --no-cache openssl
WORKDIR /app

ARG TELOXIDE_TOKEN
ARG TELOXIDE_PROXY

COPY --from=build /app/target/release/ .

# Set the entrypoint command for the container
CMD ["./Rustatoskr"]
