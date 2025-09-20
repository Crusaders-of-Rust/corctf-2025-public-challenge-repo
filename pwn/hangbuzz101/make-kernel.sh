#!/bin/bash

FLAG="corctf{s0ft_l0cKuP_n3t_sCh3d!}"
if [ -z "$FLAG" ]; then
    FLAG="corctf{test}"
fi

git clone git://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git hangbuzz
cp config hangbuzz/.config
sed -i "s/BUG: soft lockup/BUG: soft lockup, here is your flag: ${FLAG}/g" hangbuzz/kernel/watchdog.c
pushd hangbuzz
make olddefconfig
KBUILD_BUILD_TIMESTAMP='Sun August 10 12:00:00 UTC 2025' KBUILD_BUILD_USER=dependent KBUILD_BUILD_HOST=types make -j`nproc`
popd