use clap::{Parser, Subcommand, ValueEnum};
use evm_engine::crypto;
use evm_engine::matcher::{self, Position};
use rayon::prelude::*;
use std::process::exit;

#[derive(Parser)]
#[command(name = "tunic")]
#[command(author = "Tunic Team", version = "0.1.0")]
#[command(about = "High-performance EVM vanity address generator", long_about = "A multi-threaded search engine for EVM-compatible vanity addresses.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a vanity address using brute-force
    Generate {
        /// Mode of generation: prefix, suffix, or combine
        #[arg(short, long)]
        mode: Mode,

        /// The prefix to start with (e.g. "0xdead")
        #[arg(long)]
        prefix: Option<String>,

        /// The suffix to end with (e.g. "beef")
        #[arg(long)]
        suffix: Option<String>,

        /// The prefix and suffix separated by a colon (e.g. "de:ad")
        #[arg(long)]
        combine: Option<String>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Prefix,
    Suffix,
    Combine,
}

fn decode_to_nibbles(s: &str) -> Vec<u8> {
    let clean_s = if s.starts_with("0x") { &s[2..] } else { s };
    // Basic validation before decoding
    if !clean_s.chars().all(|c| c.is_ascii_hexdigit()) {
        eprintln!("Error: '{}' is not a valid hex string", s);
        exit(1);
    }
    clean_s.chars()
        .map(|c| c.to_digit(16).expect("invalid hex in pattern") as u8)
        .collect()
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { mode, prefix, suffix, combine } => {
            let (prefix_nibbles, suffix_nibbles, position) = match mode {
                Mode::Prefix => {
                    let s = prefix.expect("Error: --prefix is required for Prefix mode");
                    (decode_to_nibbles(&s), vec![], Position::Prefix)
                }
                Mode::Suffix => {
                    let s = suffix.expect("Error: --suffix is required for Suffix mode");
                    (vec![], decode_to_nibbles(&s), Position::Suffix)
                }
                Mode::Combine => {
                    let s = combine.expect("Error: --combine is required for Combine mode (format prefix:suffix)");
                    let parts: Vec<&str> = s.split(':').collect();
                    if parts.len() != 2 {
                        eprintln!("Error: combine format must be prefix:suffix (e.g. --combine dead:beef)");
                        exit(1);
                    }
                    (decode_to_nibbles(parts[0]), decode_to_nibbles(parts[1]), Position::Combine)
                }
            };

            println!("Searching for vanity address (using all CPU cores)...");

            let result = rayon::iter::repeat(())
                .find_map_any(|_| {
                    let (address_bytes, private_key_bytes) = crypto::generate_keypair_raw();
                    
                    let matched = match position {
                        Position::Prefix => matcher::matches_nibbles(&address_bytes, &prefix_nibbles, Position::Prefix),
                        Position::Suffix => matcher::matches_nibbles(&address_bytes, &suffix_nibbles, Position::Suffix),
                        Position::Combine => {
                            matcher::matches_nibbles(&address_bytes, &prefix_nibbles, Position::Prefix)
                                && matcher::matches_nibbles(&address_bytes, &suffix_nibbles, Position::Suffix)
                        }
                    };

                    if matched {
                        Some(crypto::encode_result(&address_bytes, &private_key_bytes))
                    } else {
                        None
                    }
                });

            if let Some((private_key, address)) = result {
                println!("\nSuccess!");
                println!("Address: {}", address);
                println!("Private Key: {}", private_key);
            }
        }
    }
}
