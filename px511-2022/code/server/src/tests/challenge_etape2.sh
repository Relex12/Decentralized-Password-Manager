#!/bin/sh

cmd=$(curl -k --location --request POST 'https://0.0.0.0:40443/create_account' --header 'Content-Type: application/json' --data-raw "{\"public_key\":\"1\", \"code\":5000}");
echo $cmd;