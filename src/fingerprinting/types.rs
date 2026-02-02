#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FramePeaks {
    pub bins: [u16; 6],
    pub keep_mask: u8,
}

impl FramePeaks {
    pub fn is_empty(&self) -> bool {
        self.keep_mask == 0
    }

    pub fn iter_kept_bins(&self) -> impl Iterator<Item = u16> + '_ {
        (0..6).filter_map(|i| {
            let bit = 1u8 << i;
            if (self.keep_mask & bit) != 0 {
                Some(self.bins[i])
            } else {
                None
            }
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct FingerprintSample {
    pub keys: Vec<u64>,
    pub anchor_times: Vec<f32>,
}
