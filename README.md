# Felidae Network Node

This repo contains the code for the felidae network blockchain.

## Node Setup

This node has been tested for Ubuntu version 20.04.

#### Install dependencies

Use a terminal shell to execute the following commands:

```bash
sudo apt update
# May prompt for location information
sudo apt install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev curl
```
#### Rust Developer Environment

This project uses [`rustup`](https://rustup.rs/) to help manage the Rust toolchain. First install
and configure `rustup`:

```bash
# Install
curl https://sh.rustup.rs -sSf | sh
# Configure
source ~/.cargo/env
```

Finally, configure the Rust toolchain:

```bash
rustup default stable
rustup update nightly
rustup update stable
rustup target add wasm32-unknown-unknown --toolchain nightly
```

### Run

Use Rust's native `cargo` command to build and launch the template node:

```sh
cargo run --release -- --dev --tmp
```

### Build

The `cargo run` command will perform an initial build. Use the following command to build the node
without launching it:

```sh
cargo build --release
```

### Embedded Docs

Once the project has been built, the following command can be used to explore all parameters and
subcommands:

```sh
./target/release/felidae-node -h
```

## Run

The provided `cargo run` command will launch a temporary node and its state will be discarded after
you terminate the process. After the project has been built, there are other ways to launch the
node.

### Single-Node Development Chain

This command will start the single-node development chain with persistent state:

```bash
./target/release/felidae-node --dev
```

Purge the development chain's state:

```bash
./target/release/felidae-node purge-chain --dev
```

Start the development chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/felidae-node -lruntime=debug --dev
```

### Node

A blockchain node is an application that allows users to participate in a blockchain network.

After the node has been [built](#build), refer to the embedded documentation to learn more about the
capabilities and configuration parameters that it exposes:

```shell
./target/release/felidae --help
```

### Multi-Node Testnet

#### Generate node key

Generate pubic key from a secret phrase. This secret phrase is dedicated to development and should not be used in any other places.

```bash
./target/release/felidae-node key inspect --scheme ed25519 "spirit scan alone brave volume gallery put order index before eager update"
```
```bash
Secret phrase:       spirit scan alone brave volume gallery put order index before eager update
  Network ID:        substrate
  Secret seed:       0xcad0be1eb91ac5000b0f2b163430cdcc601bcfa65be61703b2376381a3387eda
  Public key (hex):  0x4cc79714885dced625d7fbf8341008ae4a1fb714452a62c50456597f29ecc270
  Account ID:        0x4cc79714885dced625d7fbf8341008ae4a1fb714452a62c50456597f29ecc270
  Public key (SS58): 5DoNnbvKvthQZ2YNzjtkU1rbQmMvo8zQAW4wK2FPR4U6H5ZK
  SS58 Address:      5DoNnbvKvthQZ2YNzjtkU1rbQmMvo8zQAW4wK2FPR4U6H5ZK
```

Save this public key
0x4cc79714885dced625d7fbf8341008ae4a1fb714452a62c50456597f29ecc270

#### Config validators' session keys

Config validators secret phrase in testnet.sh.

```
# Copy paste your mnemonic here.
SECRET="universe toast because trouble bulb inmate cruel shock erode border hour entry"
```

This command will generate 4 validator session keys.

```bash
./scripts/testnet.sh 4
```

```
(
// 5Ec15LH7q8HJqpCpCVMpFjeh2Yc45FQRxn5oiLz4LSVTL14u
array_bytes::hex_n_into_unchecked("7057095d75c7bd79e0fae628781857897685a56f277cf5ce74d138419cbddd1c"),
// 5HTVKiqguCHz5kToLBantKnrN3D8qQ18jY3RxW8NtsW3b4dJ
array_bytes::hex_n_into_unchecked("ee8d9ead9773a108321fd28f24b8983b11edf197eb44b66061fb2c8b0a82fe51"),
// 5CuaF5jF64gdt3Vt49hT8TChHx4AoAESD92Sz3xnJnmBwPfM
array_bytes::hex2array_unchecked("25456531bab5a05e99c31e71f44e634ceca3abdc470060b2cc5f4bb24ce91aa6").unchecked_into(),
// 5GCT4MoLs73naKmnZdjqKdjzhu3BSaCBXqCMzuyP9JSfQjCQ
array_bytes::hex2array_unchecked("b6d914c0e0867c0f30be1b2bbb9adb12b2e9ce1fca787bd82378786dc9e2f65e").unchecked_into(),
// 5Cfq4BxgF7j1xYQhcNNjdggeoyETJ2pBNKyi24vVrqyrUZPx
array_bytes::hex2array_unchecked("1ac9d269fd7055ad74d00a5bedc2e1073b1b3c69a06664f48d02c523b2c29d65").unchecked_into(),
// 5HEx8oBqKoYyQ2zfCe1v4yww3dcnTQT2kNzhWM92JrPS54Mo
array_bytes::hex2array_unchecked("e4fdaf83334c46d90be17c5be7374f28bf450bbfdad2cd48275e038c824e384d").unchecked_into(),
),
(
// 5EeG3Lv2uL6Y5QbnhwhAbA85H8xNM7BFAWjHPqL2BMzjhLuK
array_bytes::hex_n_into_unchecked("720fe80d17b62d6f649ab8d7e425c06817b54f2f38c4268533909b5512a4fb1f"),
// 5DXvJrbKvSjYMXb29C4sufgYNZPf79B2EvBKytGnhBiaAQQW
array_bytes::hex_n_into_unchecked("40fdc65d56b3c2c0ee0a3b3a678d5bd91f21b6f1b05fa9832075aa7ce1326f3a"),
// 5GQerxpNohj7vS4Rzpdh8jzGhwehCKaCKDJpuc7m3kUKGa8P
array_bytes::hex2array_unchecked("c027c2951314e08c5a87360bc8ff675e41696029f746ddf5408468a38fce1f71").unchecked_into(),
// 5EsVEFtr3fPhk9JU9B9UrBPuR5HPtVb73RArQC8sWSBJJMG5
array_bytes::hex2array_unchecked("7c267df50215050f51b2dc8daf8c8862add3eec8c3c7bd34c7416f142bcc602d").unchecked_into(),
// 5CajMWJaVpZ71JoquPwb2Q9X7N4yEF6qzhciYdu2gfDVJfZX
array_bytes::hex2array_unchecked("16e666f279051a7b78488427e0cf6e2ecdd12c1a6fa39cafd187159bece45b5e").unchecked_into(),
// 5FsCVP9ykMp8LHjCsuVeJQijH3s2mvGhc4rhN1Hia7MeYygg
array_bytes::hex2array_unchecked("a82a6050c0e374451a1b7f25aff82f12e0198cd97daaecab7be804c0dceca833").unchecked_into(),
),
(
// 5Djy3YvvhWCCkab4ePFESoFt6tBDDaxRGUWnKPjHTuGP2MPw
array_bytes::hex_n_into_unchecked("4a2dee2b1c17eaf3a80fb4068fd24ed56a054f71831d8e504c9af0c2f35af534"),
// 5GCUF8WSuHf74x9SStxsTX86uMgmoSzTepKzpUDYERWLehD8
array_bytes::hex_n_into_unchecked("b6dd128f9ec50bd42026939e53fafd517957ef0ad778d67b4afc93682abd7766"),
// 5DGoToXx45yvFtBGo6cPwaxiBLuNhF7wrEnfFHqW7JHZZrNw
array_bytes::hex2array_unchecked("35760c8ac3aa85df80544b20347f782927f0db9f13ccf4001013c89199a352f4").unchecked_into(),
// 5Ec2PR8EwMuaPRo4oAQVDwZW9Wh9bdR8LGN5ugZy6iU6Rk7W
array_bytes::hex2array_unchecked("705b73d388629bf6591d96285a527f0fee36d4c66955ccd73b1e3cd20fb20002").unchecked_into(),
// 5EqZCSPB6FoeRHHYu2WpUT88LFNAHhEQfNBQRfNMUP3RhUDs
array_bytes::hex2array_unchecked("7aad5c5f332396096f05afc304350b5d13ca262fa3645129cee460173acd1827").unchecked_into(),
// 5FWLzFus2rDETubyD7EvSSuct3kfFpK5MHbENAaHf7qzv8z3
array_bytes::hex2array_unchecked("9842d43c280fb08b5d7b8d84683162660fd40b69389acb48ab7da4af2554a26a").unchecked_into(),
),
(
// 5GmbfJoWYwbnPqiCspdVXAVnxUafA6h51ixnJLcTVeSHEsRp
array_bytes::hex_n_into_unchecked("d02123612afeaeb0f5da1370ec989c34e3f4cffe15635ba8edd90094cd6bf229"),
// 5CA3vR9ZoBBxXdMLcMMZT1Mh7jUQf3GcpU1hN1p8rBLg3MNe
array_bytes::hex_n_into_unchecked("0413c544b1303301f9fc622609750daadc29e4a5346c3fb73983274c5f87cf21"),
// 5CoZje3pPHzsEJ6CxDSFtBhE9UFuV7BYfoVoEu7bKsLTXdYw
array_bytes::hex2array_unchecked("20b036c9c9a85516ce6a2314a02bcffa1d28e7b50a4bc05aef8a85285f2f318b").unchecked_into(),
// 5FRc2RDMwRduirLcteT9t2yqew9jaH7rqPV1i3VKBWChtszD
array_bytes::hex2array_unchecked("94a538cf115ae998fb1c43c364b0f66b7d9d7cff6eb0e78b0144be031c46a91b").unchecked_into(),
// 5HmqZeWbir1sR9i4f2zvBUG65b5iiz5fz8wBoAexVedTRAHW
array_bytes::hex2array_unchecked("fc8c2bdce66b710132d0cdac7ce83bf8adf6ae2d5bc9dddcd237641b48c3555a").unchecked_into(),
// 5CJJZtihoPJ7cyYPGqpSsijcRGb27qWMv7oLbe2bRWT4HYis
array_bytes::hex2array_unchecked("0a5f097d9b0f5eba22528ecfe85fee43a2225a3d789ca68c98163f0346180235").unchecked_into(),
),

```

Replace the session keys from vec![...] in node/cli/src/chain_spec.rs with above generated keys.

```rust
fn staging_testnet_config_genesis() -> GenesisConfig {
    let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![...];
    #--snip--
}
```

Recompile
```
cargo build --release
```

#### Config root key

Generate pubic key from a secret phrase. This secret phrase is dedicated to development and should not be used in any other places.

```bash
./target/release/felidae-node key inspect --scheme sr25519 "fuel rigid basic host nephew deer morning flavor car chase iron silly"
````

```
Secret phrase:       fuel rigid basic host nephew deer morning flavor car chase iron silly
  Network ID:        substrate
  Secret seed:       0xe5df8a945ad840e7b46b8053dc59213b5e8f717f04d2a636756c98be2bab7ec5
  Public key (hex):  0xceefaffb49b14c577806cb62761ae002d82d06a965988ee606eccae9977f5a16
  Account ID:        0xceefaffb49b14c577806cb62761ae002d82d06a965988ee606eccae9977f5a16
  Public key (SS58): 5Gk2vWjUq1Z75HVhkKKQpEuvrZwnXYG2GAKyS6hU428yuv9K
  SS58 Address:      5Gk2vWjUq1Z75HVhkKKQpEuvrZwnXYG2GAKyS6hU428yuv9K

```

Remove prefix 0x of public key, replace the root key in `node/cli/src/chain_spec.rs`

```rust
let root_key: AccountId = array_bytes::hex_n_into_unchecked(
		// 5Gk2vWjUq1Z75HVhkKKQpEuvrZwnXYG2GAKyS6hU428yuv9K
		"ceefaffb49b14c577806cb62761ae002d82d06a965988ee606eccae9977f5a16",
	);
```

#### Launch first node
Launch first node with the public key above.

```bash
./target/release/felidae-node --chain staging -d data/validator1 --name validator1 --in-peers 256 --validator --port 30333 --ws-port 9044 --rpc-port 9033 --ws-external --rpc-cors all --rpc-methods=unsafe --node-key 0x4cc79714885dced625d7fbf8341008ae4a1fb714452a62c50456597f29ecc270
```

#### Launch other three nodes

```bash
./target/release/felidae-node --chain staging -d data/validator2 --name validator2 --validator --port 30334 --ws-port 9045 --rpc-port 9034 --ws-external --rpc-cors all --rpc-methods=unsafe --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWRPm2bkfKzj8UDSGaDuCWEBBd61kbzuPLxXVLEPqZRweh
```

```bash
./target/release/felidae-node --chain staging -d data/validator3 --name validator3 --validator --port 30335 --ws-port 9046 --rpc-port 9035 --ws-external --rpc-cors all --rpc-methods=unsafe --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWRPm2bkfKzj8UDSGaDuCWEBBd61kbzuPLxXVLEPqZRweh
```

```bash
./target/release/felidae-node --chain staging -d data/validator4 --name validator4 --validator --port 30336 --ws-port 9047 --rpc-port 9036 --ws-external --rpc-cors all --rpc-methods=unsafe --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWRPm2bkfKzj8UDSGaDuCWEBBd61kbzuPLxXVLEPqZRweh
```

### Setup node session keys
- Copy validators' session keys to babe1 ~ 4, gran1 ~ 4, imol1 ~ 4, audi1 ~ 4, add prefix 0x.

- Fill right secret phrase

- Run command to setup node session keys
  ```bash
  cd scripts/session_keys
  ./run.sh
  ```