#!/bin/bash

while true; do
    read -p "Are you sure you want to reset the database? [y/n] " yn
    case $yn in
        [Yy]* ) break;;
        [Nn]* ) exit;;
        * ) echo "Please answer yes or no.";;
    esac
done

set -e
set -x

docker compose down -v
docker compose up -d
cargo run -p migrate