use seahorse::App;
use std::env;

mod commands;

use commands::hash::hash_command;
use commands::generate::generate_command;
use commands::currency::currency_command;
use commands::update::{update_command, check_auto_update};
use commands::password::password_command;
use commands::qr::qr_command;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    // Check for updates in the background (only once per day)
    let _ = check_auto_update().await;

    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("oat [name]")
        .command(generate_command())
        .command(hash_command())
        .command(currency_command())
        .command(update_command())
        .command(password_command())
        .command(qr_command());

    app.run(args);
}
