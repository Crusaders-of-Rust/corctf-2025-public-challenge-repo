# ---- cargo-chef: plan -------------------------------------------------------
FROM rust:1.89-bullseye AS chef
RUN cargo install cargo-chef
WORKDIR /app

COPY corchat .
WORKDIR /app/corchat
RUN cargo chef prepare --recipe-path recipe.json


# ---- cargo-chef: cook dependencies -----------------------------------------
FROM rust:1.89-bullseye AS cacher
RUN cargo install cargo-chef
WORKDIR /app/corchat

COPY --from=chef /app/corchat/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json


# ---- builder: compile the actual binaries -----------------------------------
FROM rust:1.89-bullseye AS builder
WORKDIR /app

COPY corchat .
WORKDIR /app/corchat

COPY --from=cacher /app/target /app/target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

RUN cargo build --release --bins

# ---- runtime: minimal final image -------------------------------------------
FROM debian:bullseye-slim AS runtime
ARG USER=ctf
ARG UID=1000
WORKDIR /home/${USER}

RUN apt-get update && apt-get install -y --no-install-recommends \
    socat \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u ${UID} -s /bin/bash ${USER}
COPY flag.txt /root/flag.txt
COPY messages.txt /root/messages.txt

COPY --from=builder /app/target/release/corchat /usr/local/bin/corchat
COPY --from=builder /app/target/release/bots    /usr/local/bin/bots

RUN chown root:root /usr/local/bin/corchat \
 && chmod 4755      /usr/local/bin/corchat

RUN chown root:root       /usr/local/bin/bots \
 && chmod 0755            /usr/local/bin/bots

RUN printf '%s\n' \
  '#!/bin/sh' \
  'set -eu' \
  '/usr/local/bin/bots /root/messages.txt &' \
  'su ctf -c "exec socat -d -d TCP-LISTEN:5000,reuseaddr,fork EXEC:\"/bin/sh\",pty,stderr,setsid,sigint,sane"' \
  > /entrypoint.sh \
 && chmod +x /entrypoint.sh

EXPOSE 5000
ENTRYPOINT ["/entrypoint.sh"]
