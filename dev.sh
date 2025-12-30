#!/usr/bin/env bash

# Function to cleanup background processes
cleanup() {
    kill $(jobs -p) 2>/dev/null
}

trap cleanup EXIT

./build.sh

mailhog &
cd backend && cargo watch -x "run -- --dev"
