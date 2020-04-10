// A note on encoding efficiency: 0.75 for Base64, 0.744 for Base62, 0.732 for Base58
// slatepack uses a modified Base58Check encoding to create armored slate payloads:
// 1. Take first four bytes of SHA256(SHA256(slate.as_bytes()))
// 2. Concatenate result of step 1 and slate.as_bytes()
// 3. Base58 encode bytes from step 2
// Finally add armor framing and space/newline formatting as desired

use sha2::{Digest, Sha256};
use std::io::Result;

// Framing and formatting for slate armor
static HEADER: &str = "BEGIN SLATEPACK. ";
static FOOTER: &str = ". END SLATEPACK.";
const WORD_LENGTH: usize = 15;

// slatepack takes a slate json string and returns an encoded armored slate string
pub fn slatepack(json_slate: &str) -> Result<String> {
    // Serialize the slate with the base58check encoding
    let encoded_slate = base58check(&json_slate)?;
    // Prettify the output for the armor payload
    let formatted_slate = format_slate(&encoded_slate)?;
    // Construct the armor framing and payload
    let armored_slate = format!("{}{}{}", HEADER, formatted_slate, FOOTER);
    Ok(armored_slate)
}

// MODIFIED Base58Check encoding for slate strings
pub fn base58check(slate: &str) -> Result<String> {
    // Serialize the slate json string to a vector of bytes
    let mut slate_bytes: Vec<u8> = slate.as_bytes().to_vec();
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
pub fn format_slate(slate: &str) -> Result<String> {
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
pub fn generate_check(payload: &Vec<u8>) -> Result<Vec<u8>> {
    let mut first_hash = Sha256::new();
    first_hash.input(payload);
    let mut second_hash = Sha256::new();
    second_hash.input(first_hash.result());
    let checksum = second_hash.result();
    let check_bytes: Vec<u8> = checksum[0..4].to_vec();
    Ok(check_bytes)
}

// TODO: checks that an armored slate has a valid checksum
//pub fn check_slatepack(armored_slate: &str) -> Result<()> {
//    Ok()
//}
