use clap::{Parser, Subcommand, ValueEnum};
use evm_engine::crypto;
use evm_engine::matcher::{self, Position};
use rayon::prelude::*;
use std::process::exit;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use colored::*;

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
    /// Update the CLI to the latest version from GitHub
    UpLatest,
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

fn check_for_updates() {
    let current_version = env!("CARGO_PKG_VERSION");
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("Tunic-onchain-name")
        .repo_name("tunic-engine")
        .build()
        .map(|r| r.fetch());

    if let Ok(Ok(releases)) = releases {
        if let Some(latest) = releases.first() {
            if self_update::version::bump_is_greater(current_version, &latest.version)
                .unwrap_or(false)
            {
                eprintln!(
                    "\n{}\n",
                    "Your device is running an older version. The latest version is now available. Please update to the latest version by running `tunic up-latest`.".yellow()
                );
            }
        }
    }
}

fn handle_update() -> Result<(), Box<dyn std::error::Error>> {
    let target = self_update::get_target();
    let asset_name = match target {
        t if t.contains("apple-darwin") => "tunic-macos",
        t if t.contains("linux") => "tunic-linux",
        t if t.contains("windows") => "tunic-windows.exe",
        _ => return Err(format!("Unsupported target: {}", target).into()),
    };

    let status = self_update::backends::github::Update::configure()
        .repo_owner("Tunic-onchain-name")
        .repo_name("tunic-engine")
        .bin_name("tunic")
        .target(asset_name)
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;
    println!("Update status: `{}`!", status.version());
    Ok(())
}

fn main() {
    check_for_updates();
    let cli = Cli::parse();

    match cli.command {
        Commands::UpLatest => {
            if let Err(e) = handle_update() {
                eprintln!("{}: {}", "Error updating".red(), e);
                exit(1);
            }
        }
        Commands::Generate {
            mode,
            prefix,
            suffix,
            combine,
        } => {
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
