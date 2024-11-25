#!/bin/bash

git pull
cargo update
cargo build --release
sudo killall government-ai-project-backend
sudo /home/cyh/cyhdev_back/target/release/cyhdev_back >> ./logs/$(date '+%Y-%m-%d_%H-%M-%S').log 2>&1 &
if [ $? -eq 0 ]; then
    echo "Server has successfully started"
else
    echo "Failed to start the server"
fi
