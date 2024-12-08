
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust Version](https://img.shields.io/badge/rust-1.67%2B-orange.svg)](https://www.rust-lang.org)

# OmniTensor Node

The `omnitensor-node` is the core component of the OmniTensor network, a decentralized AI infrastructure designed to democratize access to AI compute resources.

## Features

- Decentralized AI compute network
- Proof-of-Work (PoW) and Proof-of-Stake (PoS) hybrid consensus mechanism
- GPU and NPU support for AI computations
- Secure and efficient task distribution and result aggregation
- Built-in marketplace for AI models and compute resources
- Advanced telemetry and monitoring capabilities
- Decentralized AI inference using community GPUs
- Blockchain consensus integration
- Network communication with P2P layer
- Scalable task scheduling for AI workloads

## Getting Started

### Prerequisites

Ensure you have the following installed:

- Rust (latest stable version)
- Cargo (comes with Rust)
- A compatible GPU (optional for compute tasks)

### Build Instructions

Clone the repository:

```
git clone https://github.com/OmniTensor/omnitensor-node/
cd omnitensor-node
```

Build the project:

```
make build
```

Run the node:

```
make run
```

### Testing

To run the tests:

```
make test
```

## Contributing

We welcome contributions from the community! Please read the [CONTRIBUTING.md](https://github.com/OmniTensor/omnitensor-node/blob/main/CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Prerequisites

- Rust 1.67 or higher
- CUDA 11.0 or higher (for GPU support)
- RocksDB 6.20.3 or higher
- CMake 3.16 or higher

## Contact

For support or inquiries, please contact us at [team@omnitensor.io](mailto:team@omnitensor.io).
