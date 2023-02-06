FROM rust:1.67-bullseye as builder
WORKDIR /usr/src/myapp
COPY . .
RUN apt-get update && \
    apt-get -y install clang wget && \
    wget https://go.dev/dl/go1.18.10.linux-amd64.tar.gz \
    && rm -rf /usr/local/go && tar -C /usr/local -xzf go1.18.10.linux-amd64.tar.gz
ENV PATH="$PATH:/usr/local/go/bin"
RUN cargo install --path .
 
FROM debian:bullseye-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/dailystoic /usr/local/bin/dailystoic
CMD ["dailystoic", "/quotes/quotes.json"]