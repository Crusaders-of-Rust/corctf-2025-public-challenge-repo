FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    qemu-system-x86 \
    qemu-utils \
    libvirt-clients \
    libvirt-daemon-system \
    ovmf

WORKDIR /app

EXPOSE 8080

CMD [ "qemu-system-x86_64", "-m", "4G", "-cpu", "host,hv_relaxed,hv_vapic,hv_spinlocks=0x1fff,hv_time,hv_reset,hv_runtime,hv_synic,hv_stimer,hv_tlbflush,hv_ipi,hv_vpindex", "-enable-kvm", "-drive", "file=/app/windows.qcow2,if=ide", "-net", "nic", "-net", "user,hostfwd=tcp:0.0.0.0:8080-10.0.2.15:8080", "-nographic", "-serial", "none", "-monitor", "none", "-snapshot" ]
