#!/bin/bash

result=$(docker exec regtest_esplora /bin/cli $@)
echo $result | tr -d '\r'
