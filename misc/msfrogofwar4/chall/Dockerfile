# build phase
FROM rust:slim AS builder

WORKDIR /app
COPY . .
WORKDIR /app/chess-web/
RUN cargo build --release

# final image
FROM debian:trixie-slim

WORKDIR /app
COPY --from=builder /app/chess-web/target/release/chess-web ./
COPY --from=builder /app/chess-web/static ./static
COPY --from=builder /app/chess-web/chess_engine_wasm.wasm ./chess_engine_wasm.wasm

RUN useradd -ms /bin/bash user
USER user

CMD ["/app/chess-web"]
