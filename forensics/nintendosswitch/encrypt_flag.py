#!/usr/bin/env python3
import secrets, struct, argparse, pathlib, textwrap

def xorshift32(x: int) -> int:
    x ^= (x << 13) & 0xFFFFFFFF
    x ^= (x >> 17) & 0xFFFFFFFF
    x ^= (x << 5)  & 0xFFFFFFFF
    return x & 0xFFFFFFFF

def keystream(seed: int):
    state = seed & 0xFFFFFFFF
    while True:
        state = xorshift32(state)
        yield state & 0xFF   # one byte at a time

def main():
    ap = argparse.ArgumentParser(
        description="Encrypt flag with xorshift32 stream cipher")
    ap.add_argument("flag", help="flag text or @file")
    ap.add_argument("out",  help="output blob file")
    args = ap.parse_args()

    # load plaintext
    flag = (
        pathlib.Path(args.flag[1:]).read_text().strip()
        if args.flag.startswith("@") else args.flag
    ).encode()

    seed = struct.unpack("<I", secrets.token_bytes(4))[0]
    ks   = keystream(seed)

    ct   = bytes(b ^ next(ks) for b in flag)
    pathlib.Path(args.out).write_bytes(ct)

    print(f"Seed: 0x{seed:08x}")
    print(f"Blob written: {args.out} ({len(ct)} bytes)")
    print("Put the seed in libnss_ctf.c and COPY the blob in the Dockerfile.")

if __name__ == "__main__":
    main()
