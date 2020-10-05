FROM rust:latest

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:10-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /root/
COPY --from=0 /usr/src/app/target/release/dino-park-lookout .
CMD ["./dino-park-lookout"]
