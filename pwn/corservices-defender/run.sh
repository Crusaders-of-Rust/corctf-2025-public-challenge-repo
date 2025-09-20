#!/bin/sh

docker build -t qemu-windows .

docker run --rm -it \
  --device /dev/kvm \
  -p 127.0.0.1:8080:8080 \
  -v ./windows.qcow2:/app/windows.qcow2 \
  qemu-windows
