use ed25519_dalek::{PublicKey, SecretKey};
use sha2::Sha512;

fn ed25519_privkey_to_pubkey(sec: &[u8; 32]) -> [u8; 32] {
    let secret_key = SecretKey::from_bytes(sec).unwrap();
    let public_key = PublicKey::from_secret::<Sha512>(&secret_key);
    public_key.to_bytes()
}

pub fn secret_to_pubkey(key_material: [u8; 32]) -> [u8; 32] {
    ed25519_privkey_to_pubkey(&key_material)
}

#[cfg(test)]
mod tests {
  // importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_ed25519_secret_to_pubkey() {
        // TEST 1 from https://tools.ietf.org/html/rfc8032#section-7.1
        let mut privkey = [0u8; 32];
        privkey.copy_from_slice(
            &hex::decode("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
                .unwrap(),
        );
        let mut expected_pubkey = [0u8; 32];
        expected_pubkey.copy_from_slice(
            &hex::decode("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
                .unwrap(),
        );
        assert_eq!(ed25519_privkey_to_pubkey(&privkey), expected_pubkey);
    }

    #[test]
    fn test_secret_to_pubkey_from_privkey() {
        // TEST 1 from https://tools.ietf.org/html/rfc8032#section-7.1
        let mut privkey = [0u8; 32];
        privkey.copy_from_slice(
            &hex::decode("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
                .unwrap(),
        );
        let mut expected_pubkey = [0u8; 32];
        expected_pubkey.copy_from_slice(
            &hex::decode("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
                .unwrap(),
        );
        assert_eq!(
            secret_to_pubkey(privkey),
            expected_pubkey
        );
    }
}
