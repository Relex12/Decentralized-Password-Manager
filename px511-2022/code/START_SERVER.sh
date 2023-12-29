#!/bin/sh


nb_line=$(ps -aux | grep  target/release/server | wc -l);

if [ $nb_line -eq 1 ]; then
    $(RUST_LOG=info cargo r -p server -- --address 0.0.0.0 --port 40443 --cert ./server/cert.pem --key ./server/key.pem )
fi

