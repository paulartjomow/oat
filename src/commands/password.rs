use seahorse::{Command, Context, Flag, FlagType};
use rand::Rng;
use std::collections::HashSet;

pub fn password_command() -> Command {
    Command::new("password")
        .description("Generate secure passwords with customizable rules")
        .usage("oat password [options]")
        .flag(
            Flag::new("length", FlagType::Int)
                .description("Password length (default: 12)")
                .alias("l"),
        )
        .flag(
            Flag::new("count", FlagType::Int)
                .description("Number of passwords to generate (default: 1)")
                .alias("c"),
        )
        .flag(
            Flag::new("no-uppercase", FlagType::Bool)
                .description("Exclude uppercase letters")
                .alias("nu"),
        )
        .flag(
            Flag::new("no-lowercase", FlagType::Bool)
                .description("Exclude lowercase letters")
                .alias("nl"),
        )
        .flag(
            Flag::new("no-numbers", FlagType::Bool)
                .description("Exclude numbers")
                .alias("nn"),
        )
        .flag(
            Flag::new("no-symbols", FlagType::Bool)
                .description("Exclude symbols")
                .alias("ns"),
        )
        .flag(
            Flag::new("symbols", FlagType::String)
                .description("Custom symbol set (overrides default symbols)")
                .alias("s"),
        )
        .flag(
            Flag::new("exclude", FlagType::String)
                .description("Characters to exclude from password")
                .alias("e"),
        )
        .flag(
            Flag::new("include", FlagType::String)
                .description("Additional characters to include")
                .alias("i"),
        )
        .flag(
            Flag::new("no-ambiguous", FlagType::Bool)
                .description("Exclude ambiguous characters (0, O, l, 1, I)")
                .alias("na"),
        )
        .action(password_action)
}

