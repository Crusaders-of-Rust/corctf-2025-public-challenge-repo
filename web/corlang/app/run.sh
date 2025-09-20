#!/bin/sh

docker build -t corlang-app .

docker run -p 8080:8080 corlang-app
