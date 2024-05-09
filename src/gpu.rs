use ocl::ProQue;
use ocl::builders::ProgramBuilder;
use ocl::enums::DeviceSpecifier;
use ocl::flags::MemFlags;
use ocl::Buffer;
use ocl::Platform;
use ocl::Result;

fn convert_ocl_error(error: ocl::Error) -> String {
    return error.to_string();
}

fn convert_ocl_core_error(error: ocl::OclCoreError) -> String {
    return error.to_string();
}

#[derive(Clone, Copy)]
pub struct GpuOptions {
    pub platform_idx: usize,
    pub device_idx: usize,
    pub threads: usize,
    pub local_work_size: Option<usize>,
    pub global_work_size: Option<usize>,
    pub max_address_value: u64,
}

pub struct Gpu {
    kernel: ocl::Kernel,
    result: Buffer<u8>,
    key_root: Buffer<u8>,
}

impl Gpu {

    pub fn new(opts: GpuOptions) -> Result<Gpu> {

        let mut prog_bldr = ProgramBuilder::new();

        // check this...
        let namespace_qualifier = if cfg!(feature = "apple") {
            "#define NAMESPACE_QUALIFIER __private\n"
        } else {
            "#define NAMESPACE_QUALIFIER __generic\n"
        };
        prog_bldr
            .source(namespace_qualifier)
            .src(include_str!("opencl/types.cl"))
            .src(include_str!("opencl/curve25519-constants.cl"))
            .src(include_str!("opencl/curve25519-constants2.cl"))
            .src(include_str!("opencl/curve25519.cl"))
            .src(include_str!("opencl/sha/inc_hash_functions.cl"))
            .src(include_str!("opencl/sha/sha256.cl"))
            .src(include_str!("opencl/sha/sha512.cl"))
            .src(include_str!("opencl/sha_bindings.cl"))
            .src(include_str!("opencl/bip39.cl"))
            .src(include_str!("opencl/lisk.cl"))
            .src(include_str!("opencl/entry.cl"));
        let platforms = Platform::list();

        if platforms.len() == 0 {
            return Err("No OpenCL platforms exist (check your drivers and OpenCL setup)".into());
        }
        if opts.platform_idx >= platforms.len() {
            return Err(format!(
                "Platform index {} too large (max {})",
                opts.platform_idx,
                platforms.len() - 1
            )
            .into());
        }

        let mut pro_que = ProQue::builder()
            .prog_bldr(prog_bldr)
            .platform(platforms[opts.platform_idx])
            .device(DeviceSpecifier::Indices(vec![opts.device_idx]))
            .dims(1)
            .build()?;

        let device = pro_que.device();
        eprintln!("Initializing GPU {} {}", device.vendor()?, device.name()?);

        // are dims set properly ?
        let result = pro_que
            .buffer_builder::<u8>()
            .flags(MemFlags::new().write_only())
            .len(32)
            //.fill_val(0u8)
            .build()?;
        pro_que.set_dims(32);

        let key_root = pro_que
            .buffer_builder::<u8>()
            .flags(MemFlags::new().read_only().host_write_only())
            .len(32)
            .build()?;
        pro_que.set_dims(6);

//        req.write(opts.matcher.req()).enq()?;
//        mask.write(opts.matcher.mask()).enq()?;
//        result.write(&[!0u64] as &[u64]).enq()?;

        let gen_key_type_code: u8 = 1;
        let kernel = {
            let mut kernel_builder = pro_que.kernel_builder("generate_pubkey");
            kernel_builder
                .global_work_size(opts.threads)
                .arg(&result)
                .arg(&key_root)
                // check this again and again....
                .arg(opts.max_address_value)
                .arg(gen_key_type_code);

            if let Some(local_work_size) = opts.local_work_size {
                kernel_builder.local_work_size(local_work_size);
            }
            if let Some(global_work_size) = opts.global_work_size {
                kernel_builder.global_work_size(global_work_size);
            }
            kernel_builder.build()?
        };

        Ok(Gpu {
            kernel,
            result,
            key_root,
        })
    }

