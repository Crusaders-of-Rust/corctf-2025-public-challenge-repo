#!/bin/sh
set -e

IMAGE_NAME="vouched"
CONTAINER_NAME="vouched"
PORT="8000"

docker build -t "$IMAGE_NAME" .

CONTAINER_ID=$(docker ps -aq -f name="$CONTAINER_NAME" 2>/dev/null)
if [ -n "$CONTAINER_ID" ]; then
  docker rm -f "$CONTAINER_NAME" >/dev/null 2>&1 || true
fi

docker run --name "$CONTAINER_NAME" --publish 8000:8000 "$IMAGE_NAME"