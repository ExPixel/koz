#!/bin/sh

trap "echo 'received SIGINT' && exit 0" INT TERM
echo "waiting for exit"

sleep infinity &
wait $!