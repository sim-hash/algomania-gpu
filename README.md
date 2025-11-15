# algomania-gpu

algomania-gpu is a GPU-accelerated vanity address generator for Algorand. It uses OpenCL for massively parallel ed25519 keypair generation and produces Algorand account mnemonics when public key matches are found.

## Requirements

- Rustup or another method to run stable Rust
- OpenCL 1.2+ compatible GPU and drivers
  - NVIDIA (CUDA-compatible via OpenCL)
  - AMD (via ROCm/OpenCL)
  - Intel (integrated or discrete GPUs)

## Installation

Clone and build from source:

```bash
git clone
cargo build --release
./target/release/algomania-gpu --version
```

## Basic Usage

Search for an address starting with "ALGO":

```bash
$ ./target/release/algomania-gpu --gpu ALGO
```

Generate multiple matching addresses with `--limit`:

```bash
$ ./target/release/algomania-gpu --gpu TEST --limit 5
```

Disable progress output with `--no-progress` for cleaner output when piping results.

## GPU Configuration

The tool supports OpenCL-based GPU computing. Specify GPU platform and device using `--gpu-platform [index]` and `--gpu-device [index]` flags respectively.

**Advanced tuning options:**
- `--gpu-threads N` - Number of parallel GPU threads (default: 1048576)
- `--gpu-local-work-size N` - Local work group size (optional, for advanced tuning)
- `--gpu-global-work-size N` - Global work size (optional, for advanced tuning)

Example with specific GPU device:

```bash
$ ./target/release/algomania-gpu --gpu MYPREFIX --gpu-platform 1 --gpu-device 0
```

## Optimizing Chances of Finding Matches

The difficulty of finding vanity addresses increases exponentially with prefix length:

- **1-3 characters**: Very quick, seconds to minutes on most GPUs
- **4-5 characters**: Minutes to hours on modern GPUs
- **6-7 characters**: Hours to days on high-end GPUs
- **8+ characters**: Days to weeks or longer, requires significant GPU resources

Performance scales with GPU compute power. Modern high-end GPUs can generate millions of keys per second.

## Algorand Address Format

Algorand addresses are 58-character Base32-encoded strings using the alphabet: `ABCDEFGHIJKLMNOPQRSTUVWXYZ234567`

The tool supports prefix matching with these characters. Wildcards (`.` and `*`) may be used in patterns.

## Troubleshooting

**OpenCL library not found**: Ensure OpenCL drivers are installed for your GPU. For NVIDIA, install CUDA toolkit. For AMD, install ROCm or AMD GPU drivers.

**Platform/Device index errors**: List available OpenCL platforms and devices using `clinfo` or similar tools to verify correct indices.

**Low performance**: Try adjusting `--gpu-threads` to match your GPU's compute capacity. Start with the default (1048576) and experiment with higher or lower values.

## Credits

algomania-gpu is inspired by and derived from:
- [lisk-vanity](https://github.com/webmaster128/lisk-vanity) by webmaster128
- [algomania](https://github.com/kirse/algomania) by kirse (CPU-only Algorand implementation)

SHA-512 OpenCL implementation sourced from hashcat. Curve25519 operations adapted for Algorand's Ed25519 key derivation.

## License

BSD-2-Clause
