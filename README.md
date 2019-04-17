# How to partcipate

Make a Pull Request with your IP Address added to `iguana/m_notary_testnet` and your public key and name added to `iguana/testnet.json`.

# Install and start the notary

#### * Clone this repo
#### * Install dependencies
```shell
sudo apt install build-essential pkg-config libc6-dev m4 \
        g++-multilib autoconf libtool ncurses-dev unzip git python \
        zlib1g-dev wget bsdmainutils automake libboost-all-dev \
        libssl-dev libprotobuf-dev protobuf-compiler \
        libqrencode-dev libdb++-dev ntp ntpdate vim software-properties-common \
        curl libevent-dev libcurl4-gnutls-dev libsodium-dev cmake clang
```
#### * Install nanomsg from https://github.com/nanomsg/nanomsg
#### * Clone and build Komodo from https://github.com/jl777/komodo --branch dev
#### * Create ~/.komodo.conf
```
rpcuser=<secure username>
rpcpassword=<secure password>
txindex=1
server=1
daemon=1
rpcworkqueue=256
```
#### * Start KMD daemon
#### * Export keys
#### * Stop KMD daemon
#### * Start KMD daemon with `-pubkey=<your public key>`
#### * Start BEER, PIZZA and TXSCLCC with `-pubkey=<your public key>`
```shell
komodod -ac_name=BEER -ac_supply=100000000 -addnode=78.47.196.146 -pubkey=<your public key>
```
```shell
komodod -ac_name=PIZZA -ac_supply=100000000 -addnode=78.47.196.146 -pubkey=<your public key>
```
```shell
komodod -ac_name=TXSCLCC -ac_supply=0 -ac_reward=2500000000 -ac_halving=210000 -ac_cc=2 -addressindex=1 -spentindex=1 -pubkey=<your public key> -addnode=54.36.126.42 -addnode=94.130.224.11
```
#### * Open ports required for p2p.

| Coin          | Port          |
| ------------- |-------------: |
| KMD           | 7770          |
| BEER          | 8922          |
| PIZZA         | 11607         |
| TXSCLCC       | 51797         |
| Iguana        | 17711         |

#### * Fund public key address on all 3 nodes.
#### * Import your private key to all 3 nodes.
```shell
komodo-cli -ac_name=<coin name> importprivkey <your private key>
```
*If you need PIZZA or BEER, use the faucets at https://www.atomicexplorer.com/#/faucet/ or ask in #notarynode channel.*

For TXSCLCC chain, mine 1 block using 1 CPU thread.
```shell
komodo-cli -ac_name=TXSCLCC setgenerate true 1 #start mining
```
```shell
komodo-cli -ac_name=TXSCLCC setgenerate false #stop mining
```
#### * Create `~/2019NNtestnet/iguana/pubkey.txt`
```
pubkey=<your public key>
```
#### * Create `~/2019NNtestnet/iguana/passphrase.txt`
```
passphrase=<your private key>
```
#### * Wait until the PR is merged.
#### * Start/restart notorization with Iguana
*This script must be run from within the `2019NNtestnet/iguana` directory.*
```shell
cd ~/2019NNtestnet/iguana
./m_notary_testnet
```
