#!/bin/bash

set -m

cleanup() {
    echo "Cleaning up..."
    pkill -P $$
    exit 0
}

trap cleanup EXIT

# Kill process group when server exits
(
    /server&
    server_pid=$!
    wait $server_pid
    echo "Server exited"
    # Give the client a second to restore the terminal
    sleep 5
    kill -TERM $$
) &

/bin/bash -i

exit 0
