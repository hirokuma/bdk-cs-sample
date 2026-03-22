# bdk-cs-example

## original

[book-of-bdk](https://github.com/bitcoindevkit/book-of-bdk/blob/474aa61ecd444833354e4e1177000d90362f1471/examples/rust/syncing/electrum/src/main.rs)

## Bitcoin Regtest backend

* [How to run the explorer for Bitcoin regtest](https://github.com/Blockstream/esplora/tree/b380f086cb23e1b69ace201794bc8db26ed71c96#how-to-run-the-explorer-for-bitcoin-regtest)

```bash
./docker/run-backend.sh
```

## Generate to address

```bash
./docker/bcli.sh generatetoaddress <ADDR> 1
```

## Run

```bash
cargo run
```

### example

Run this program.

```shell
$ cargo run
Creating new wallet database.
Generated address bcrt1pg5j7xlkvwej9etlatp87as98xg7k0lr3wq4kzdz50fsd3gr6mrds54gedd at index 0
Wallet balance: 0 sat
```

Generate to address.

```shell
$ ./docker/bcli.sh generatetoaddress 1 bcrt1pg5j7xlkvwej9etlatp87as98xg7k0lr3wq4kzdz50fsd3gr6mrds54gedd
[ "44db1e84fd0f1dc23953d90ef150d6a44ee29168adddf723e79b534616ad6a22" ]
```

Wait...

```log
Wallet balance changed: 0 --> 1250000000 sat
```

## Remove wallet data

```bash
rm wallet-data/wallet.*
```
