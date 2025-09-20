FROM alpine

RUN apk add --no-cache qemu-system-x86_64

WORKDIR /opt
COPY bzImage initramfs.cpio.gz run.sh ./

CMD /bin/sh ./run.sh
