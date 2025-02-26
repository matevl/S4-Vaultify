// Importing the necessary libraries

// Definition of the S-Box used in AES
const S_BOX: [u8; 256] = [
    0x63, 0x7C, 0x77, 0x7B, 0xF2, 0x6B, 0x6F, 0xC5, 0x30, 0x01, 0x67, 0x2B, 0xFE, 0xD7, 0xAB, 0x76,
    0xCA, 0x82, 0xC9, 0x7D, 0xFA, 0x59, 0x47, 0xF0, 0xAD, 0xD4, 0xA2, 0xAF, 0x9C, 0xA4, 0x72, 0xC0,
    0xB7, 0xFD, 0x93, 0x26, 0x36, 0x3F, 0xF7, 0xCC, 0x34, 0xA5, 0xE5, 0xF1, 0x71, 0xD8, 0x31, 0x15,
    0x04, 0xC7, 0x23, 0xC3, 0x18, 0x96, 0x05, 0x9A, 0x07, 0x12, 0x80, 0xE2, 0xEB, 0x27, 0xB2, 0x75,
    0x09, 0x83, 0x2C, 0x1A, 0x1B, 0x6E, 0x5A, 0xA0, 0x52, 0x3B, 0xD6, 0xB3, 0x29, 0xE3, 0x2F, 0x84,
    0x53, 0xD1, 0x00, 0xED, 0x20, 0xFC, 0xB1, 0x5B, 0x6A, 0xCB, 0xBE, 0x39, 0x4A, 0x4C, 0x58, 0xCF,
    0xD0, 0xEF, 0xAA, 0xFB, 0x43, 0x4D, 0x33, 0x85, 0x45, 0xF9, 0x02, 0x7F, 0x50, 0x3C, 0x9F, 0xA8,
    0x51, 0xA3, 0x40, 0x8F, 0x92, 0x9D, 0x38, 0xF5, 0xBC, 0xB6, 0xDA, 0x21, 0x10, 0xFF, 0xF3, 0xD2,
    0xCD, 0x0C, 0x13, 0xEC, 0x5F, 0x97, 0x44, 0x17, 0xC4, 0xA7, 0x7E, 0x3D, 0x64, 0x5D, 0x19, 0x73,
    0x60, 0x81, 0x4F, 0xDC, 0x22, 0x2A, 0x90, 0x88, 0x46, 0xEE, 0xB8, 0x14, 0xDE, 0x5E, 0x0B, 0xDB,
    0xE0, 0x32, 0x3A, 0x0A, 0x49, 0x06, 0x24, 0x5C, 0xC2, 0xD3, 0xAC, 0x62, 0x91, 0x95, 0xE4, 0x79,
    0xE7, 0xC8, 0x37, 0x6D, 0x8D, 0xD5, 0x4E, 0xA9, 0x6C, 0x56, 0xF4, 0xEA, 0x65, 0x7A, 0xAE, 0x08,
    0xBA, 0x78, 0x25, 0x2E, 0x1C, 0xA6, 0xB4, 0xC6, 0xE8, 0xDD, 0x74, 0x1F, 0x4B, 0xBD, 0x8B, 0x8A,
    0x70, 0x3E, 0xB5, 0x66, 0x48, 0x03, 0xF6, 0x0E, 0x61, 0x35, 0x57, 0xB9, 0x86, 0xC1, 0x1D, 0x9E,
    0xE1, 0xF8, 0x98, 0x11, 0x69, 0xD9, 0x8E, 0x94, 0x9B, 0x1E, 0x87, 0xE9, 0xCE, 0x55, 0x28, 0xDF,
    0x8C, 0xA1, 0x89, 0x0D, 0xBF, 0xE6, 0x42, 0x68, 0x41, 0x99, 0x2D, 0x0F, 0xB0, 0x54, 0xBB, 0x16,
];

