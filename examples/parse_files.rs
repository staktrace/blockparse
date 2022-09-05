use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    for arg in env::args().skip(1) {
        let mut file = File::open(&arg).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        println!("Parsing {:?}...", arg);
        match blocktastic::parse::parse_blockfile(&bytes) {
            Ok(blocks) => {
                println!("Parsed {} blocks", blocks.len());
                blocks.iter().for_each(|b| println!("{}", b));
            }
            Err(e) => println!("Error: {}", e),
        };
    }
}
