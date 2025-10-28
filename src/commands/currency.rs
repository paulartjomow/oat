use seahorse::Command;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use futures::executor;

pub fn currency_command() -> Command {
    Command::new("currency")
        .description("Convert amounts, view rates, and list supported currencies")
        .usage("oat currency [subcommand]")
        .command(convert_command())
        .command(rates_command())
        .command(list_command())
}

fn convert_command() -> Command {
    Command::new("convert")
        .description("Convert an amount between two currencies")
        .usage("oat currency convert [amount] [from] [to]")
        .action(|c| {
            if c.args.len() != 3 {
                eprintln!("Usage: oat currency convert [amount] [from] [to]");
                eprintln!("Example: oat currency convert 100 USD EUR");
                return;
            }
            
            let amount: f64 = match c.args[0].parse() {
                Ok(amt) => amt,
                Err(_) => {
                    eprintln!("Error: Amount must be a valid number");
                    return;
                }
            };
            
            let from_currency = c.args[1].to_uppercase();
            let to_currency = c.args[2].to_uppercase();
            
            executor::block_on(async {
                convert_currency(amount, from_currency, to_currency).await;
            });
        })
}

fn rates_command() -> Command {
    Command::new("rates")
        .description("Show exchange rates relative to a base currency")
        .usage("oat currency rates [base_currency]")
        .action(|c| {
            let base_currency = if c.args.is_empty() {
                "USD".to_string()
            } else {
                c.args[0].to_uppercase()
            };
            
            executor::block_on(async {
                show_rates(base_currency).await;
            });
        })
}

fn list_command() -> Command {
    Command::new("list")
        .description("List supported currency codes")
        .usage("oat currency list")
        .action(|_| {
            executor::block_on(async {
                list_currencies().await;
            });
        })
}

#[derive(Deserialize)]
struct ExchangeRateResponse {
    base: String,
    rates: HashMap<String, f64>,
}



async fn convert_currency(amount: f64, from: String, to: String) {
    let client = Client::new();
    
    // Using exchangerate-api.com free tier (no API key required)
    let url = format!("https://api.exchangerate-api.com/v4/latest/{}", from);
    
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<ExchangeRateResponse>().await {
                    Ok(data) => {
                        if let Some(rate) = data.rates.get(&to) {
                            let converted_amount = amount * rate;
                            println!("💱 Currency Conversion");
                            println!("────────────────────────");
                            println!("{:.2} {} = {:.2} {}", amount, from, converted_amount, to);
                            println!("Exchange rate: 1 {} = {:.6} {}", from, rate, to);
                        } else {
                            eprintln!("❌ Currency '{}' not found or not supported", to);
                            eprintln!("Use 'oat currency list' to see supported currencies");
                        }
                    }
                    Err(_) => eprintln!("❌ Failed to parse exchange rate data"),
                }
            } else {
                eprintln!("❌ Failed to fetch exchange rates: HTTP {}", response.status());
                if response.status().as_u16() == 404 {
                    eprintln!("Currency '{}' might not be supported", from);
                }
            }
        }
        Err(e) => eprintln!("❌ Network error: {}", e),
    }
}

async fn show_rates(base_currency: String) {
    let client = Client::new();
    let url = format!("https://api.exchangerate-api.com/v4/latest/{}", base_currency);
    
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<ExchangeRateResponse>().await {
                    Ok(data) => {
                        println!("💰 Exchange Rates for {}", data.base);
                        println!("────────────────────────────────");
                        
                        // Sort currencies for better display
                        let mut rates: Vec<(&String, &f64)> = data.rates.iter().collect();
                        rates.sort_by_key(|&(currency, _)| currency);
                        
                        // Display major currencies first
                        let major_currencies = ["USD", "EUR", "GBP", "JPY", "CAD", "AUD", "CHF", "CNY"];
                        
                        println!("Major Currencies:");
                        for currency in &major_currencies {
                            if let Some(rate) = data.rates.get(*currency) {
                                if *currency != data.base {
                                    println!("  {} → {}: {:.6}", data.base, currency, rate);
                                }
                            }
                        }
                        
                        println!("\nOther Currencies:");
                        for (currency, rate) in &rates {
                            if !major_currencies.contains(&currency.as_str()) && *currency != &data.base {
                                println!("  {} → {}: {:.6}", data.base, currency, rate);
                            }
                        }
                        
                        println!("\n📊 Total currencies: {}", data.rates.len());
                    }
                    Err(_) => eprintln!("❌ Failed to parse exchange rate data"),
                }
            } else {
                eprintln!("❌ Failed to fetch exchange rates: HTTP {}", response.status());
                if response.status().as_u16() == 404 {
                    eprintln!("Currency '{}' might not be supported", base_currency);
                }
            }
        }
        Err(e) => eprintln!("❌ Network error: {}", e),
    }
}

async fn list_currencies() {
    let client = Client::new();
    
    // Using a different endpoint that provides currency codes and names
    let url = "https://api.exchangerate-api.com/v4/latest/USD";
    
    match client.get(url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<ExchangeRateResponse>().await {
                    Ok(data) => {
                        println!("🌍 Supported Currencies");
                        println!("──────────────────────────");
                        
                        let mut currencies: Vec<&String> = data.rates.keys().collect();
                        currencies.sort();
                        
                        // Add USD since it's the base currency and won't be in the rates
                        println!("USD - United States Dollar");
                        
                        // Display in columns for better readability
                        let mut count = 1;
                        for currency in currencies {
                            print!("{:<4}", currency);
                            if count % 10 == 0 {
                                println!();
                            } else {
                                print!(" ");
                            }
                            count += 1;
                        }
                        
                        if count % 10 != 1 {
                            println!();
                        }
                        
                        println!("\n📊 Total supported currencies: {}", data.rates.len() + 1);
                        println!("\n💡 Tip: Use 'oat currency convert 100 USD EUR' to convert currencies");
                        println!("💡 Tip: Use 'oat currency rates EUR' to see all rates for EUR");
                    }
                    Err(_) => eprintln!("❌ Failed to parse currency data"),
                }
            } else {
                eprintln!("❌ Failed to fetch currency list: HTTP {}", response.status());
            }
        }
        Err(e) => eprintln!("❌ Network error: {}", e),
    }
} 