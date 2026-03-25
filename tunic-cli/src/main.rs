use clap::{Parser, Subcommand, ValueEnum};
use evm_engine::crypto;
use evm_engine::matcher::{self, Position};
use rayon::prelude::*;
use std::process::exit;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

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
    Generate {
        #[arg(short, long)]
        mode: Mode,

        #[arg(long)]
        prefix: Option<String>,

        #[arg(long)]
        suffix: Option<String>,

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

            let num_threads = rayon::current_num_threads();
            println!(
                "Searching using {} threads (all available cores)...",
                num_threads
            );

            let found = Arc::new(AtomicBool::new(false));
            let counter = Arc::new(AtomicU64::new(0));

            let found_reporter = Arc::clone(&found);
            let counter_reporter = Arc::clone(&counter);

            // Spawn progress reporter thread
            std::thread::spawn(move || {
                let start = Instant::now();
                let mut last_count = 0u64;
                loop {
                    std::thread::sleep(Duration::from_secs(3));
                    if found_reporter.load(Ordering::Relaxed) {
                        break;
                    }
                    let current_count = counter_reporter.load(Ordering::Relaxed);
                    let delta = current_count - last_count;
                    last_count = current_count;
                    let elapsed = start.elapsed().as_secs_f64();
                    let rate = delta as f64 / 3.0;

                    eprintln!(
                        "  {:.2}M iter/sec | {:.0} total | {:.1}s elapsed",
                        rate / 1_000_000.0,
                        current_count as f64,
                        elapsed
                    );
                }
            });

            let result = rayon::iter::repeat(())
                .find_map_any(|_| {
                    if found.load(Ordering::Relaxed) {
                        return None;
                    }

                    for _ in 0..1000 {
                        let (address_bytes, private_key_bytes) = crypto::generate_keypair_raw();
                        counter.fetch_add(1, Ordering::Relaxed);

                        let matched = match position {
                            Position::Prefix => matcher::matches_nibbles(
                                &address_bytes,
                                &prefix_nibbles,
                                Position::Prefix,
                            ),
                            Position::Suffix => matcher::matches_nibbles(
                                &address_bytes,
                                &suffix_nibbles,
                                Position::Suffix,
                            ),
                            Position::Combine => {
                                matcher::matches_nibbles(
                                    &address_bytes,
                                    &prefix_nibbles,
                                    Position::Prefix,
                                ) && matcher::matches_nibbles(
                                    &address_bytes,
                                    &suffix_nibbles,
                                    Position::Suffix,
                                )
                            }
                        };

                        if matched {
                            found.store(true, Ordering::Relaxed);
                            return Some(crypto::encode_result(&address_bytes, &private_key_bytes));
                        }
                    }

                    None
                });

            if let Some((private_key, address)) = result {
                println!("\nSuccess!");
                println!("Address: {}", address);
                println!("Private Key: {}", private_key);
            }
        }
    }
}
