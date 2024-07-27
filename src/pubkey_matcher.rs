use num_bigint::BigInt;
use num_traits::pow;

use derivation::pubkey_to_address;

// largest valid address
pub fn max_address(input: &str) -> u64 {
    let mut result: u64 = 0;
    for (i, c) in input.chars().enumerate() {
        let char_value = c as u64;
        result += char_value * 10u64.pow((input.len() - 1 - i) as u32);
    }
    result
}

pub struct PubkeyMatcher {
    prefix: String,
    mask: Vec<u8>,
}

impl PubkeyMatcher {

    pub fn new(prefix: String, mask: Vec<u8>) -> PubkeyMatcher {
        PubkeyMatcher {
            prefix,
            mask,
        }
    }

    pub fn matches(&self, pubkey: [u8; 32]) -> bool {
        return self.mask == pubkey[..self.mask.len()]
    }

    pub fn starts_with(&self, address: String) -> bool {
        address.starts_with(&self.prefix)
    }

    pub fn estimated_attempts(&self) -> BigInt {
        let mut bits_in_mask = 0;
        for byte in &self.mask {
            bits_in_mask += byte.count_ones() as usize;
        }
        BigInt::from(1) << bits_in_mask
    }
}

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;

//    #[test]
//    fn test_max_address() {
//        assert_eq!(max_address(2000), 18446744073709551615u64);
//        assert_eq!(max_address(200), 18446744073709551615u64);
//        assert_eq!(max_address(20), 18446744073709551615u64);
//        assert_eq!(max_address(15), 999999999999999u64);
//        assert_eq!(max_address(10), 9999999999u64);
//        assert_eq!(max_address(2), 99u64);
//        assert_eq!(max_address(1), 9u64);
//    }

//    #[test]
//    fn test_estimated_attempts() {
//        let matcher_all = PubkeyMatcher::new(10000);
//        let estimated = matcher_all.estimated_attempts();
//        assert_eq!(estimated, BigInt::from(1));
//
//        // truncate(2^64 / 10^15)
//        let matcher_fifteen = PubkeyMatcher::new(15);
//        let estimated = matcher_fifteen.estimated_attempts();
//        assert_eq!(estimated, BigInt::from(18446));
//
//        // truncate(2^64 / 10^10)
//        let matcher_ten = PubkeyMatcher::new(10);
//        let estimated = matcher_ten.estimated_attempts();
//        assert_eq!(estimated, BigInt::from(1844674407));
//
//        // truncate(2^64 / 10^5)
//        let matcher_five = PubkeyMatcher::new(5);
//        let estimated = matcher_five.estimated_attempts();
//        assert_eq!(estimated, BigInt::from(184467440737095u64));
//
//        // truncate(2^64 / 10^3)
//        let matcher_three = PubkeyMatcher::new(3);
//        let estimated = matcher_three.estimated_attempts();
//        assert_eq!(estimated, BigInt::from(18446744073709551u64));
//    }
}