// Definition of the inverse S-Box used for decryption
const INV_S_BOX: [u8; 256] = [
    0x52, 0x09, 0x6A, 0xD5, 0x30, 0x36, 0xA5, 0x38, 0xBF, 0x40, 0xA3, 0x9E, 0x81, 0xF3, 0xD7, 0xFB,
    0x7C, 0xE3, 0x39, 0x82, 0x9B, 0x2F, 0xFF, 0x87, 0x34, 0x8E, 0x43, 0x44, 0xC4, 0xDE, 0xE9, 0xCB,
    0x54, 0x7B, 0x94, 0x32, 0xA6, 0xC2, 0x23, 0x3D, 0xEE, 0x4C, 0x95, 0x0B, 0x42, 0xFA, 0xC3, 0x4E,
    0x08, 0x2E, 0xA1, 0x66, 0x28, 0xD9, 0x24, 0xB2, 0x76, 0x5B, 0xA2, 0x49, 0x6D, 0x8B, 0xD1, 0x25,
    0x72, 0xF8, 0xF6, 0x64, 0x86, 0x68, 0x98, 0x16, 0xD4, 0xA4, 0x5C, 0xCC, 0x5D, 0x65, 0xB6, 0x92,
    0x6C, 0x70, 0x48, 0x50, 0xFD, 0xED, 0xB9, 0xDA, 0x5E, 0x15, 0x46, 0x57, 0xA7, 0x8D, 0x9D, 0x84,
    0x90, 0xD8, 0xAB, 0x00, 0x8C, 0xBC, 0xD3, 0x0A, 0xF7, 0xE4, 0x58, 0x05, 0xB8, 0xB3, 0x45, 0x06,
    0xD0, 0x2C, 0x1E, 0x8F, 0xCA, 0x3F, 0x0F, 0x02, 0xC1, 0xAF, 0xBD, 0x03, 0x01, 0x13, 0x8A, 0x6B,
    0x3A, 0x91, 0x11, 0x41, 0x4F, 0x67, 0xDC, 0xEA, 0x97, 0xF2, 0xCF, 0xCE, 0xF0, 0xB4, 0xE6, 0x73,
    0x96, 0xAC, 0x74, 0x22, 0xE7, 0xAD, 0x35, 0x85, 0xE2, 0xF9, 0x37, 0xE8, 0x1C, 0x75, 0xDF, 0x6E,
    0x47, 0xF1, 0x1A, 0x71, 0x1D, 0x29, 0xC5, 0x89, 0x6F, 0xB7, 0x62, 0x0E, 0xAA, 0x18, 0xBE, 0x1B,
    0xFC, 0x56, 0x3E, 0x4B, 0xC6, 0xD2, 0x79, 0x20, 0x9A, 0xDB, 0xC0, 0xFE, 0x78, 0xCD, 0x5A, 0xF4,
    0x1F, 0xDD, 0xA8, 0x33, 0x88, 0x07, 0xC7, 0x31, 0xB1, 0x12, 0x10, 0x59, 0x27, 0x80, 0xEC, 0x5F,
    0x60, 0x51, 0x7F, 0xA9, 0x19, 0xB5, 0x4A, 0x0D, 0x2D, 0xE5, 0x7A, 0x9F, 0x93, 0xC9, 0x9C, 0xEF,
    0xA0, 0xE0, 0x3B, 0x4D, 0xAE, 0x2A, 0xF5, 0xB0, 0xC8, 0xEB, 0xBB, 0x3C, 0x83, 0x53, 0x99, 0x61,
    0x17, 0x2B, 0x04, 0x7E, 0xBA, 0x77, 0xD6, 0x26, 0xE1, 0x69, 0x14, 0x63, 0x55, 0x21, 0x0C, 0x7D,
];

// Definition of the RCON constants used in AES key expansion
const RCON: [[u8; 4]; 7] = [
    [0x01, 0x00, 0x00, 0x00],
    [0x02, 0x00, 0x00, 0x00],
    [0x04, 0x00, 0x00, 0x00],
    [0x08, 0x00, 0x00, 0x00],
    [0x10, 0x00, 0x00, 0x00],
    [0x20, 0x00, 0x00, 0x00],
    [0x40, 0x00, 0x00, 0x00],
];

// Function that performs a circular rotation of a word (an array of 4 bytes)
pub fn rot_word(word: [u8; 4]) -> [u8; 4] {
    [word[1], word[2], word[3], word[0]]
}

// Function that applies the S-Box to each byte of a word
pub fn sub_word(word: [u8; 4]) -> [u8; 4] {
    [
        S_BOX[word[0] as usize],
        S_BOX[word[1] as usize],
        S_BOX[word[2] as usize],
        S_BOX[word[3] as usize],
    ]
}

// Function that performs the XOR operation between two words
pub fn xor_words(a: [u8; 4], b: [u8; 4]) -> [u8; 4] {
    [a[0] ^ b[0], a[1] ^ b[1], a[2] ^ b[2], a[3] ^ b[3]]
}

