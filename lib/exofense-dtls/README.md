# Exofense-DTLS

DTLS Implementation for Exofense.

## To Run Example

### Without logging

```rust
// Server
cargo run --color=always --package exofense-dtls --example dtls_server -- --host 0.0.0.0:50000

//Client
cargo run --color=always --package exofense-dtls --example dtls_client -- --server 0.0.0.0:50000
```

### With logging

```rust
// Server
RUST_LOG="info" cargo run --color=always --package exofense-dtls --example dtls_server -- --host 0.0.0.0:50000

//Client
RUST_LOG="info" cargo run --color=always --package exofense-dtls --example dtls_client -- --server 0.0.0.0:50000
```

Logging level can be one of "error", "warn", "info", "debug", "trace".
