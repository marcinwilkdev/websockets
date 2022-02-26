FROM rust:1.58

COPY ./ ./

RUN cargo build --release

CMD ["./target/release/websockets"]
