FROM rust:1.75 as builder

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY bins ./bins

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/query-smith-web /usr/local/bin/
COPY --from=builder /build/target/release/query-smith /usr/local/bin/
COPY --from=builder /build/target/release/slack-bot /usr/local/bin/
COPY --from=builder /build/target/release/evals /usr/local/bin/
COPY --from=builder /build/target/release/context-enricher /usr/local/bin/

EXPOSE 3000 3001

CMD ["query-smith-web"]
