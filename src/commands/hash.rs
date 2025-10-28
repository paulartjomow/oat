use seahorse::Command;
use sha2::{Sha256, Sha512, Digest};
use md5::Md5;
use std::fs;

pub fn hash_command() -> Command {
    Command::new("hash")
        .description("Compute MD5/SHA hashes for text or files")
        .usage("oat hash [algorithm] [input] [options]")
        .command(
            Command::new("md5")
                .description("Generate MD5 hash from text or file")
                .usage("oat hash md5 [text] or oat hash md5 --file [filepath]")
                .action(|c| {
                    if c.args.is_empty() {
                        eprintln!("Error: Please provide text to hash or use --file flag");
                        return;
                    }
                    
                    if c.args[0] == "--file" {
                        if c.args.len() < 2 {
                            eprintln!("Error: Please provide a file path after --file");
                            return;
                        }
                        hash_file(&c.args[1], "md5");
                    } else {
                        let input = c.args.join(" ");
                        hash_text(&input, "md5");
                    }
                })
        )
        .command(
            Command::new("sha256")
                .description("Generate SHA-256 hash from text or file")
                .usage("oat hash sha256 [text] or oat hash sha256 --file [filepath]")
                .action(|c| {
                    if c.args.is_empty() {
                        eprintln!("Error: Please provide text to hash or use --file flag");
                        return;
                    }
                    
                    if c.args[0] == "--file" {
                        if c.args.len() < 2 {
                            eprintln!("Error: Please provide a file path after --file");
                            return;
                        }
                        hash_file(&c.args[1], "sha256");
                    } else {
                        let input = c.args.join(" ");
                        hash_text(&input, "sha256");
                    }
                })
        )
        .command(
            Command::new("sha512")
                .description("Generate SHA-512 hash from text or file")
                .usage("oat hash sha512 [text] or oat hash sha512 --file [filepath]")
                .action(|c| {
                    if c.args.is_empty() {
                        eprintln!("Error: Please provide text to hash or use --file flag");
                        return;
                    }
                    
                    if c.args[0] == "--file" {
                        if c.args.len() < 2 {
                            eprintln!("Error: Please provide a file path after --file");
                            return;
                        }
                        hash_file(&c.args[1], "sha512");
                    } else {
                        let input = c.args.join(" ");
                        hash_text(&input, "sha512");
                    }
                })
        )
        .command(
            Command::new("all")
                .description("Print MD5, SHA-256 and SHA-512 for input or file")
                .usage("oat hash all [text] or oat hash all --file [filepath]")
                .action(|c| {
                    if c.args.is_empty() {
                        eprintln!("Error: Please provide text to hash or use --file flag");
                        return;
                    }
                    
                    if c.args[0] == "--file" {
                        if c.args.len() < 2 {
                            eprintln!("Error: Please provide a file path after --file");
                            return;
                        }
                        hash_file_all(&c.args[1]);
                    } else {
                        let input = c.args.join(" ");
                        hash_text_all(&input);
                    }
                })
        )
}

fn hash_text(input: &str, algorithm: &str) {
    let hash = match algorithm {
        "md5" => {
            let mut hasher = Md5::new();
            hasher.update(input.as_bytes());
            hex::encode(hasher.finalize())
        },
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(input.as_bytes());
            hex::encode(hasher.finalize())
        },
        "sha512" => {
            let mut hasher = Sha512::new();
            hasher.update(input.as_bytes());
            hex::encode(hasher.finalize())
        },
        _ => {
            eprintln!("Unsupported algorithm: {}", algorithm);
            return;
        }
    };
    
    println!("{}: {}", algorithm.to_uppercase(), hash);
}

fn hash_file(filepath: &str, algorithm: &str) {
    let data = match fs::read(filepath) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filepath, e);
            return;
        }
    };
    
    let hash = match algorithm {
        "md5" => {
            let mut hasher = Md5::new();
            hasher.update(&data);
            hex::encode(hasher.finalize())
        },
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(&data);
            hex::encode(hasher.finalize())
        },
        "sha512" => {
            let mut hasher = Sha512::new();
            hasher.update(&data);
            hex::encode(hasher.finalize())
        },
        _ => {
            eprintln!("Unsupported algorithm: {}", algorithm);
            return;
        }
    };
    
    println!("{} ({}): {}", algorithm.to_uppercase(), filepath, hash);
}

fn hash_text_all(input: &str) {
    println!("Input: \"{}\"", input);
    println!("─────────────────────────────────────────────────");
    hash_text(input, "md5");
    hash_text(input, "sha256");
    hash_text(input, "sha512");
}

fn hash_file_all(filepath: &str) {
    let data = match fs::read(filepath) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filepath, e);
            return;
        }
    };
    
    println!("File: {}", filepath);
    println!("Size: {} bytes", data.len());
    println!("─────────────────────────────────────────────────");
    hash_file(filepath, "md5");
    hash_file(filepath, "sha256");
    hash_file(filepath, "sha512");
} 