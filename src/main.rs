mod backend;
mod error_manager;

use crate::backend::aes_keys::decrypted_key::{decrypt_block, pkcs7_unpad};
use s4_vaultify::backend::aes_keys::crypted_key::*;
use s4_vaultify::backend::aes_keys::keys_password::*;
use std::{env, fs};

pub fn main() {}
