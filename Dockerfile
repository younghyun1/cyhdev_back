ARG APP_NAME=cyhdev_com_back
FROM rust:alpine AS build
ARG APP_NAME
WORKDIR /app
RUN apk add --no-cache build-base ca-certificates openssl-dev upx

ENV RUSTFLAGS="-C target-cpu=native"

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/root/.cargo/registry/ \
    <<EOF
set -e
cargo build --release --locked
cp ./target/release/$APP_NAME /bin/server
upx --best --lzma /bin/server
EOF

FROM scratch
COPY --from=build /bin/server /server
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
EXPOSE 30737
CMD ["/server"]
