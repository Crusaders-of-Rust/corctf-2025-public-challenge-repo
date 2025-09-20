
FROM python:3.11-alpine AS python_base

FROM pwn.red/jail

COPY --from=python_base / /srv

COPY server.py /srv/app/run
COPY commitment.py /srv/app/commitment.py
COPY flag.txt /srv/app/flag.txt

RUN chmod +x /srv/app/run

ENV JAIL_TIME=300 JAIL_MEM=40M JAIL_CPU=300