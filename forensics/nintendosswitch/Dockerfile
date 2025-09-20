########### Stage 1: build ########
FROM debian:bookworm AS build
RUN apt-get update && apt-get install -y --no-install-recommends build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY src/libnss_ctf.c .
RUN gcc -s -shared -fPIC -o libnss_ctf.so.2 libnss_ctf.c && \
    strip --strip-all libnss_ctf.so.2 && \
    objcopy --remove-section=.comment libnss_ctf.so.2 && \
    objcopy --rename-section .rodata=.junk libnss_ctf.so.2

########## runtime stage ##########
FROM debian:bookworm
COPY --from=build /build/libnss_ctf.so.2 /lib/x86_64-linux-gnu/
RUN mkdir -p /opt/ctf
COPY flag.blob /opt/ctf/flag.blob

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        strace ltrace binutils openssh-server && \
    rm -rf /var/lib/apt/lists/*

RUN chmod 0444 /opt/ctf/flag.blob && \
    printf 'passwd: files ctf\nshadow: files\n' > /etc/nsswitch.conf && \
    printf 'hosts:  files dns\n' >> /etc/nsswitch.conf && \
    rm -rf /var/lib/apt/lists/*

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

EXPOSE 5000
ENTRYPOINT [ "/usr/sbin/sshd", "-D", "-e", "-p", "5000" ]
