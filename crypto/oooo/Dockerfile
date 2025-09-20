FROM python:slim-bullseye AS app

FROM pwn.red/jail
COPY --from=app / /srv
COPY server.py /srv/app/run
COPY flag.txt /srv/app/
ENV JAIL_TIME=300 JAIL_MEM=40M JAIL_CPU=1000