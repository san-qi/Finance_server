use crypto::buffer::{BufferResult, ReadBuffer, WriteBuffer};
use crypto::{aes, blockmodes, buffer, symmetriccipher};

// Encrypt a buffer with the given key and iv using
// AES-256/CBC/Pkcs encryption.
fn encrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
    let iv: [u8; 16] = [0; 16];
    // Create an encryptor instance of the best performing
    // type available for the platform.
    let mut encryptor =
        aes::cbc_encryptor(aes::KeySize::KeySize256, key, &iv, blockmodes::PkcsPadding);

    // Each encryption operation encrypts some data from
    // an input buffer into an output buffer. Those buffers
    // must be instances of RefReaderBuffer and RefWriteBuffer
    // (respectively) which keep track of how much data has been
    // read from or written to them.
    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    // Each encryption operation will "make progress". "Making progress"
    // is a bit loosely defined, but basically, at the end of each operation
    // either BufferUnderflow or BufferOverflow will be returned (unless
    // there was an error). If the return value is BufferUnderflow, it means
    // that the operation ended while wanting more input data. If the return
    // value is BufferOverflow, it means that the operation ended because it
    // needed more space to output data. As long as the next call to the encryption
    // operation provides the space that was requested (either more input data
    // or more output space), the operation is guaranteed to get closer to
    // completing the full operation - ie: "make progress".
    //
    // Here, we pass the data to encrypt to the enryptor along with a fixed-size
    // output buffer. The 'true' flag indicates that the end of the data that
    // is to be encrypted is included in the input buffer (which is true, since
    // the input data includes all the data to encrypt). After each call, we copy
    // any output data to our result Vec. If we get a BufferOverflow, we keep
    // going in the loop since it means that there is more work to do. We can
    // complete as soon as we get a BufferUnderflow since the encryptor is telling
    // us that it stopped processing data due to not having any more data in the
    // input buffer.
    loop {
        let result = encryptor.encrypt(&mut read_buffer, &mut write_buffer, true)?;

        // "write_buffer.take_read_buffer().take_remaining()" means:
        // from the writable buffer, create a new readable buffer which
        // contains all data that has been written, and then access all
        // of that data as a slice.
        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .map(|&i| i),
        );

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

// Decrypts a buffer with the given key and iv using
// AES-256/CBC/Pkcs encryption.
//
// This function is very similar to encrypt(), so, please reference
// comments in that function. In non-example code, if desired, it is possible to
// share much of the implementation using closures to hide the operation
// being performed. However, such code would make this example less clear.
fn decrypt(
    encrypted_data: &[u8],
    key: &[u8],
) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
    let iv: [u8; 16] = [0; 16];
    let mut decryptor =
        aes::cbc_decryptor(aes::KeySize::KeySize256, key, &iv, blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(encrypted_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true)?;
        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .map(|&i| i),
        );
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

fn convert_key(key: &str) -> [u8; 32] {
    let _key = key;
    let mut key = [0u8; 32];
    for ele in _key.as_bytes() {
        key.fill(*ele);
    }
    key
}

// There are no errors in the encryption process os that U can safely unwrap.
pub fn encrypt_str(message: &str, key: &str) -> Option<String> {
    let key = convert_key(key);
    let encrypted_data = encrypt(message.as_bytes(), &key).unwrap();
    let mut res = String::new();
    for ele in &encrypted_data {
        res += format!("{:03}", ele).as_str();
    }

    Some(res)
}

// The encryption process may cause errors due to key mismatch.
pub fn decrypt_str(message: &str, key: &str) -> Option<String> {
    let key = convert_key(key);

    let mut encrypted_data: Vec<u8> = Vec::new();
    for pos in 0..message.len() / 3 {
        let start_pos = pos * 3;
        let value = &message[start_pos..start_pos + 3].parse::<u8>().unwrap();
        encrypted_data.push(*value);
    }

    match decrypt(&encrypted_data, &key) {
        Ok(data) => String::from_utf8(data.to_owned()).map_or_else(|_| None, |it| Some(it)),
        Err(_) => None,
    }
}

#[cfg(test)]
mod test {
    use crate::util::cryp::{decrypt_str, encrypt_str};

    #[test]
    fn test1() {
        // In a real program, the key and iv may be determined
        // using some other mechanism. If a password is to be used
        // as a key, an algorithm like PBKDF2, Bcrypt, or Scrypt (all
        // supported by Rust-Crypto!) would be a good choice to derive
        // a password. For the purposes of this example, the key and
        // iv are just random values.

        let message = "12345+12343453535";
        let encrypted_data = encrypt_str(message, "1234567").unwrap();
        let decrypted_data = decrypt_str(&encrypted_data, "1111111");
        // println!("{}", decrypted_data);
        assert!(message == decrypted_data.unwrap());
    }

    #[test]
    fn test2() {
        let da1 = decrypt_str("090076046167197207034029157203252224225062000230093055114009217233036203149248181219219195083028", "1111111");
        let da2 = decrypt_str("090076046167197207034029157203252224225062000230093055114009217233036203149248181219219195083028", "1234567");
        // println!("{}", decrypted_data);
        assert!(da1 == da2);
    }
}
