use pubkey_matcher::PubkeyMatcher;

#[cfg(feature = "gpu")]
pub use gpu_impl::Gpu;


#[derive(Clone, Copy)]
pub struct GpuOptions<'a> {
    pub platform_idx: usize,
    pub device_idx: usize,
    pub threads: usize,
    pub local_work_size: Option<usize>,
    pub global_work_size: Option<usize>,
    pub matcher: &'a PubkeyMatcher,
}
