from solders.instruction import AccountMeta
from solders.keypair import Keypair
from solders.pubkey import Pubkey
from pwn import *
import json
import os

# context.log_level = "debug"

os.system("cargo build-sbf")

with open("target/deploy/solve.so", "rb") as f:
    program = f.read()

with open("target/deploy/solve-keypair.json", "r") as f:
    privkey = json.loads(f.read())
    keypair = Keypair.from_bytes(bytes(privkey))

print("")
print("loaded keypair with public key:", keypair.pubkey())

r = remote(args.HOST or "localhost", args.PORT or 5000)

print(r.recvuntil(b"program pubkey: ").decode())
r.sendline(str(keypair.pubkey()).encode())

print(r.recvuntil(b"program len: ").decode())
r.sendline(str(len(program)).encode())

r.send(program)

print(r.recvuntil(b"corcoin: ").decode())
corcoin_program_pubkey = Pubkey.from_string(r.recvline().strip().decode())

print(r.recvuntil(b"player: ").decode())
player_program_pubkey = Pubkey.from_string(r.recvline().strip().decode())

print(r.recvuntil(b"Player account created: ").decode())
player_pubkey = Pubkey.from_string(r.recvline().strip().decode())

print(r.recvuntil(b"Validator: ").decode())
validator_pubkey = Pubkey.from_string(r.recvline().strip().decode())

print(r.recvuntil(b"Vote Account: ").decode())
vote_account_pubkey = Pubkey.from_string(r.recvline().strip().decode())

print("accounts:")
print("corcoin program:", corcoin_program_pubkey)
print("player program:", player_program_pubkey)
print("player:", player_pubkey)
print("validator:", validator_pubkey)
print("vote account:", vote_account_pubkey)

def run_program(accounts: list[AccountMeta] = [], data: bytearray = b""):
    r.recvuntil(b"Choose an option [1-5]:", timeout=10)
    r.sendline(b"1")
    r.recvuntil(b"num accounts: \n", timeout=10);
    r.sendline(str(len(accounts)).encode())

    for account in accounts:
        meta = "z"
        if account.is_signer:
            meta += "s"
        if account.is_writable:
            meta += "w"
        account_str = f"{meta} {account.pubkey}"
        r.sendline(account_str.encode())

    r.recvuntil(b"ix len: \n", timeout=10)
    r.sendline(str(len(data)).encode())
    r.send(data)

def advance_clock():
    r.recvuntil(b"Choose an option [1-5]:", timeout=10)
    r.sendline(b"2")

def get_flag():
    r.recvuntil(b"Choose an option [1-5]:", timeout=10)
    r.sendline(b"3")
    r.interactive()

def check_lamports():
    r.recvuntil(b"Choose an option [1-5]:", timeout=10)
    r.sendline(b"4")
    r.recvuntil(b"Player lamports: ", timeout=10)
    lamports = int(r.recvline().strip().decode())
    return lamports

r.interactive()