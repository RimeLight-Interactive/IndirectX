/// DXBC container checksum (modified MD5).
///
/// Ported from vkd3d-proton `libs/vkd3d-shader/checksum.c`, which itself
/// credits RenderDoc (`dxbc_container.cpp`, © Baldur Karlsson / Crytek, MIT).
///
/// The algorithm is standard MD5 with a non-standard padding scheme:
///   - Input starts at byte offset 20 (skipping the DXBC magic + the 4×u32
///     checksum field itself + the u32 version field).
///   - The bit-length footer uses two 32-bit words: `num_bits` and
///     `(num_bits >> 2) | 1`, rather than the standard 64-bit LE length.
///
/// # Usage
/// ```
/// let mut blob: Vec<u8> = build_my_dxbc_container();
/// let hash = dxbc_checksum(&blob);
/// // Checksum lives at bytes 4..20 (four little-endian u32s).
/// blob[4..8].copy_from_slice(&hash[0].to_le_bytes());
/// blob[8..12].copy_from_slice(&hash[1].to_le_bytes());
/// blob[12..16].copy_from_slice(&hash[2].to_le_bytes());
/// blob[16..20].copy_from_slice(&hash[3].to_le_bytes());
/// ```

// ---------------------------------------------------------------------------
// MD5 core (RFC 1321) — per-round constants and the four auxiliary functions.
// ---------------------------------------------------------------------------

const S: [[u32; 4]; 4] = [
    [7, 12, 17, 22], // Round 1
    [5,  9, 14, 20], // Round 2
    [4, 11, 16, 23], // Round 3
    [6, 10, 15, 21], // Round 4
];

/// Precomputed table: `T[i] = floor(2^32 * |sin(i+1)|)` for i in 0..64.
#[rustfmt::skip]
const T: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee,
    0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
    0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa,
    0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
    0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
    0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05,
    0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039,
    0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
    0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

#[derive(Clone)]
struct Md5 {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
}

impl Md5 {
    fn new() -> Self {
        Self {
            a: 0x67452301,
            b: 0xefcdab89,
            c: 0x98badcfe,
            d: 0x10325476,
        }
    }

    /// Process one 64-byte (512-bit) block given as 16 little-endian u32s.
    fn compress(&mut self, m: &[u32; 16]) {
        let (mut a, mut b, mut c, mut d) = (self.a, self.b, self.c, self.d);

        for i in 0usize..64 {
            let (f, g): (u32, usize) = match i {
                0..=15  => ((b & c) | (!b & d),           i),
                16..=31 => ((d & b) | (!d & c),           (5 * i + 1) % 16),
                32..=47 => (b ^ c ^ d,                    (3 * i + 5) % 16),
                _       => (c ^ (b | !d),                 (7 * i) % 16),
            };
            let round = i / 16;
            let shift = S[round][i % 4];
            let tmp = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                (a.wrapping_add(f).wrapping_add(m[g]).wrapping_add(T[i]))
                    .rotate_left(shift),
            );
            a = tmp;
        }

        self.a = self.a.wrapping_add(a);
        self.b = self.b.wrapping_add(b);
        self.c = self.c.wrapping_add(c);
        self.d = self.d.wrapping_add(d);
    }

    /// Feed an arbitrary byte slice — must be a multiple of 64 bytes.
    fn update(&mut self, data: &[u8]) {
        assert!(data.len() % 64 == 0);
        for chunk in data.chunks_exact(64) {
            let mut m = [0u32; 16];
            for (i, w) in m.iter_mut().enumerate() {
                let off = i * 4;
                *w = u32::from_le_bytes(chunk[off..off + 4].try_into().unwrap());
            }
            self.compress(&m);
        }
    }
}

// ---------------------------------------------------------------------------
// DXBC-specific checksum
// ---------------------------------------------------------------------------

/// The DXBC container header layout (all LE):
///   0x00  magic "DXBC"   (4 bytes)
///   0x04  checksum[0..4] (16 bytes) ← this is what we compute
///   0x14  version        (4 bytes)
///   0x18  total_size     (4 bytes)
///   0x1C  chunk_count    (4 bytes)
///
/// Hashing starts at offset 20 (0x14), i.e. skipping magic + checksum.
const PAYLOAD_OFFSET: usize = 20;

