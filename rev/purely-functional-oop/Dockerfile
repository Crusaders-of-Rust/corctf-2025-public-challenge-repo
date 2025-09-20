
FROM python:3.11-alpine AS python_base

RUN python3 -m pip install lark requests

FROM pwn.red/jail

COPY hook.sh /jail/hook.sh
RUN chmod +x /jail/hook.sh

COPY --from=python_base / /srv

RUN echo 'nameserver 1.1.1.1' > /srv/etc/resolv.conf
COPY compiler.py /srv/app/compiler.py
COPY formula_executor.py /srv/app/formula_executor.py
COPY server.py /srv/app/run
COPY submission_verifier.py /srv/app/submission_verifier.py
COPY flag.txt /srv/app/flag.txt

RUN chmod +x /srv/app/run

ENV JAIL_TIME=60 JAIL_MEM=40M JAIL_CPU=300