> Life is long, if you know how to use it

# Daily stoic

This repository is an example highlighting the use of some novel technologies:

1. Waku decentralized messaging in `rust` (using `rust-waku-bindings`).
2. `bash` scripting to interact with Waku as a JSON-RPC client.

## Architecture

This makes uses of `WAKU2-RELAY`, with the following specifications:

1. PubSub topic set to `/waku/2/default-waku/proto`
2. Content topics `/dailystoic/1/request/proto` and `/dailystoic/1/broadcast/proto`.

User flow:

1. Subscribe to a message that is delivered every 24 hrs (monitor the `PubSub` for the content topic `/dailystoic/1/broadcast/proto`).
2. Ask for a daily stoic (`Request`) via `/dailystoic/1/request/proto`.

## `dailystoic`

This is a binary, written in `rust`. Usage:

```bash
dailystoic /quotes/quotes.json
```

The first and only argument being passed is the file path to a JSON file containing the quotes.

## Bash client

When running your own `nwaku` node, you can use the JSON-RPC client to interact with waku. For this, the following are required:

* bash
* protoc
* jq
* xxd
* Running nwaku ndoe

To use the bash client to get a daily stoic quote using your local `nwaku` node:

```bash
cd bash_client
./client.sh
```
