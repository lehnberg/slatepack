// A note on encoding efficiency: 0.75 for Base64, 0.744 for Base62, 0.732 for Base58
// slatepack uses a modified Base58Check encoding to create armored slate payloads:
// 1. Take first four bytes of SHA256(SHA256(slate.as_bytes()))
// 2. Concatenate result of step 1 and slate.as_bytes()
// 3. Base58 encode bytes from step 2
// Finally add armor framing and space/newline formatting as desired

use regex::Regex;
use sha2::{Digest, Sha256};
use std::io::{ErrorKind, Result};
use std::str;
#[macro_use]
extern crate lazy_static;

// Framing and formatting for slate armor
static HEADER: &str = "BEGIN SLATEPACK. ";
static FOOTER: &str = ". END SLATEPACK.";
const WORD_LENGTH: usize = 15;

lazy_static! {
    static ref HEADER_REGEX: Regex = Regex::new(concat!(
        r"^[>\n\r\t ]*BEGIN[>\n\r\t ]+([a-zA-Z0-9]+[>\n\r\t ]+)",
        r"?SLATEPACK[>\n\r\t ]*$"
    ))
    .unwrap();
    static ref FOOTER_REGEX: Regex = Regex::new(concat!(
        r"^[>\n\r\t ]*END[>\n\r\t ]+([a-zA-Z0-9]+[>\n\r\t ]+)",
        r"?SLATEPACK[>\n\r\t ]*$"
    ))
    .unwrap();
    static ref WHITESPACE_LIST: [u8; 5] = [b'>', b'\n', b'\r', b'\t', b' '];
}

// Deprecated, use armor_string() or armor_bytes()
pub fn armor(slate: &str) -> Result<String> {
    Ok(armor_string(&slate)?)
}

// Takes a slate json string and returns an encoded armored slate string
pub fn armor_string(json_slate: &str) -> Result<String> {
    // Serialize the slate json string to bytes
    let slate_bytes = json_slate.as_bytes();
    // Armor the slate bytes
    let armored_slate = armor_bytes(&slate_bytes)?;
    Ok(armored_slate)
}

// Takes slate bytes (same as encoded in slate files) and returns armored string
pub fn armor_bytes(binary_slate: &[u8]) -> Result<String> {
    // Serialize the slate bytes with the base58check encoding
    let encoded_slate = base58check(&binary_slate)?;
    // Prettify the output for the armor payload
    let formatted_slate = format_slate(&encoded_slate)?;
    // Construct the armor framing and payload
    let armored_slate = format!("{}{}{}", HEADER, formatted_slate, FOOTER);
    Ok(armored_slate)
}

// Takes an armored slate string and returns slate bytes
// TODO: verify and sanitize array bounds
pub fn remove_armor(armor_slate: &str) -> Result<Vec<u8>> {
    // Convert the armored slate to bytes for parsing
    let raw_armor_bytes: Vec<u8> = armor_slate.as_bytes().to_vec();
    // Collect the bytes up to the first period, this is the header
    let header_bytes = &raw_armor_bytes
        .iter()
        .take_while(|byte| **byte != b'.')
        .cloned()
        .collect::<Vec<u8>>();
    // Verify the header...
    check_header(&header_bytes)?;
    // Get the length of the header
    let header_len = *&header_bytes.len() + 1;
    // Skip the length of the header to read for the payload until the next period
    let payload_bytes = &raw_armor_bytes[header_len as usize..]
        .iter()
        .take_while(|byte| **byte != b'.')
        .cloned()
        .collect::<Vec<u8>>();
    // Get length of the payload to check the footer framing
    let payload_len = *&payload_bytes.len();
    // Get footer bytes and verify them
    let consumed_bytes = header_len + payload_len + 1;
    let footer_bytes = &raw_armor_bytes[consumed_bytes as usize..]
        .iter()
        .take_while(|byte| **byte != b'.')
        .cloned()
        .collect::<Vec<u8>>();
    check_footer(&footer_bytes)?;
    // Clean up the payload bytes to be deserialized
    let clean_payload = &payload_bytes
        .iter()
        .filter(|byte| !WHITESPACE_LIST.contains(byte))
        .cloned()
        .collect::<Vec<u8>>();
    // Decode payload from base58
    let base_decode = bs58::decode(&clean_payload).into_vec().unwrap();
    let error_code = &base_decode[0..4];
    let slate_bytes = &base_decode[4..];
    // Make sure the error check code is valid for the slate data
    error_check(&error_code.to_vec(), &slate_bytes.to_vec())?;
    Ok(slate_bytes.to_vec())
}

// Takes an error check code and a slate binary and verifies that the code was generated from slate
fn error_check(error_code: &Vec<u8>, slate_bytes: &Vec<u8>) -> Result<()> {
    let new_check = generate_check(&slate_bytes)?;
    if error_code.iter().eq(new_check.iter()) {
        Ok(())
    } else {
        Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "Bad slate error code- some data was corrupted".to_string(),
        ))
    }
}

// Checks header framing bytes and returns an error if they are invalid
fn check_header(header: &Vec<u8>) -> Result<()> {
    let framing = str::from_utf8(&header).unwrap();
    if HEADER_REGEX.is_match(framing) {
        Ok(())
    } else {
        Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "Bad armor header".to_string(),
        ))
    }
}

// Checks footer framing bytes and returns an error if they are invalid
fn check_footer(footer: &Vec<u8>) -> Result<()> {
    let framing = str::from_utf8(&footer).unwrap();
    if FOOTER_REGEX.is_match(framing) {
        Ok(())
    } else {
        Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "Bad armor footer".to_string(),
        ))
    }
}

// MODIFIED Base58Check encoding for slate bytes
fn base58check(slate: &[u8]) -> Result<String> {
    // Serialize the slate json string to a vector of bytes
    let mut slate_bytes: Vec<u8> = slate.to_vec();
    // Get the four byte checksum for the slate binary
    let mut check_bytes: Vec<u8> = generate_check(&slate_bytes)?;
    // Make a new buffer and concatenate checksum bytes and slate bytes
    let mut slate_buf = Vec::new();
    slate_buf.append(&mut check_bytes);
    slate_buf.append(&mut slate_bytes);
    // Encode the slate buffer containing the slate binary and checksum bytes as Base58
    let b58_slate = bs58::encode(slate_buf).into_string();
    Ok(b58_slate)
}

// Adds human readable formatting to the slate payload for armoring
fn format_slate(slate: &str) -> Result<String> {
    let formatter = slate
        .chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if i != 0 && i % WORD_LENGTH == 0 {
                Some(' ')
            } else {
                None
            }
            .into_iter()
            .chain(std::iter::once(c))
        })
        .collect::<String>();
    Ok(formatter)
}

// Returns the first four bytes of a double sha256 hash of some bytes
fn generate_check(payload: &Vec<u8>) -> Result<Vec<u8>> {
    let mut first_hash = Sha256::new();
    first_hash.input(payload);
    let mut second_hash = Sha256::new();
    second_hash.input(first_hash.result());
    let checksum = second_hash.result();
    let check_bytes: Vec<u8> = checksum[0..4].to_vec();
    Ok(check_bytes)
}
