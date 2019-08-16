### What is ktnn ?

ktnn stands for "Komodo testnet notarization network", it is an unofficial notarization network used for testing purposes. Everyone is allowed to join

The first purpose of ktnn is to allow NN operators to test changes without testing the changes on a production notarization network. BTC testnet will be notarized with KMD but other coins and smartchains can be added/changed later
https://en.bitcoin.it/wiki/Testnet

As example, an enhancement of BTC coinsplit can be tested on ktnn without operating actual BTC. BTC testnet coins have no financial value

### How to setup ktnn ?

Source: this "How to setup ktnn ?" is mainly based on https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#_3rd-party-server-setup

- KMD Bootstrap:
https://dexstats.info/bootstrap.php

- tBTC faucet:
https://coinfaucet.eu/en/btc-testnet/
https://testnet-faucet.mempool.co/


## Warning
Please read the source if you are not familiar with Notary Nodes operations

## Requirements
TODO
- CPU: A CPU
- RAM: 8GB
- Disk: 500GB
- Bandwidth: 100Mbps

## Operating System
Debian/Ubuntu LTS x64 - minimal installation with Openssh server.

## Security

Please execute the steps in https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#security

## Install Komodo by compiling it from source

Please execute the steps in https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#install-komodo-by-compiling-it-from-source

## Generate one pubkey, address & WIF's

Please generate one pubkey, address & WIF's as described in https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#install-komodo-by-compiling-it-from-source

Use phm87's fork instead of Decker's original code, testnet BTC use different addresses:
```shell
https://github.com/phm87/komodo_scripts -b patch-3
```

Do not send the ktnn pubkey to kolo, send it to phm87 (or do a PR to ktnn branch of iguana)

## Setup Iguana

# Clone the jl777/SuperNET -b beta_season3 source
```shell
cd ~
git clone https://github.com/phm87/SuperNET/ -b ktnn
cd SuperNET/iguana
```

# Create a pubkey.txt file
You will need to create a pubkey.txt file inside ~/SuperNET/iguana directory. This will be used to start the dPoW process in the dpowassets script later on. pubkey.txt file should contain only the pubkey of the appropriate server. The file should contain only the information in the example below. Change 02a854251adfee222bede8396fed0756985d4ea905f72611740867c7a4ad6488c1 with the appropriate pubkey for the server the file is on.
```shell
pubkey=02a854251adfee222bede8396fed0756985d4ea905f72611740867c7a4ad6488c1
```

# Create wp_7787
The main server iguana will use the port 7787.

Create wp_7787 file inside the ~/SuperNET/iguana dir with the same passphrase you used to generate your Main address/pubkey. The file should look as follows (replace YOUR VERY SECURE PASSPHRASE with your own):
```shell
curl --url "http://127.0.0.1:7787" --data "{\"method\":\"walletpassphrase\",\"params\":[\"YOUR VERY SECURE PASSPHRASE\", 9999999]}"
```

# Make wp_7787 file executable
```shell
chmod 700 wp_7787
```

## Install bitcoin

Except for bitcoin.conf file, please execute the steps in https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#bitcoin

bitcoin.conf file content should contain testnet=1
```shell
testnet=1
server=1
daemon=1
txindex=1
rpcuser=bitcoinrpc
rpcpassword=password
bind=127.0.0.1
rpcbind=127.0.0.1
rpcallowip=127.0.0.1
```


## Start the daemons and sync all the chains

```shell
komodod &
bitcoind &
```
Please read the following link for more details: https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#start-the-daemons-and-sync-all-the-chains


## Import private key into KMD, BTC, 3P coins and assetcahins

Please use the tBTC keys instead of BTC (mainet) keys.

https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#import-your-main-private-key
https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#import-3rd-party-private-key


## Validate the addresses

https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#validate-the-addresses-2


## Copy the pubkey.txt file that we created from the ~/SuperNET/iguana dir

https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#copy-the-pubkey-txt-file-that-we-created-from-the-supernet-iguana-dir-2


## Stop All the Coin Daemons Safely
```shell
komodo-cli stop &
bitcoin-cli stop &
```
https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#stop-all-the-coin-daemons-safely


## Set ulimit parameters on Ubuntu permanently

https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#set-ulimit-parameters-on-ubuntu-permanently


## Edit the /etc/security/limits.conf file

https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#edit-the-etc-security-limits-conf-file


## Edit the /etc/pam.d/common-session file

https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#edit-the-etc-pam-d-common-session-file


## Check the values now

https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#check-the-values-now


## Create a start Script

https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#create-a-start-script

```shell
#!/bin/bash
source ~/komodo/src/pubkey.txt
bitcoind &
sleep 60
cd komodo/src
./komodod -pubkey=$pubkey &
```
Make the file executable:
```shell
chmod +x start
```

## Execute the start script

https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#execute-the-script


## Start iguana

https://docs.komodoplatform.com/notary/setup-Komodo-Notary-Node.html#start-iguana
```shell
cd ~/SuperNET/iguana
./m_notary_ktnn
```

Now, the calls contained in the dpowassets file are included in the m_notary_ktnn file.

TODO: dpowassets_ktnn separated


TODO: IP things, pubkeys ...

dig +short myip.opendns.com @resolver1.opendns.com
