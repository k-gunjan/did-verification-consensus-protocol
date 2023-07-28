## Testing benchmarks

```bash
cargo test --package pallet-did --features runtime-benchmarks
```

```bash
./target/release/felidae-node benchmark pallet --chain dev --execution=wasm --wasm-execution=compiled --pallet pallet_did --extrinsic "*" --steps 50 --repeat 20 --output pallets/weights.rs
```