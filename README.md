# algomania-gpu

## Install on Ubuntu 22.04 / Vultr A100

curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh
source ~/.cargo/env
sudo apt install libssl-dev
rustup install 1.74.1
rustup default 1.74.1
cargo build --release
cd target/release
./algomania-gpu --gpu <PREFIX-HERE> -l <MAX_MATCHES>

## nVidia Docker Notes
https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html#prerequisites

## Other stuff for TensorDock
sudo apt install pkg-config
sudo apt install libssl-dev
### note: had to wait a bit for apt lock files to release
sudo ln -s /usr/local/cuda-12.2/targets/x86_64-linux/lib/libOpenCL.so.1 /usr/lib/libOpenCL.so