    // ???????????????????????????????????????????????????????????//
    pub fn compute(&mut self, key_root: &[u8]) -> Result<Option<[u8; 32]>> {
        debug_assert!({
            // Ensure result is filled with zeros
            let mut result = [0u8; 32];
            self.result.read(&mut result as &mut [u8]).enq()?;
            result.iter().all(|&b| b == 0)
        });

        self.key_root.write(key_root).enq()?;
        unsafe {
            self.kernel.enq()?;
        }

        let mut out = [0u8; 32];
        self.result.read(&mut out as &mut [u8]).enq()?;

        let matched = !out.iter().all(|&b| b == 0);
        if matched {
            let zeros = [0u8; 32];
            self.result.write(&zeros as &[u8]).enq()?;
            return Ok(Option::Some(out));
        } else {
            return Ok(Option::None);
        }
    }
}

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;
    use pubkey_matcher::max_address;

//    #[test]
//    fn test_finds_private_key_directly() {
//        let gpu_options = GpuOptions { platform_idx: 0, device_idx: 0, threads: 1, local_work_size: None, global_work_size: max_address(15),  };
//        let gpu_platform = 0;
//        let gpu_device = 0;
//        let gpu_threads = 1; // Only a single attempt
//        let gpu_local_work_size = None; // let GPU device decide
//        let max_length = 15;
//        let mut gpu = Gpu::new(gpu_options).unwrap();
//
//        // 456C62AF90D3DFD765B7D4B56038CBE19AFA5AEA9CF3AA3B1E9E476C8CAFBBC2D4C27C4E12914952BDADF3C92FC2AC16230AEEA99E52D9E21DC3269EEF845488
//        // is the privkey in libsodium format. The first half is the private seed and the second half is the pubkey. The corresponding address
//        // is 550592072897524L (address length 15).
//        let mut key_base = [0u8; 32];
//        let mut expected_match = [0u8; 32];
//        hex::decode_to_slice(
//            "456c62af90d3dfd765b7d4b56038cbe19afa5aea9cf3aa3b1e9e476c8cafbbc2",
//            &mut key_base,
//        )
//        .unwrap();
//        hex::decode_to_slice(
//            "456c62af90d3dfd765b7d4b56038cbe19afa5aea9cf3aa3b1e9e476c8cafbbc2",
//            &mut expected_match,
//        )
//        .unwrap();
//
//        let found = gpu
//            .compute(&key_base)
//            .expect("Failed to run GPU computation");
//
//        if let Some(found_private_key) = found {
//            assert_eq!(found_private_key, expected_match);
//        } else {
//            panic!("No matching key found");
//        }
//    }
//
//    #[test]
//    fn test_finds_private_key_in_last_byte() {
//        let gpu_platform = 0;
//        let gpu_device = 0;
//        let gpu_threads = 256; // low number to allow quick tests on CPU platforms and ensure we don't find a different solution than expected
//        let gpu_local_work_size = None; // let GPU device decide
//        let max_length = 15;
//        let mut gpu = Gpu::new(
//            gpu_platform,
//            gpu_device,
//            gpu_threads,
//            gpu_local_work_size,
//            max_address(max_length),
//        )
//        .unwrap();
//
//        // 456C62AF90D3DFD765B7D4B56038CBE19AFA5AEA9CF3AA3B1E9E476C8CAFBBC2D4C27C4E12914952BDADF3C92FC2AC16230AEEA99E52D9E21DC3269EEF845488
//        // is the privkey in libsodium format. The first half is the private seed and the second half is the pubkey. The corresponding address
//        // is 550592072897524L (address length 15). We start with a key a little bit lower to check if we find this one.
//        let mut key_base = [0u8; 32];
//        let mut expected_match = [0u8; 32];
//        hex::decode_to_slice(
//            "456c62af90d3dfd765b7d4b56038cbe19afa5aea9cf3aa3b1e9e476c8cafbb00",
//            &mut key_base,
//        )
//        .unwrap();
//        hex::decode_to_slice(
//            "456c62af90d3dfd765b7d4b56038cbe19afa5aea9cf3aa3b1e9e476c8cafbbc2",
//            &mut expected_match,
//        )
//        .unwrap();
//
//        let found = gpu
//            .compute(&key_base)
//            .expect("Failed to run GPU computation");
//
//        if let Some(found_private_key) = found {
//            assert_eq!(found_private_key, expected_match);
//        } else {
//            panic!("No matching key found");
//        }
//    }
}
