use std::f64;
use std::process;
use std::sync::atomic;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

extern crate clap;
extern crate digest;
extern crate hex;
extern crate num_cpus;
extern crate algonaut;
extern crate ring;
extern crate sha2;
extern crate byteorder;

extern crate rand;
use rand::rngs::OsRng;
use rand::RngCore;

extern crate num_bigint;

extern crate num_traits;
use num_traits::ToPrimitive;

#[cfg(feature = "gpu")]
extern crate ocl;

use algonaut::transaction::account::Account;

mod derivation;
use derivation::ADDRESS_ALPHABET;

mod pubkey_matcher;
use pubkey_matcher::PubkeyMatcher;

#[cfg(feature = "gpu")]
mod gpu_impl;

mod gpu;
use gpu::{Gpu, GpuOptions};

struct ThreadParams {
    limit: usize,
    found_n: Arc<AtomicUsize>,
    attempts: Arc<AtomicUsize>,
    matcher: Arc<PubkeyMatcher>,
}


fn check_solution(params: &ThreadParams, key_material: [u8; 32]) -> bool {

    let public_key = derivation::ed25519_privkey_to_pubkey(&key_material);
    let matches = params.matcher.matches(&public_key);

    if matches {

        let wallet = Account::from_seed(key_material);
        println!(
            "\nFound matching account!\nPrivate Key: {:?} \nAddress: {} \nMnemonic: {}",
            wallet.seed(),
            wallet.address(),
            wallet.mnemonic()
        );
        println!();

        if params.limit != 0
            && params.found_n.fetch_add(1, atomic::Ordering::Relaxed) + 1 >= params.limit
        {
            process::exit(0);
        }
    }
    matches
}

fn char_to_base32_value(ch: char) -> Option<u8> {
    if ch == '.' || ch == '*' {
        Some(0)
    } else {
        ADDRESS_ALPHABET.iter().position(|&c| (c as char) == ch).map(|p| p as u8)
    }
}

fn create_req_mask_for_prefix(prefix: &str) -> (Vec<u8>, Vec<u8>) {
    let mut req = vec![0u8; 36];
    let mut mask = vec![0u8; 36];

    for (i, ch) in prefix.chars().enumerate() {
        if i >= 58 {
            break;
        }

        if let Some(value) = char_to_base32_value(ch) {
            match i % 8 {
                0 => {
                    mask[0] |= 0xF8;
                    req[0] |= value << 3;
                },
                1 => {
                    mask[0] |= 0x07;
                    mask[1] |= 0xC0;
                    req[0] |= (value >> 2) & 0x07;
                    req[1] |= (value & 0x03) << 6;
                },
                2 => {
                    mask[1] |= 0x3E;
                    req[1] |= (value << 1) & 0x3E;
                },
                3 => {
                    mask[1] |= 0x01;
                    mask[2] |= 0xF0;
                    req[1] |= (value >> 4) & 0x01;
                    req[2] |= (value & 0x0F) << 4;
                },
                4 => {
                    mask[2] |= 0x0F;
                    mask[3] |= 0x80;
                    req[2] |= (value >> 1) & 0x0F;
                    req[3] |= (value & 0x01) << 7;
                },
                5 => {
                    mask[3] |= 0x7C;
                    req[3] |= (value << 2) & 0x7C;
                },
                6 => {
                    mask[3] |= 0x03;
                    mask[4] |= 0xE0;
                    req[3] |= (value >> 3) & 0x03;
                    req[4] |= (value & 0x07) << 5;
                },
                7 => {
                    mask[4] |= 0x1F;
                    req[4] |= value & 0x1F;
                },
                _ => unreachable!()
            }
        }

        if i > 0 && i % 8 == 7 {
            let mut new_req = vec![0u8; 36];
            let mut new_mask = vec![0u8; 36];
            new_req[..31].copy_from_slice(&req[5..36]);
            new_mask[..31].copy_from_slice(&mask[5..36]);
            req = new_req;
            mask = new_mask;
        }
    }

    (req, mask)
}

