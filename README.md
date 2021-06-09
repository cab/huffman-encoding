# huffman-encoding

[![crates.io](https://img.shields.io/crates/v/huffman-encoding?label=latest)](https://crates.io/crates/huffman-encoding)

## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
huffman-encoding = "0.1"
```

```rust
// weights are represented as value -> frequency pairs
let weights = vec![
  ("hello".to_string(), 2),
  ("hey".to_string(), 3),
  ("howdy".to_string(), 1),
];
let huffman = huffman_encoding::Huffman::new(weights).unwrap();
let data = vec!["howdy".into(), "howdy".into(), "hey".into(), "hello".into()];
// encode into a bit_vec::BitVec
let encoded = huffman.encode(&data).unwrap();
// decode back into a Vec<String>
let decoded = huffman.decode_owned(&encoded);
```