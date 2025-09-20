.PHONY: run solve

build:
	cd program && anchor build
	cp program/target/deploy/corcoin.so challenge/
	cd server && cargo build --release
	cp server/target/release/server challenge/

run:
	cd challenge && ./server

solve:
	cd solve && python3 solve.py
