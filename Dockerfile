FROM rust as build

ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/cloudconsole

COPY . .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian10

COPY --from=build /usr/local/cargo/bin/cloudconsole /usr/local/bin/cloudconsole

WORKDIR /app

COPY ./config /app/config

CMD ["cloudconsole"]