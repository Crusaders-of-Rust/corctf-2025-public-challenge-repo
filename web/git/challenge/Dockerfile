FROM --platform=amd64 debian:trixie AS builder
WORKDIR /tmp
RUN apt-get update && apt-get install -y gcc
RUN echo '#include <stdio.h>\nint main() { puts("corctf{h0me_r0ll3d_http_what_cou1d_g0_wr0ng?}"); }' > flag.c && \
	gcc -o flag flag.c && \
	chmod 111 flag
ADD https://fossil-scm.org/home/uv/fossil-linux-x64-202507091416.tar.gz fossil.tar.gz
RUN tar xf fossil.tar.gz

FROM --platform=amd64 debian:trixie
COPY --from=builder /tmp/flag /flag
COPY --from=builder /tmp/fossil /usr/bin/fossil
RUN useradd -ms /bin/bash admin
USER admin
WORKDIR /home/admin
COPY --chown=admin ctf.fossil .
CMD fossil server --https ctf.fossil