// Key expansion function for AES-256 (256-bit key)
// Returns a vector containing the round keys
pub fn key_expansion(key: &[u8]) -> Vec<[[u8; 4]; 4]> {
    const NK: usize = 8; // Number of words in the key (256 bits / 32 bits per word)
    const NR: usize = 14; // Number of rounds for AES-256
    const NB: usize = 4; // Number of columns in the state
    let total_words = NB * (NR + 1);
    let mut words: Vec<[u8; 4]> = Vec::with_capacity(total_words);

    // Initialize the first words with the original key
    for i in 0..NK {
        words.push([key[4 * i], key[4 * i + 1], key[4 * i + 2], key[4 * i + 3]]);
    }

    // Generate the remaining words for key expansion
    for i in NK..total_words {
        let mut temp = words[i - 1];
        if i % NK == 0 {
            temp = sub_word(rot_word(temp));
            let rcon = RCON[(i / NK) - 1];
            temp = xor_words(temp, rcon);
        } else if i % NK == 4 {
            temp = sub_word(temp);
        }
        let word = xor_words(words[i - NK], temp);
        words.push(word);
    }

    // Transform the generated words into round keys (4x4 matrices)
    let mut round_keys: Vec<[[u8; 4]; 4]> = Vec::with_capacity(NR + 1);
    for round in 0..(NR + 1) {
        let mut round_key = [[0u8; 4]; 4];
        for c in 0..4 {
            for r in 0..4 {
                round_key[r][c] = words[round * 4 + c][r];
            }
        }
        round_keys.push(round_key);
    }
    round_keys
}

// Function that applies the SubBytes transformation to the state
pub fn sub_bytes(state: &mut [[u8; 4]; 4]) {
    for r in 0..4 {
        for c in 0..4 {
            state[r][c] = S_BOX[state[r][c] as usize];
        }
    }
}

// Function that performs the ShiftRows transformation on the state
pub fn shift_rows(state: &mut [[u8; 4]; 4]) {
    state[1].rotate_left(1); // Shift the second row one byte to the left
    state[2].rotate_left(2); // Shift the third row two bytes to the left
    state[3].rotate_left(3); // Shift the fourth row three bytes to the left
}

// Function that multiplies a byte by 2 in GF(2^8)
pub fn xtime(x: u8) -> u8 {
    if x & 0x80 != 0 {
        (x << 1) ^ 0x1b
    } else {
        x << 1
    }
}

// Function that performs the MixColumns transformation on the state
pub fn mix_columns(state: &mut [[u8; 4]; 4]) {
    for c in 0..4 {
        let a0 = state[0][c];
        let a1 = state[1][c];
        let a2 = state[2][c];
        let a3 = state[3][c];

        let r0 = xtime(a0) ^ (xtime(a1) ^ a1) ^ a2 ^ a3;
        let r1 = a0 ^ xtime(a1) ^ (xtime(a2) ^ a2) ^ a3;
        let r2 = a0 ^ a1 ^ xtime(a2) ^ (xtime(a3) ^ a3);
        let r3 = (xtime(a0) ^ a0) ^ a1 ^ a2 ^ xtime(a3);

        state[0][c] = r0;
        state[1][c] = r1;
        state[2][c] = r2;
        state[3][c] = r3;
    }
}

// Function that adds the round key to the state (XOR operation)
pub fn add_round_key(state: &mut [[u8; 4]; 4], round_key: &[[u8; 4]; 4]) {
    for r in 0..4 {
        for c in 0..4 {
            state[r][c] ^= round_key[r][c];
        }
    }
}

// Function that converts a 16-byte block into a 4x4 state matrix
pub fn block_to_state(block: &[u8]) -> [[u8; 4]; 4] {
    let mut state = [[0u8; 4]; 4];
    for i in 0..16 {
        state[i / 4][i % 4] = block[i];
    }
    state
}

// Function that converts a state matrix (4x4) into a 16-byte block
pub fn state_to_block(state: &[[u8; 4]; 4]) -> [u8; 16] {
    let mut block = [0u8; 16];
    for i in 0..16 {
        block[i] = state[i / 4][i % 4];
    }
    block
}

// Function that applies the inverse SubBytes transformation to the state (for decryption)
pub fn inv_sub_bytes(state: &mut [[u8; 4]; 4]) {
    for r in 0..4 {
        for c in 0..4 {
            state[r][c] = INV_S_BOX[state[r][c] as usize];
        }
    }
}

// Function that performs the inverse ShiftRows transformation on the state
pub fn inv_shift_rows(state: &mut [[u8; 4]; 4]) {
    state[1].rotate_right(1); // Inverse shift of the second row
    state[2].rotate_right(2); // Inverse shift of the third row
    state[3].rotate_right(3); // Inverse shift of the fourth row
}

// Multiplication functions in GF(2^8) for the inverse MixColumns
fn mul9(x: u8) -> u8 {
    xtime(xtime(xtime(x))) ^ x
}
fn mul11(x: u8) -> u8 {
    xtime(xtime(xtime(x)) ^ x) ^ x
}
fn mul13(x: u8) -> u8 {
    xtime(xtime(xtime(x)) ^ xtime(x)) ^ x
}
fn mul14(x: u8) -> u8 {
    xtime(xtime(xtime(x)) ^ xtime(x) ^ x)
}

