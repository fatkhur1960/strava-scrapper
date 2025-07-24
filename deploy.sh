#!/bin/bash
rsync -avzP --delete-after --exclude=target --exclude=asnrun-scrapper . root@202.10.45.80:/root/crawler
ssh root@202.10.45.80 "cd /root/crawler && /root/.cargo/bin/cargo build --release && cp target/release/asnrun-scrapper asnrun-scrapper && rm -rf src"
