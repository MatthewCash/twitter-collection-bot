FROM rust:1.66 AS chef

RUN cargo install cargo-chef --version 0.1.51

WORKDIR app


FROM chef AS planner
COPY Cargo.* ./
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS build
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY Cargo.* ./
COPY src src
RUN cargo build --release --bin app


FROM gcr.io/distroless/cc
COPY --from=build /lib/x86_64-linux-gnu/libz.so.1 /lib/x86_64-linux-gnu/libz.so.1

COPY --from=build /app/target/release/app /usr/local/bin/

CMD ["/usr/local/bin/app"]
