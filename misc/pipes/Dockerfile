FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY ./src .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update && apt-get install -y protobuf-compiler
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY ./src .
ARG DOCKER_BUILD=1
RUN cargo build --release

# Uncomment this section to build solver for testing
# ###############################################################
# FROM chef AS solve-planner
# COPY ./solver .
# RUN cargo chef prepare --recipe-path recipe.json

# FROM chef AS solve-builder
# RUN apt-get update && apt-get install -y protobuf-compiler
# COPY --from=planner /app/recipe.json recipe.json
# RUN cargo chef cook --release --recipe-path recipe.json
# COPY ./solver .
# ARG DOCKER_BUILD=1
# RUN cargo build --release
# ###############################################################

FROM ubuntu:latest AS base
RUN apt-get update && \
    apt-get install -y --no-install-recommends openssh-server && \
    apt-get clean

RUN groupadd -r server && \
    useradd -r -g server -m server && \
    groupadd -r ctf && \
    useradd -r -g ctf -m ctf && \
    echo 'ctf:ctf' | chpasswd && \
    mkdir -p /home/ctf/.ssh /var/run/sshd && \
    chown -R ctf:ctf /home/ctf/.ssh && \
    chmod 700 /home/ctf/.ssh

RUN <<EOF
echo "Match User ctf" >> /etc/ssh/sshd_config
echo "  PermitTTY yes" >> /etc/ssh/sshd_config
echo "  AllowTcpForwarding no" >> /etc/ssh/sshd_config
echo "  X11Forwarding no" >> /etc/ssh/sshd_config
EOF

COPY --from=builder --chown=server:server --chmod=6111 /app/target/release/server /app/target/release/client /
COPY --chown=server:server --chmod=0400 flag.txt /flag.txt
# Uncomment to upload solver for testing
# COPY --from=solve-builder /app/target/release/solver /

COPY run.sh /
EXPOSE 5000
ENTRYPOINT [ "/usr/sbin/sshd", "-D", "-e", "-p", "5000" ]
