#!/bin/bash
pushd /root/crawler 2>/dev/null
JOBS=$(nproc --all)

echo "Running Scrapper with $JOBS"

source /root/crawler/.env
nohup ./asnrun-scrapper --jobs=$JOBS > output.log 2>&1 &
popd
