inline void print_bytes(const uchar *data, size_t len) {
 	for (size_t i = 0; i < len; ++i) {
 		printf("%.2x", data[i]);
 	}
 	printf("\n");
 }

 inline void print_words(const u32 *data, size_t len) {
 	for (size_t i = 0; i < len; ++i) {
 		printf("%.8x ", data[i]); }
 	printf("\n");
 }

/** * result:
 *     The 32 byte key material that is written once a matching address was found.
 *     This is all zero by default and any non-zero result indicates a match. All local
 *     threads write to the same global memory, so we can get corrupted results if
 *     multiple threads find a match. This usually does not happen for hard enough
 *     tasks but we need to double check the result in the caller code for this reason.
 * key_material_base:
 *     The root input key material. This is 32 bytes from a cryptographically secure
 *     random number generator. The thread ID is XORed into the last 8 bytes of this.
 * max_address_value:
 *     The largest address value that is considered a match, e.g. 999999999999 when
 *     looking for 12 digit addresses.
 */
__kernel void generate_pubkey (__global unsigned long *result, __global uchar *key_material_base, __global uchar *pub_req, __global uchar *pub_mask, uchar prefix_len, __global uchar *public_offset) {
	size_t const thread = get_global_id (0);
	uchar key_material[32];
	for (size_t i = 0; i < 32; i++) {
		key_material[i] = key_material_base[i];
	}

	*((size_t *) key_material) += thread;

	uchar menomic_hash[32];
	uchar *key;

	// privkey or extended privkey
	key = key_material;
	bignum256modm a;
	ge25519 ALIGN(16) A;

	u32 in[32] = { 0 }; // must be 128 bytes zero-filled for sha512_update to work
	uchar hash[64];

	sha512_ctx_t hasher;

	sha512_init (&hasher);

	to_32bytes_sha2_input(in, key);

	sha512_update(&hasher, in, 32);
	sha512_final(&hasher);

	from_sha512_result(hash, hasher.h);
	hash[0] &= 248;
	hash[31] &= 127;
	hash[31] |= 64;

	expand256_modm(a, hash, 32);
	ge25519_scalarmult_base_niels(&A, a);

	uchar pubkey[32];
	ge25519_pack(pubkey, &A);

	for (uchar i = 0; i < prefix_len; i++) {
		if ((pubkey[i] & pub_mask[i]) != pub_req[i]) {
			return;
		}
	}

	*result = thread;
}
