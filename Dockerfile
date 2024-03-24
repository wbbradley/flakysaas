FROM rust:1-bookworm

COPY . /opt/flakysaas
WORKDIR /opt/flakysaas
RUN cargo build --profile=release
EXPOSE 9001:9001
ENTRYPOINT /opt/flakysaas/target/release/flakysaas
