
FROM python:3.11-alpine AS python_base

RUN python3 -m pip install numpy

FROM pwn.red/jail

COPY --from=python_base / /srv

COPY server.py /srv/app/run
COPY flag.txt /srv/app/flag.txt

RUN chmod +x /srv/app/run

ENV JAIL_TIME=60 JAIL_MEM=40M JAIL_CPU=300 JAIL_POW=5000 JAIL_PIDS=0
