use std::f64;
use std::process;
use std::sync::atomic;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

extern crate clap;
extern crate digest;
extern crate ed25519_dalek;
extern crate hex;
extern crate num_bigint;
extern crate num_cpus;
extern crate sha2;

extern crate rand;
extern crate algonaut;
extern crate base32;

use algonaut::transaction::account::Account;
use num_traits::pow;
use rand::{OsRng, Rng};

extern crate num_traits;
use num_traits::ToPrimitive;

#[cfg(feature = "gpu")]
extern crate ocl;

mod cpu;

mod derivation;
use derivation::secret_to_pubkey;

mod pubkey_matcher;
use pubkey_matcher::PubkeyMatcher;

#[cfg(feature = "gpu")]
mod gpu;
#[cfg(feature = "gpu")]
use gpu::Gpu;

use crate::gpu::GpuOptions;
use crate::pubkey_matcher::max_address;

struct ThreadParams {
    limit: usize,
    found_n: Arc<AtomicUsize>,
    output_progress: bool,
    attempts: Arc<AtomicUsize>,
    matcher: Arc<PubkeyMatcher>,
}

fn check_solution(params: &ThreadParams, key_material: [u8; 32]) -> bool {

    let matches  = params.matcher.matches(secret_to_pubkey(key_material));

    if matches {

        let wallet = Account::from_seed(key_material);

        if !params.matcher.starts_with(wallet.address().to_string()) {
            return false;
        }

        println!();
        println!("Found matching account!\nPrivate Key: {:?} \nAddress: {} \nMnemonic: {}", wallet.seed(), wallet.address(), wallet.mnemonic());
        println!();

        // TODO remove this
        if params.output_progress {
            eprintln!("");
        }

        if params.limit != 0
            && params.found_n.fetch_add(1, atomic::Ordering::Relaxed) + 1 >= params.limit
        {
            process::exit(0);
        }
    }
    matches
}

fn main() {
    let args = clap::App::new("lisk-vanity")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Simon Warta <simon@warta.it>")
        .about("Generate short Lisk addresses")
        .arg(
            clap::Arg::with_name("length")
                .value_name("LENGTH")
                .default_value("14")
                .required_unless("suffix")
                .help("The max length for the address"),
        )
        .arg(
            clap::Arg::with_name("cpu_threads")
                .short("t")
                .long("cpu-threads")
                .value_name("N")
                .help("The number of CPU threads to use [default: number of cores minus one]"),
        )
        .arg(
            clap::Arg::with_name("gpu")
                .short("g")
                .long("gpu")
                .help("Enable use of the GPU through OpenCL"),
        )
        .arg(
            clap::Arg::with_name("limit")
                .short("l")
                .long("limit")
                .value_name("N")
                .default_value("1")
                .help("Generate N addresses, then exit (0 for infinite)"),
        )
        .arg(
            clap::Arg::with_name("gpu_threads")
                .long("gpu-threads")
                .value_name("N")
                .default_value("1048576")
                .help("The number of GPU threads to use"),
        )
        .arg(
            clap::Arg::with_name("gpu_local_work_size")
                .long("gpu-local-work-size")
                .value_name("N")
                .help("The GPU local work size. A custom value it may increase performance. By default the OpenCL driver is responsible for setting a proper value. Don't use this if you don't know what you are doing."),
        )
        .arg(
            clap::Arg::with_name("no_progress")
                .long("no-progress")
                .help("Disable progress output"),
        )
        .arg(
            clap::Arg::with_name("gpu_platform")
                .long("gpu-platform")
                .value_name("INDEX")
                .default_value("0")
                .help("The GPU platform to use"),
        )
        .arg(
            clap::Arg::with_name("gpu_device")
                .long("gpu-device")
                .value_name("INDEX")
                .default_value("0")
                .help("The GPU device to use"),
        )
        .get_matches();

    // TODO change this
    let max_length: String = args
        .value_of("length")
        .unwrap()
        .parse()
        .expect("Failed to parse LENGTH");

    // TODO don't forget to validate the matcher_base
    let matcher_base = PubkeyMatcher::new(max_length.clone());

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
    let _generate_passphrase = args.is_present("generate_passphrase");

    // test this...???
    let gpu_global_work_size = args.value_of("gpu_global_work_size").map(|s| {
        s.parse()
            .expect("Failed to parse GPU local work size option")
    });

    let cpu_threads = args
        .value_of("cpu_threads")
        .map(|s| s.parse().expect("Failed to parse thread count option"))
        .unwrap_or_else(|| num_cpus::get() - 1);
    let mut thread_handles = Vec::with_capacity(cpu_threads);
    eprintln!("Estimated attempts needed: {}", estimated_attempts);
    for _ in 0..cpu_threads {
        let mut rng = OsRng::new().expect("Failed to get RNG for seed");
        let mut key_or_seed = [0u8; 32];
        rng.fill_bytes(&mut key_or_seed);
        let params = ThreadParams {
            limit,
            output_progress,
            matcher: matcher_base.clone(),
            found_n: found_n_base.clone(),
            attempts: attempts_base.clone(),
        };
        thread_handles.push(thread::spawn(move || loop {
            if check_solution(&params, key_or_seed) {
                rng.fill_bytes(&mut key_or_seed);
            } else {
                if output_progress {
                    params.attempts.fetch_add(1, atomic::Ordering::Relaxed);
                }
                for byte in key_or_seed.iter_mut().rev() {
                    *byte = byte.wrapping_add(1);
                    if *byte != 0 {
                        break;
                    }
                }
            }
        }));
    }

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
        let mut key_base = [0u8; 32];
        let params = ThreadParams {
            limit,
            output_progress,
            matcher: matcher_base.clone(),
            found_n: found_n_base.clone(),
            attempts: attempts_base.clone(),
        };

        // TODO whare is gpu_local_work size and gpu_global_work_size
        let mut gpu = Gpu::new(GpuOptions {
            platform_idx: gpu_platform,
            device_idx: gpu_device,
            threads: gpu_threads,
            local_work_size: gpu_local_work_size,
            global_work_size: gpu_global_work_size,
            max_address_value: max_address(max_length.len()),

        })
        .unwrap();
        gpu_thread = Some(thread::spawn(move || {
            let mut rng = OsRng::new().expect("Failed to get RNG for seed");
            loop {
                rng.fill_bytes(&mut key_base);
                let found = gpu
                    .compute(&key_base)
                    .expect("Failed to run GPU computation");
                if output_progress {
                    params
                        .attempts
                        .fetch_add(gpu_threads, atomic::Ordering::Relaxed);
                }

                if let Some(found_private_key) = found {
                    if !check_solution(&params, found_private_key) {
                        eprintln!(
                            "GPU returned non-matching solution: {}", hex::encode_upper(&found_private_key)
                        );
                    }
                } else {
                    // just continue
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
            thread::sleep(Duration::from_millis(100));
        });
    }
    if let Some(gpu_thread) = gpu_thread {
        gpu_thread.join().expect("Failed to join GPU thread");
    }
    for handle in thread_handles {
        handle.join().expect("Failed to join thread");
    }
    eprintln!("No computation devices specified");
    process::exit(1);
}
