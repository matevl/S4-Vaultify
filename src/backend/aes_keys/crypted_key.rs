// Importing the necessary libraries
use ring::pbkdf2; // For password-based key derivation (PBKDF2)
use sha2::{Digest, Sha256}; // For SHA256 hashing
use std::env; // For accessing environment variables
use std::fs; // For file management
use std::num::NonZeroU32; // For working with non-zero integers

// Definition of the S-Box used in AES for byte substitution
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

// Function that performs a circular rotation on a word (an array of 4 bytes)
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

// AES-256 key expansion function (256-bit key)
// Generates a vector of round keys as 4x4 matrices
pub fn key_expansion(key: &[u8]) -> Vec<[[u8; 4]; 4]> {
    const NK: usize = 8; // Number of words in the key (256 bits / 32 bits)
    const NR: usize = 14; // Number of rounds for AES-256
    const NB: usize = 4; // Number of columns in the state
    let total_words = NB * (NR + 1);
    let mut words: Vec<[u8; 4]> = Vec::with_capacity(total_words);

    // Initialize the first words with the initial key
    for i in 0..NK {
        words.push([key[4 * i], key[4 * i + 1], key[4 * i + 2], key[4 * i + 3]]);
    }

    // Expand the remaining words of the key
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

    // Transform the generated words into 4x4 matrices corresponding to round keys
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

// Function that applies the SubBytes transformation on the state (byte substitution)
pub fn sub_bytes(state: &mut [[u8; 4]; 4]) {
    for r in 0..4 {
        for c in 0..4 {
            state[r][c] = S_BOX[state[r][c] as usize];
        }
    }
}

// Function that performs the ShiftRows transformation on the state (row shifting)
pub fn shift_rows(state: &mut [[u8; 4]; 4]) {
    state[1].rotate_left(1); // Shift the second row one position to the left
    state[2].rotate_left(2); // Shift the third row two positions to the left
    state[3].rotate_left(3); // Shift the fourth row three positions to the left
}

// Function that multiplies a byte by 2 in GF(2^8)
pub fn xtime(x: u8) -> u8 {
    if x & 0x80 != 0 {
        (x << 1) ^ 0x1b
    } else {
        x << 1
    }
}

// Function that performs the MixColumns transformation on the state (mixing columns)
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

// Function that adds the round key to the state using XOR
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

// Function that converts a state (4x4 matrix) into a 16-byte block
pub fn state_to_block(state: &[[u8; 4]; 4]) -> [u8; 16] {
    let mut block = [0u8; 16];
    for i in 0..16 {
        block[i] = state[i / 4][i % 4];
    }
    block
}

// Function that encrypts a 16-byte block using the generated round keys
pub fn encrypt_block(block: &[u8], round_keys: &Vec<[[u8; 4]; 4]>) -> [u8; 16] {
    let nr = 14; // Number of rounds for AES-256
    let mut state = block_to_state(block);

    // First round: add the initial key
    add_round_key(&mut state, &round_keys[0]);

    // Intermediate rounds
    for round in 1..nr {
        sub_bytes(&mut state); // Byte substitution
        shift_rows(&mut state); // Row shifting
        mix_columns(&mut state); // Column mixing
        add_round_key(&mut state, &round_keys[round]); // Add the round key
    }

    // Final round (without mix_columns)
    sub_bytes(&mut state);
    shift_rows(&mut state);
    add_round_key(&mut state, &round_keys[nr]);

    state_to_block(&state)
}

// Function that adds PKCS#7 padding to a vector of bytes to reach the block size
pub fn pkcs7_pad(data: &mut Vec<u8>, block_size: usize) {
    let pad_len = block_size - (data.len() % block_size);
    data.extend(std::iter::repeat(pad_len as u8).take(pad_len));
}

// Assume the previous AES-related functions are included here...

/**
 * Encrypts the content of a file using AES-256 encryption.
 *
 * @param data - The plaintext data to encrypt.
 * @param key - The encryption key.
 * @return Vec<u8> - The encrypted ciphertext.
 */
pub fn encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    // Generate round keys using the provided key
    let round_keys = key_expansion(key);

    // Prepare the output buffer for ciphertext
    let mut ciphertext = Vec::new();

    // Process the data in 16-byte blocks
    let mut buffer = [0u8; 16];
    let mut i = 0;

    while i < data.len() {
        // Copy a block of data into the buffer
        let block_size = std::cmp::min(16, data.len() - i);
        buffer[..block_size].copy_from_slice(&data[i..i + block_size]);

        // Apply PKCS#7 padding if it's the last block
        if block_size < 16 {
            pkcs7_pad(&mut buffer.to_vec(), 16);
        }

        // Encrypt the block
        let encrypted_block = encrypt_block(&buffer, &round_keys);
        ciphertext.extend_from_slice(&encrypted_block);

        // Move to the next block
        i += 16;
    }

    ciphertext
}
