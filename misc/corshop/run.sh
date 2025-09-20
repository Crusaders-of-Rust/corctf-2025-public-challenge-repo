#! /bin/bash
docker build . -t cor-shop
docker run --rm -p 5000:5000 --privileged cor-shop