fn password_action(c: &Context) {
    let length = c.int_flag("length").unwrap_or(12) as usize;
    let count = c.int_flag("count").unwrap_or(1) as usize;
    let no_uppercase = c.bool_flag("no-uppercase");
    let no_lowercase = c.bool_flag("no-lowercase");
    let no_numbers = c.bool_flag("no-numbers");
    let no_symbols = c.bool_flag("no-symbols");
    let custom_symbols = c.string_flag("symbols").ok();
    let exclude_chars = c.string_flag("exclude").unwrap_or_default();
    let include_chars = c.string_flag("include").unwrap_or_default();
    let no_ambiguous = c.bool_flag("no-ambiguous");

    if length == 0 {
        eprintln!("Error: Password length must be greater than 0");
        return;
    }

    if count == 0 {
        eprintln!("Error: Password count must be greater than 0");
        return;
    }

    let config = PasswordConfig {
        length,
        include_uppercase: !no_uppercase,
        include_lowercase: !no_lowercase,
        include_numbers: !no_numbers,
        include_symbols: !no_symbols,
        custom_symbols,
        exclude_chars: exclude_chars.chars().collect(),
        include_chars: include_chars.chars().collect(),
        no_ambiguous,
    };

    match build_character_set(&config) {
        Ok(charset) => {
            if charset.is_empty() {
                eprintln!("Error: No characters available for password generation. Check your exclusion rules.");
                return;
            }

            for i in 0..count {
                let password = generate_password(&charset, length);
                if count == 1 {
                    println!("{}", password);
                } else {
                    println!("Password {}: {}", i + 1, password);
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

#[derive(Debug)]
struct PasswordConfig {
    length: usize,
    include_uppercase: bool,
    include_lowercase: bool,
    include_numbers: bool,
    include_symbols: bool,
    custom_symbols: Option<String>,
    exclude_chars: HashSet<char>,
    include_chars: HashSet<char>,
    no_ambiguous: bool,
}

fn build_character_set(config: &PasswordConfig) -> Result<Vec<char>, String> {
    let mut charset = HashSet::new();

    // Define character sets
    let uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let lowercase = "abcdefghijklmnopqrstuvwxyz";
    let numbers = "0123456789";
    let default_symbols = "!@#$%^&*()_+-=[]{}|;:,.<>?";
    let ambiguous_chars = "0Ol1I";

    // Add character sets based on configuration
    if config.include_uppercase {
        charset.extend(uppercase.chars());
    }

    if config.include_lowercase {
        charset.extend(lowercase.chars());
    }

    if config.include_numbers {
        charset.extend(numbers.chars());
    }

    if config.include_symbols {
        let symbols = config.custom_symbols.as_deref().unwrap_or(default_symbols);
        charset.extend(symbols.chars());
    }

    // Add custom include characters
    charset.extend(config.include_chars.iter());

    // Remove ambiguous characters if requested
    if config.no_ambiguous {
        for c in ambiguous_chars.chars() {
            charset.remove(&c);
        }
    }

    // Remove excluded characters
    for c in &config.exclude_chars {
        charset.remove(c);
    }

    // Validate that we have at least one character type if length > 1
    if config.length > 1 {
        let has_required = (config.include_uppercase && charset.iter().any(|c| c.is_ascii_uppercase()))
            || (config.include_lowercase && charset.iter().any(|c| c.is_ascii_lowercase()))
            || (config.include_numbers && charset.iter().any(|c| c.is_ascii_digit()))
            || (config.include_symbols && charset.iter().any(|c| !c.is_ascii_alphanumeric()));

        if !has_required && config.include_chars.is_empty() {
            return Err("No valid characters available for password generation".to_string());
        }
    }

    Ok(charset.into_iter().collect())
}

fn generate_password(charset: &[char], length: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut password = String::with_capacity(length);

    for _ in 0..length {
        let random_index = rng.gen_range(0..charset.len());
        password.push(charset[random_index]);
    }

    password
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_generation_length() {
        let charset: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
        let password = generate_password(&charset, 10);
        assert_eq!(password.len(), 10);
    }

    #[test]
    fn test_character_set_building() {
        let config = PasswordConfig {
            length: 8,
            include_uppercase: true,
            include_lowercase: true,
            include_numbers: false,
            include_symbols: false,
            custom_symbols: None,
            exclude_chars: HashSet::new(),
            include_chars: HashSet::new(),
            no_ambiguous: false,
        };

        let charset = build_character_set(&config).unwrap();
        assert!(charset.len() > 0);
        assert!(charset.iter().any(|c| c.is_ascii_uppercase()));
        assert!(charset.iter().any(|c| c.is_ascii_lowercase()));
        assert!(!charset.iter().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_exclude_characters() {
        let mut exclude_chars = HashSet::new();
        exclude_chars.insert('a');
        exclude_chars.insert('b');

        let config = PasswordConfig {
            length: 8,
            include_uppercase: false,
            include_lowercase: true,
            include_numbers: false,
            include_symbols: false,
            custom_symbols: None,
            exclude_chars,
            include_chars: HashSet::new(),
            no_ambiguous: false,
        };

        let charset = build_character_set(&config).unwrap();
        assert!(!charset.contains(&'a'));
        assert!(!charset.contains(&'b'));
        assert!(charset.contains(&'c'));
    }

    #[test]
    fn test_ambiguous_character_exclusion() {
        let config = PasswordConfig {
            length: 8,
            include_uppercase: true,
            include_lowercase: true,
            include_numbers: true,
            include_symbols: false,
            custom_symbols: None,
            exclude_chars: HashSet::new(),
            include_chars: HashSet::new(),
            no_ambiguous: true,
        };

        let charset = build_character_set(&config).unwrap();
        assert!(!charset.contains(&'0'));
        assert!(!charset.contains(&'O'));
        assert!(!charset.contains(&'l'));
        assert!(!charset.contains(&'1'));
        assert!(!charset.contains(&'I'));
    }
} 