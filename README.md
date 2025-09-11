# Hyperion Blockchain

A simple yet functional blockchain implementation written in Rust, featuring a modular architecture with separate components for core blockchain logic, node operations, and mining.

## Architecture

Hyperion is built with a modular design consisting of three main components:

- **`hyperion-core`**: Core blockchain primitives and consensus algorithms
- **`hyperion-node`**: Full node implementation with RPC server and networking
- **`hyperion-miner`**: Dedicated mining client with multi-threaded PoW mining

## Features

### Core Blockchain
- **Proof of Work (PoW)** consensus mechanism
- **Dynamic difficulty adjustment** based on block time
- **Merkle tree** transaction verification
- **Block validation** with proper chain linking
- **Serialization** support for persistence

### Node Capabilities
- **JSON-RPC API** server for mining and blockchain queries
- **Mempool** transaction management
- **Blockchain persistence** to disk
- **Network listener** for peer communication (basic)
- **Real-time block validation** and acceptance

### Mining Features
- **Multi-threaded mining** with configurable worker count
- **Work distribution** across mining threads
- **Automatic work restart** when blocks are found
- **Mining statistics** and hashrate reporting
- **Race condition prevention** with atomic solution detection
- **Configurable mining** via CLI and config files

## Quick Start

### Prerequisites

- Rust 1.89.0 or later
- Cargo package manager

### Installation

1. Clone the repository:
```bash
git clone https://github.com/Ashintosh/hyperion.git
cd hyperion
```

2. Build all components:
```bash
cargo build --release
```

### Running the Node

Start a Hyperion node with RPC server:

```bash
cargo run --bin hyperion-node
```

The node will:
- Listen for RPC requests on `http://127.0.0.1:6001`
- Start a network listener on `127.0.0.1:6000`
- Create a genesis block and initialize the blockchain
- Add test transactions to the mempool

### Running the Miner

In a separate terminal, start the miner:

```bash
cargo run --bin hyperion-miner
```

The miner will:
- Connect to the node's RPC server
- Request block templates
- Mine blocks using multiple threads
- Submit found blocks back to the node

### Configuration

Create a `config.toml` file in the `hyperion-miner` directory:

```toml
node_url = "http://127.0.0.1:6001"
threads = 4
reconnect_delay = 5
work_update_interval = 1000
stats_interval = 30
log_level = "info"
```

> NOTE: The miner is in a very basic form, and may not be fully configurable.

## 🔧 API Reference

### RPC Endpoints

The node exposes a JSON-RPC 2.0 API on port 6001:

#### `get_block_template`
Get a block template for mining.

```bash
curl -X POST http://127.0.0.1:6001/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getblocktemplate","params":null}'
```

#### `submit_block`
Submit a mined block.

```bash
curl -X POST http://127.0.0.1:6001/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"submitblock","params":{"block_hex":"..."}}'
```

#### `get_mining_info`
Get mining information and statistics.

```bash
curl -X POST http://127.0.0.1:6001/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"getmininginfo","params":null}'
```

#### `get_blockchain_info`
Get blockchain information.

```bash
curl -X POST http://127.0.0.1:6001/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":4,"method":"getblockchaininfo","params":null}'
```

#### `get_block_count`
Get the current block height.

```bash
curl -X POST http://127.0.0.1:6001/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":5,"method":"getblockcount","params":null}'
```

## Testing

Run the comprehensive test suite:

```bash
cargo test
```

The tests cover:
- Block validation and merkle tree computation
- Blockchain operations and validation
- Transaction handling
- Serialization/deserialization
- Proof of Work validation
- Difficulty adjustment algorithms

## Mining Statistics

When running, the miner displays real-time statistics:

```
Mining Stats - Hashrate: 1234.56 H/s, Blocks: 5, Uptime: 2m 30s
```

## Development

### Key Components

#### Consensus Algorithm
- **Target block time**: 600 seconds (10 minutes)
- **Difficulty adjustment**: Every 3 blocks
- **Proof of Work**: SHA-256 double hashing
- **Difficulty format**: Compact representation (similar to Bitcoin)

#### Block Structure
- **Version**: Block format version
- **Previous hash**: Hash of the previous block
- **Merkle root**: Root hash of transaction merkle tree
- **Timestamp**: Block creation time
- **Difficulty**: Target difficulty for PoW
- **Nonce**: Proof of Work nonce
- **Transactions**: List of transactions in the block

### Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

## Performance Notes

- The current implementation uses a very low difficulty for demonstration purposes
- In production, difficulty would be much higher and blocks would take longer to mine
- The miner uses atomic operations to prevent race conditions between workers
- Block validation includes full merkle tree verification and PoW checking

## Future Improvements

- [ ] Peer-to-peer networking with block propagation
- [ ] Transaction fees and UTXO model
- [ ] Digital signatures for transactions
- [ ] Wallet functionality for key management
- [ ] Mining pool support
- [ ] REST API alongside JSON-RPC
- [ ] Blockchain explorer interface
- [ ] Performance optimizations for high-difficulty mining
- [ ] Network synchronization and consensus
- [ ] Configuration for different network modes (testnet/mainnet)
- [ ] Significant code cleanup/refactoring

## 📄 License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by Bitcoin's blockchain architecture
- Built with the excellent Rust async ecosystem (Tokio, Axum)
- Thanks to the Rust community for outstanding documentation and tools

---

**Note**: This is an educational blockchain implementation. It is not suitable for production use in its current form and lacks many security features required for a real cryptocurrency.