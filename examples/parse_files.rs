use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    for arg in env::args().skip(1) {
        let mut file = File::open(&arg).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        println!("Parsing {:?}...", arg);
        match blockparse::parse_blockfile(&bytes, Some(blockparse::Network::MAINNET)) {
            Ok(blocks) => println!("Success!\n{:#?}", blocks),
            Err(e) => println!("Error: {}", e),
        };
    }
}
