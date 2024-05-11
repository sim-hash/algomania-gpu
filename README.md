# algomania-gpu

## Install on Ubuntu 22.04 / Vultr A100

curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh
source ~/.cargo/env
apt install libssl-dev
rustup install 1.74.1
rustup default 1.74.1
cargo build --release
cd target/release
./algomania-gpu --gpu <PREFIX-HERE> -l <MAX_MATCHES>