/// Compute the four u32 DXBC checksum words for a fully-assembled container.
/// The blob must be at least 21 bytes long (as the DXBC header demands).
///
/// Write the result back into `blob[4..20]` as four LE u32s after calling this.
pub fn dxbc_checksum(blob: &[u8]) -> [u32; 4] {
    assert!(
        blob.len() > PAYLOAD_OFFSET,
        "blob too short to be a DXBC container"
    );

    let data = &blob[PAYLOAD_OFFSET..];
    let length = data.len();

    let num_bits: u32 = (length as u32).wrapping_mul(8);
    // DXBC-specific: second length word is (num_bits >> 2) | 1, NOT the high
    // 32 bits of a 64-bit length as standard MD5 would use.
    let num_bits2: u32 = (num_bits >> 2) | 1;

    let leftover = length % 64;
    let full_blocks_len = length - leftover;

    let mut md5 = Md5::new();

    // Feed all complete 64-byte blocks.
    md5.update(&data[..full_blocks_len]);

    let tail = &data[full_blocks_len..]; // `leftover` bytes

    // Now feed the non-standard padding.  vkd3d splits into two cases based
    // on whether the leftover fits in one final block alongside the footer.
    if leftover >= 56 {
        // Leftover + 0x80 byte don't fit together with the footer in one block.
        // Block A: leftover bytes + 0x80 + zeros to fill 64 bytes.
        let mut block_a = [0u8; 64];
        block_a[..leftover].copy_from_slice(tail);
        block_a[leftover] = 0x80; // the one-bit sentinel
        // (rest of block_a is already zero)
        md5.update(&block_a);

        // Block B: num_bits in first word, zeros in [1..14], num_bits2 in last word.
        let mut block_b = [0u8; 64];
        block_b[0..4].copy_from_slice(&num_bits.to_le_bytes());
        block_b[60..64].copy_from_slice(&num_bits2.to_le_bytes());
        md5.update(&block_b);
    } else {
        // Everything fits in one final 64-byte block.
        // Layout (matching vkd3d exactly):
        //   [0..4]                    num_bits  (LE u32)
        //   [4..4+leftover]           tail bytes
        //   [4+leftover..padding_end] 0x80 + zeros
        //   [padding_end..60]         zeros  (part of `block`)
        //   [60..64]                  num_bits2 (LE u32)
        //
        // `padding_bytes = 64 - leftover - 4`, then 0x80 goes at
        // `padding_bytes - 4` bytes into the zero block (i.e. right after tail
        // and the 4-byte num_bits2 slot at the end of that sub-block).

        let padding_bytes = 64 - leftover - 4; // bytes remaining after num_bits + tail

        // Feed num_bits first (4 bytes).
        let mut prefix = [0u8; 4];
        prefix.copy_from_slice(&num_bits.to_le_bytes());

        // Then tail, then the padding block.
        let mut pad_block = [0u8; 64]; // big enough
        // 0x80 sentinel sits at offset (padding_bytes - 4) within pad_block,
        // because the last 4 bytes of padding_bytes are reserved for num_bits2.
        pad_block[padding_bytes - 4] = 0x80;
        pad_block[padding_bytes - 4 + 1..padding_bytes].fill(0);
        pad_block[padding_bytes..padding_bytes + 4]
            .copy_from_slice(&num_bits2.to_le_bytes());

        // We need to feed exactly 64 bytes total for this last block.
        // Assemble: num_bits(4) + tail(leftover) + pad_block(padding_bytes+4)
        // = 4 + leftover + padding_bytes + 4 = 4 + leftover + (64 - leftover - 4) + 4 = 68?
        // Wait — let's re-read vkd3d carefully.
        //
        // vkd3d does:
        //   MD5_Update(num_bits, 4)           → 4 bytes
        //   MD5_Update(tail, leftover)         → leftover bytes
        //   block[0] = 0x80
        //   memcpy(block + padding_bytes - 4, &num_bits2, 4)
        //   MD5_Update(block, padding_bytes)   → padding_bytes bytes
        //
        // Total = 4 + leftover + padding_bytes
        //       = 4 + leftover + (64 - leftover - 4)
        //       = 64 ✓
        //
        // So `block` here is NOT 64 bytes — it's `padding_bytes` bytes, with
        // 0x80 at index 0 and num_bits2 at index (padding_bytes-4).

        let mut final_block = vec![0u8; 4 + leftover + padding_bytes];
        final_block[0..4].copy_from_slice(&num_bits.to_le_bytes());
        final_block[4..4 + leftover].copy_from_slice(tail);
        // padding portion starts at 4 + leftover
        let pad_start = 4 + leftover;
        final_block[pad_start] = 0x80;
        // zeros already there
        final_block[pad_start + padding_bytes - 4..pad_start + padding_bytes]
            .copy_from_slice(&num_bits2.to_le_bytes());

        // final_block is exactly 64 bytes; feed it.
        debug_assert_eq!(final_block.len(), 64);
        md5.update(&final_block);
    }

    [md5.a, md5.b, md5.c, md5.d]
}

/// Convenience: patch the checksum field in-place on an already-built container.
pub fn patch_dxbc_checksum(blob: &mut [u8]) {
    let [a, b, c, d] = dxbc_checksum(blob);
    blob[4..8].copy_from_slice(&a.to_le_bytes());
    blob[8..12].copy_from_slice(&b.to_le_bytes());
    blob[12..16].copy_from_slice(&c.to_le_bytes());
    blob[16..20].copy_from_slice(&d.to_le_bytes());
}

