#!/bin/bash

docker run --name regtest_esplora \
           -p 50001:50001 -p 8094:80 \
           --volume $PWD/data_bitcoin_regtest:/data \
           --rm -i -t blockstream/esplora \
           bash -c "/srv/explorer/run.sh bitcoin-regtest explorer"
