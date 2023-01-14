FROM rust:1.64 AS build

WORKDIR /app

COPY Cargo.* ./
COPY src src

RUN cargo build --release

FROM gcr.io/distroless/cc

WORKDIR /app
COPY --from=build /app/target/release/twitter-collection-bot ./

CMD ["/app/twitter-collection-bot"]
