#!/bin/bash

IMAGE_NAME="msbug"
CONTAINER_NAME="msbug_container"

if docker ps -a --format '{{.Names}}' | grep -Eq "^${CONTAINER_NAME}\$"; then
    echo "[*] Removing existing container..."
    docker rm -f $CONTAINER_NAME
fi

echo "[*] Building Docker image..."
docker build -t $IMAGE_NAME .

echo "[*] Running Docker container..."
docker run  --name $CONTAINER_NAME -p 5000:5000 $IMAGE_NAME