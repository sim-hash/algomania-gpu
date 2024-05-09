use num_bigint::BigInt;
use num_traits::pow;

// largest valid address
pub fn max_address(prefix_max_len: usize) -> u64 {
    // TODO: Adjust this for strings...

    // longest address: 18446744073709551615 (20 chars)
    //
    // Example max_len = 15
    // Short address: 999999999999999 (15 chars)
    // Strict upper bound = 10^15 = 1000000000000000
    // return address <= self.max_address_value;

    if prefix_max_len >= 20 {
        18446744073709551615u64
    } else {
        pow(10u64, prefix_max_len) - 1
    }
}

pub struct PubkeyMatcher {
    max_address_value: u64,
    prefix: String,
    prefix_decoded: Vec<u8>,
}

impl PubkeyMatcher {

    pub fn new(prefix: String) -> PubkeyMatcher {
        assert!(prefix.len() >= 1 && prefix.len() <= 8);

        let prefix_decoded = base32::decode(base32::Alphabet::RFC4648 { padding: (true) }, &prefix).unwrap();
        println!("Pub-Key byte prefix is: {:?}", prefix_decoded);
        println!("prefix is: {:?}", prefix);

        PubkeyMatcher {
            max_address_value: max_address(prefix.len()),
            prefix,
            prefix_decoded,
        }
    }

    pub fn matches(&self, pubkey: [u8; 32]) -> bool {
        return self.prefix_decoded == pubkey[..self.prefix_decoded.len()]
    }

    pub fn starts_with(&self, address: String) -> bool {
        address.starts_with(&self.prefix)
    }

    pub fn estimated_attempts(&self) -> BigInt {
        println!("max {}", self.max_address_value);
        let number_of_good = BigInt::from(self.max_address_value) + BigInt::from(1);
        return (BigInt::from(1) << 64) / number_of_good;
    }
}

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_max_address() {
        assert_eq!(max_address(2000), 18446744073709551615u64);
        assert_eq!(max_address(200), 18446744073709551615u64);
        assert_eq!(max_address(20), 18446744073709551615u64);
        assert_eq!(max_address(15), 999999999999999u64);
        assert_eq!(max_address(10), 9999999999u64);
        assert_eq!(max_address(2), 99u64);
        assert_eq!(max_address(1), 9u64);
    }

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
