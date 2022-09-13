use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut builder = blocktastic::builder::BlockChainBuilder::new(blocktastic::Network::MainNet);
    for arg in env::args().skip(1) {
        let mut file = File::open(&arg).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        println!("Ingesting blocks in {:?}...", arg);
        let ingested_bytes = builder.ingest(&bytes);
        if ingested_bytes != bytes.len() {
            println!("Unable to ingest all {} bytes in {:?}, stopped at index {}", bytes.len(), arg, ingested_bytes);
            return;
        }
        println!("Fully ingested {:?}", arg);
    }
    builder.shutdown();
}
