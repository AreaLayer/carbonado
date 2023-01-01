use std::{fs::File, io::Write, path::PathBuf};

use anyhow::Result;
use carbonado::{
    constants::Format,
    decode, encode,
    fs::Header,
    utils::{encode_bao_hash, init_logging},
};
use ecies::utils::generate_keypair;
use log::{debug, info, trace};
use secp256k1::PublicKey;
use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);

#[test]
fn contract() -> Result<()> {
    init_logging();

    let input = "Hello world!".as_bytes();
    let (sk, pk) = generate_keypair();
    let format = Format::try_from(15)?;

    info!("Encoding input: {input:?}...");
    let (encoded, hash, encode_info) = encode(&pk.serialize(), input, 15)?;

    debug!("Encoding Info: {encode_info:#?}");
    assert_eq!(
        encoded.len() as u32,
        encode_info.bytes_verifiable,
        "Length of encoded bytes matches bytes_verifiable field"
    );

    let header = Header::new(
        &sk.serialize(),
        hash.as_bytes(),
        format,
        encode_info.bytes_verifiable,
        encode_info.padding,
    )?;
    trace!("Header: {header:?}");

    let header_bytes = header.to_vec();

    let file_path = PathBuf::from(format!("/tmp/{}.c15", encode_bao_hash(&hash)));
    info!("Writing test file to: {file_path:?}");
    let mut file = File::create(file_path)?;
    file.write_all(&header_bytes)?;
    file.write_all(&encoded)?;

    let header = Header::try_from(file)?;

    // pub pubkey: PublicKey,
    // pub hash: Hash,
    // pub signature: Signature,
    // pub format: Format,
    // pub verifiable_len: u32,
    // pub padding_len: u32,

    assert_eq!(
        header.pubkey,
        PublicKey::from_slice(&pk.serialize_compressed())?
    );
    assert_eq!(header.hash, hash);
    assert_eq!(header.format, format);
    assert_eq!(header.encoded_len, encode_info.bytes_verifiable);
    assert_eq!(header.padding_len, encode_info.padding);

    info!("Decoding Carbonado bytes");
    let decoded = decode(
        &sk.serialize(),
        hash.as_bytes(),
        &encoded,
        encode_info.padding,
        15,
    )?;

    assert_eq!(decoded, input, "Decoded output is same as encoded input");

    info!("All good!");

    Ok(())
}
