use chacha20::cipher::*;
use chacha20::ChaCha20;

fn main() {
    chacha_test();
}

fn chacha_test() {
    // En Chacha20 la clé doit faire 32bytes et nonce 12bytes
    let key = [0x42; 32];
    let nonce = [0x24; 12];
    let twice = [0x69; 12];

    let mut text = String::from("Hello there");

    // Key and IV must be references to the `GenericArray` type.
    // Here we use the `Into` trait to convert arrays into it.
    let mut cipher = ChaCha20::new(&key.into(), &nonce.into());
    let mut cipher2 = ChaCha20::new(&key.into(), &twice.into());

    let buffer = unsafe{text.as_bytes_mut()};
    println!("Plain bytes : {:?}", buffer);
    
    // apply keystream (encrypt)
    cipher.apply_keystream(buffer);

    println!("Cipher : {:?}", buffer);

    //cipher.seek(0u32);

    // decrypt ciphertext by applying keystream again
    cipher2.apply_keystream(buffer);
    println!("Decipher : {:?}", buffer);

    let decypher = String::from_utf8_lossy(buffer);
    println!("Text : {:?}", decypher);

}