// Generic multiplication in GF(2^8) based on a given coefficient
fn mul_gf8(x: u8, coef: u8) -> u8 {
    match coef {
        0x01 => x,
        0x02 => xtime(x),
        0x03 => xtime(x) ^ x,
        0x09 => mul9(x),
        0x0B => mul11(x),
        0x0D => mul13(x),
        0x0E => mul14(x),
        _ => 0,
    }
}

// Function that performs the inverse MixColumns transformation on the state (for decryption)
pub fn inv_mix_columns(state: &mut [[u8; 4]; 4]) {
    for c in 0..4 {
        let a0 = state[0][c];
        let a1 = state[1][c];
        let a2 = state[2][c];
        let a3 = state[3][c];

        let r0 = mul_gf8(a0, 0x0E) ^ mul_gf8(a1, 0x0B) ^ mul_gf8(a2, 0x0D) ^ mul_gf8(a3, 0x09);
        let r1 = mul_gf8(a0, 0x09) ^ mul_gf8(a1, 0x0E) ^ mul_gf8(a2, 0x0B) ^ mul_gf8(a3, 0x0D);
        let r2 = mul_gf8(a0, 0x0D) ^ mul_gf8(a1, 0x09) ^ mul_gf8(a2, 0x0E) ^ mul_gf8(a3, 0x0B);
        let r3 = mul_gf8(a0, 0x0B) ^ mul_gf8(a1, 0x0D) ^ mul_gf8(a2, 0x09) ^ mul_gf8(a3, 0x0E);

        state[0][c] = r0;
        state[1][c] = r1;
        state[2][c] = r2;
        state[3][c] = r3;
    }
}

// Function that decrypts a 16-byte block using the round keys
pub fn decrypt_block(block: &[u8], round_keys: &Vec<[[u8; 4]; 4]>) -> [u8; 16] {
    let nr = 14; // Number of rounds for AES-256
    let mut state = block_to_state(block);

    // Final round (without mix_columns)
    add_round_key(&mut state, &round_keys[nr]);
    inv_shift_rows(&mut state);
    inv_sub_bytes(&mut state);

    // Intermediate rounds
    for round in (1..nr).rev() {
        add_round_key(&mut state, &round_keys[round]);
        inv_mix_columns(&mut state);
        inv_shift_rows(&mut state);
        inv_sub_bytes(&mut state);
    }

    // First round
    add_round_key(&mut state, &round_keys[0]);

    state_to_block(&state)
}

// Function that removes PKCS#7 padding from a vector of bytes
pub fn pkcs7_unpad(data: &mut Vec<u8>) -> Result<(), String> {
    if data.is_empty() {
        return Err("Empty data, cannot remove padding".to_string());
    }
    let pad_len = *data.last().unwrap() as usize;
    if pad_len == 0 || pad_len > data.len() {
        return Err("Invalid PKCS#7 padding".to_string());
    }
    let start = data.len() - pad_len;
    if data[start..].iter().any(|&x| x as usize != pad_len) {
        return Err("Invalid PKCS#7 padding (incorrect bytes)".to_string());
    }
    data.truncate(data.len() - pad_len);
    Ok(())
}

// Function that adds PKCS#7 padding to a vector of bytes to reach a given block size
pub fn pkcs7_pad(data: &mut Vec<u8>, block_size: usize) {
    let pad_len = block_size - (data.len() % block_size);
    data.extend(std::iter::repeat(pad_len as u8).take(pad_len));
}

/**
 * Decrypts the content of a file using AES-256 decryption.
 *
 * @param data - The ciphertext data to decrypt.
 * @param key - The decryption key.
 * @return Result<Vec<u8>, String> - The decrypted plaintext or an error message.
 */
pub fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    // Generate round keys using the provided key
    let round_keys = key_expansion(key);

    // Prepare the output buffer for plaintext
    let mut plaintext = Vec::new();

    // Process the data in 16-byte blocks
    let mut buffer = [0u8; 16];
    let mut i = 0;

    while i < data.len() {
        // Copy a block of data into the buffer
        let block_size = std::cmp::min(16, data.len() - i);
        buffer[..block_size].copy_from_slice(&data[i..i + block_size]);

        // Decrypt the block
        let decrypted_block = decrypt_block(&buffer, &round_keys);
        plaintext.extend_from_slice(&decrypted_block);

        // Move to the next block
        i += 16;
    }

    // Remove PKCS#7 padding from the last block
    pkcs7_unpad(&mut plaintext)?;

    Ok(plaintext)
}
