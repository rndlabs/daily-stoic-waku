FROM rust:1.67-bullseye as builder
WORKDIR /usr/src/myapp
COPY . .
RUN apt-get update && apt-get -y install golang clang
RUN cargo install --path .
 
FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/dailystoic /usr/local/bin/dailystoic
CMD ["dailystoic"]