fn main() {
    let args = clap::App::new("algomania-gpu")
        .version(env!("CARGO_PKG_VERSION"))
        //.author("Lee Bousfield <ljbousfield@gmail.com>")
        .about("Generate Algorand cryptocurrency addresses with a given prefix")
        .arg(
            clap::Arg::with_name("prefix")
                .value_name("PREFIX")
                .required_unless("suffix")
                .help("The prefix for the address"),
        ).arg(
            clap::Arg::with_name("gpu")
                .short("g")
                .long("gpu")
                .help("Enable use of the GPU through OpenCL"),
        ).arg(
            clap::Arg::with_name("limit")
                .short("l")
                .long("limit")
                .value_name("N")
                .default_value("1")
                .help("Generate N addresses, then exit (0 for infinite)"),
        ).arg(
            clap::Arg::with_name("gpu_threads")
                .long("gpu-threads")
                .value_name("N")
                .default_value("1048576")
                .help("The number of GPU threads to use"),
        ).arg(
            clap::Arg::with_name("gpu_local_work_size")
                .long("gpu-local-work-size")
                .value_name("N")
                .help("The GPU local work size. Increasing it may increase performance. For advanced users only."),
        ).arg(
            clap::Arg::with_name("gpu_global_work-size")
                .long("gpu-global-work-size")
                .value_name("N")
                .help("The GPU global work size. Increasing it may increase performance. For advanced users only."),
        ).arg(
            clap::Arg::with_name("no_progress")
                .long("no-progress")
                .help("Disable progress output"),
        ).arg(
            clap::Arg::with_name("gpu_platform")
                .long("gpu-platform")
                .value_name("INDEX")
                .default_value("0")
                .help("The GPU platform to use"),
        ).arg(
            clap::Arg::with_name("gpu_device")
                .long("gpu-device")
                .value_name("INDEX")
                .default_value("0")
                .help("The GPU device to use"),
        ).get_matches();

    let ext_pubkey_req: Vec<u8>;
    let ext_pubkey_mask: Vec<u8>;
    if let Some(prefix) = args.value_of("prefix") {
        println!("Processing prefix: {}", prefix);

        let (req, mask) = create_req_mask_for_prefix(prefix);
        ext_pubkey_req = req;
        ext_pubkey_mask = mask;

        if prefix.chars().count() > 58 {
            eprintln!("Warning: prefix too long.");
            eprintln!("Only the first 58 characters of your prefix will be used.");
            eprintln!("");
        }
    } else {
        eprintln!("You must specify a non-empty prefix");
        process::exit(1);
    }

    let matcher_base = PubkeyMatcher::new(ext_pubkey_req, ext_pubkey_mask);
    let estimated_attempts = matcher_base.estimated_attempts();
    let matcher_base = Arc::new(matcher_base);
    let limit = args
        .value_of("limit")
        .unwrap()
        .parse()
        .expect("Failed to parse limit option");
    let found_n_base = Arc::new(AtomicUsize::new(0));
    let attempts_base = Arc::new(AtomicUsize::new(0));
    let output_progress = !args.is_present("no_progress");
    //let simple_output = args.is_present("simple_output");
    let mut gpu_thread = None;
    if args.is_present("gpu") {
        let gpu_platform = args
            .value_of("gpu_platform")
            .unwrap()
            .parse()
            .expect("Failed to parse GPU platform index");
        let gpu_device = args
            .value_of("gpu_device")
            .unwrap()
            .parse()
            .expect("Failed to parse GPU device index");
        let gpu_threads = args
            .value_of("gpu_threads")
            .unwrap()
            .parse()
            .expect("Failed to parse GPU threads option");
        let gpu_local_work_size = args.value_of("gpu_local_work_size").map(|s| {
            s.parse()
                .expect("Failed to parse GPU local work size option")
        });
        let gpu_global_work_size = args.value_of("gpu_global_work_size").map(|s| {
            s.parse()
                .expect("Failed to parse GPU local work size option")
        });
        let mut key_base = [0u8; 32];
        let params = ThreadParams {
            limit,
            matcher: matcher_base.clone(),
            found_n: found_n_base.clone(),
            attempts: attempts_base.clone(),
        };
        let mut gpu = Gpu::new(GpuOptions {
            platform_idx: gpu_platform,
            device_idx: gpu_device,
            threads: gpu_threads,
            local_work_size: gpu_local_work_size,
            global_work_size: gpu_global_work_size,
            matcher: &params.matcher,
        })
        .unwrap();
        gpu_thread = Some(thread::spawn(move || {
            let mut found_private_key = [0u8; 32];
            loop {
                OsRng.fill_bytes(&mut key_base);
                let found = gpu
                    .compute(&mut found_private_key as _, &key_base as _)
                    .expect("Failed to run GPU computation");
                if output_progress {
                    params
                        .attempts
                        .fetch_add(gpu_threads, atomic::Ordering::Relaxed);
                }
                if !found {
                    continue;
                }

                if !check_solution(&params, found_private_key) {
                    eprintln!(
                        "GPU returned non-matching solution: {}",
                        hex::encode_upper(&found_private_key),
                    );
                }
                for byte in &mut found_private_key {
                    *byte = 0;
                }
            }
        }));
    }
    if output_progress {
        let start_time = Instant::now();
        let attempts = attempts_base;
        thread::spawn(move || loop {
            let attempts = attempts.load(atomic::Ordering::Relaxed);
            let estimated_percent =
                100. * (attempts as f64) / estimated_attempts.to_f64().unwrap_or(f64::INFINITY);
            let runtime = start_time.elapsed();
            let keys_per_second = (attempts as f64)
                // simplify to .as_millis() when available
                / (runtime.as_secs() as f64 + runtime.subsec_millis() as f64 / 1000.0);
            eprint!(
                "\rTried {} keys (~{:.2}%; {:.1} keys/s)",
                attempts, estimated_percent, keys_per_second,
            );
            thread::sleep(Duration::from_millis(250));
        });
    }
    if let Some(gpu_thread) = gpu_thread {
        gpu_thread.join().expect("Failed to join GPU thread");
    }
    eprintln!("No computation devices specified");
    process::exit(1);
}
