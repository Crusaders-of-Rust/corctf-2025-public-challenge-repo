FROM ubuntu AS base

FROM pwn.red/jail

COPY --from=base / /srv

COPY chall /srv/app/run
COPY flag.txt /srv/app/flag.txt

RUN chmod +x /srv/app/run