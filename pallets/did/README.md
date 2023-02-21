## DID pallet

### Implementation:
This DID pallet supports the following four Extrinsics for managing a DID:

#### Add Attribute:
* Add DID attribute that will compose the DID-document.
* Allow any attribute to be added. 
* No DID attribute validation.

#### Read Attribute:
* Read DID attribute if it exists on the DID-document else return error.
* Everyone can read DID-document.

#### Update Attribute:
* Update DID attribute already existing on the DID-document
* Return Not Found Error if it doesn't exist.
* Owner can only update their owned DID-documents.

#### Remove Attribute:
* Delete DID attribute already existing on the DID-document.
* Dispatch error if it doesn't exist.
* Owner can only remove attributes they own.

# Benchmarking
To generate pallet weights and enable benchmarking, run :

```bash
cargo build --release --features runtime-benchmarks
```

```bash
./target/release/felidae-node benchmark pallet \   
    --chain dev \
    --execution=wasm \
    --wasm-execution=compiled \
    --pallet pallet_did \
    --extrinsic "*" \
    --steps 50 \
    --repeat 20 \
    --output weights.rs
```

 








