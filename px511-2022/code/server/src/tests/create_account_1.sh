#!/bin/sh

cmd=$(curl -k --location --request POST 'https://0.0.0.0:40443/create_account' --header 'Content-Type: application/json' --data-raw "{\"name\":\"moi\"}");
echo $cmd;