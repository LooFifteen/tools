# builder image
FROM rustlang/rust:nightly-alpine AS builder

COPY Cargo.toml Cargo.lock /app/
COPY src /app/src
WORKDIR /app

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

RUN cargo build --release

# runner image
FROM alpine:latest

COPY --from=builder /app/target/release/tools /bin/tools
EXPOSE 3000

CMD [ "/bin/tools" ]