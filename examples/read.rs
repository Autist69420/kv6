#[cfg(feature = "kv6")]
use kv6::kv6::KV6Format;
use std::{io::{BufReader, Read}, fs::File};
use scroll::Pread;

#[cfg(feature = "kv6")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open("data/grenade.kv6").unwrap();
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();

    let data = buffer.pread::<KV6Format>(0).unwrap();
    println!("{:#?}", data);

    Ok(())
}

#[cfg(not(feature = "kv6"))]
fn main() {
    // Code that does not depend on the "kv6" feature
    println!("Run this with the kv6 feature.")
}
