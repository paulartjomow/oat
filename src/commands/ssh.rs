use seahorse::{Command, Context, Flag, FlagType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command as StdCommand, Stdio};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SshConnection {
    name: String,
    user: String,
    host: String,
    port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    identity_file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SshConfig {
    connections: Vec<SshConnection>,
}

pub fn ssh_command() -> Command {
    Command::new("ssh")
        .description("SSH connection manager for saving and connecting to hosts")
        .usage("oat ssh [subcommand]")
        .command(add_command())
        .command(list_command())
        .command(connect_command())
        .command(remove_command())
        .command(edit_command())
}

fn add_command() -> Command {
    Command::new("add")
        .description("Add a new SSH connection profile")
        .usage("oat ssh add [options]")
        .flag(
            Flag::new("name", FlagType::String)
                .description("Connection name")
                .alias("n"),
        )
        .flag(
            Flag::new("user", FlagType::String)
                .description("SSH username")
                .alias("u"),
        )
        .flag(
            Flag::new("host", FlagType::String)
                .description("Host or IP address")
                .alias("h"),
        )
        .flag(
            Flag::new("port", FlagType::Int)
                .description("SSH port (default: 22)")
                .alias("p"),
        )
        .flag(
            Flag::new("identity-file", FlagType::String)
                .description("Path to SSH private key file")
                .alias("i"),
        )
        .action(add_action)
}

fn list_command() -> Command {
    Command::new("list")
        .description("List all saved SSH connections")
        .usage("oat ssh list")
        .action(list_action)
}

fn connect_command() -> Command {
    Command::new("connect")
        .description("Connect to a saved SSH host")
        .usage("oat ssh connect <name>")
        .action(connect_action)
}

fn remove_command() -> Command {
    Command::new("remove")
        .description("Remove a saved SSH connection")
        .usage("oat ssh remove <name>")
        .action(remove_action)
}

fn edit_command() -> Command {
    Command::new("edit")
        .description("Edit an existing SSH connection")
        .usage("oat ssh edit <name>")
        .action(edit_action)
}

fn get_config_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".oat")
}

fn get_config_file_path() -> PathBuf {
    get_config_path().join("ssh_config.json")
}

fn load_config() -> SshConfig {
    let config_path = get_config_file_path();

    if !config_path.exists() {
        return SshConfig {
            connections: Vec::new(),
        };
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error parsing config file: {}", e);
                SshConfig {
                    connections: Vec::new(),
                }
            }
        },
        Err(e) => {
            eprintln!("Error reading config file: {}", e);
            SshConfig {
                connections: Vec::new(),
            }
        }
    }
}

fn save_config(config: &SshConfig) -> Result<(), String> {
    let config_dir = get_config_path();
    
    // Create .oat directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let config_path = get_config_file_path();
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    Ok(())
}

fn prompt_input(message: &str, default: Option<&str>) -> String {
    print!("{} ", message);
    if let Some(d) = default {
        print!("[{}] ", d);
    }
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim().to_string();

    if trimmed.is_empty() {
        default.unwrap_or("").to_string()
    } else {
        trimmed
    }
}

fn add_action(c: &Context) {
    // Check if any flags are provided
    let has_name = c.string_flag("name").is_ok();
    let has_user = c.string_flag("user").is_ok();
    let has_host = c.string_flag("host").is_ok();

    let connection = if has_name || has_user || has_host {
        // Flag-based mode
        let name = c.string_flag("name").unwrap_or_else(|_| {
            eprintln!("Error: --name is required");
            String::new()
        });
        
        if name.is_empty() {
            return;
        }

        let user = c.string_flag("user").unwrap_or_else(|_| {
            eprintln!("Error: --user is required");
            String::new()
        });
        
        if user.is_empty() {
            return;
        }

        let host = c.string_flag("host").unwrap_or_else(|_| {
            eprintln!("Error: --host is required");
            String::new()
        });
        
        if host.is_empty() {
            return;
        }

        let port = c.int_flag("port").unwrap_or(22) as u16;
        let identity_file = c.string_flag("identity-file").ok();

        SshConnection {
            name,
            user,
            host,
            port,
            identity_file,
        }
    } else {
        // Interactive onboarding mode
        println!("Adding new SSH connection...\n");
        
        let name = loop {
            let input = prompt_input("Connection name:", None);
            if !input.is_empty() {
                break input;
            }
            println!("Connection name cannot be empty");
        };

        let user = loop {
            let input = prompt_input("SSH username:", None);
            if !input.is_empty() {
                break input;
            }
            println!("Username cannot be empty");
        };

        let host = loop {
            let input = prompt_input("Host or IP address:", None);
            if !input.is_empty() {
                break input;
            }
            println!("Host cannot be empty");
        };

        let port_input = prompt_input("SSH port:", Some("22"));
        let port = port_input.parse::<u16>().unwrap_or_else(|_| {
            eprintln!("Invalid port, using default 22");
            22
        });

        let identity_file_input = prompt_input("Identity file (optional):", Some("none"));
        let identity_file = if identity_file_input.is_empty() || identity_file_input == "none" {
            None
        } else {
            Some(identity_file_input)
        };

        SshConnection {
            name,
            user,
            host,
            port,
            identity_file,
        }
    };

    // Load existing config
    let mut config = load_config();

    // Check if connection with this name already exists
    if config.connections.iter().any(|c| c.name == connection.name) {
        println!("\nA connection with this name already exists.");
        print!("Do you want to overwrite it? (y/N): ");
        io::stdout().flush().unwrap();
        
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();
        
        if response.trim().to_lowercase() != "y" {
            println!("Cancelled.");
            return;
        }
        
        // Remove existing connection
        config.connections.retain(|c| c.name != connection.name);
    }

    // Add new connection
    config.connections.push(connection.clone());
    
    // Save config
    match save_config(&config) {
        Ok(_) => println!("\n✓ SSH connection '{}' added successfully!", connection.name),
        Err(e) => eprintln!("\nError saving connection: {}", e),
    }
}

fn list_action(_c: &Context) {
    let config = load_config();

    if config.connections.is_empty() {
        println!("No SSH connections saved.");
        return;
    }

    println!("Saved SSH connections:\n");
    for (i, conn) in config.connections.iter().enumerate() {
        println!("{}. {}", i + 1, conn.name);
        println!("   User: {}", conn.user);
        println!("   Host: {}", conn.host);
        println!("   Port: {}", conn.port);
        if let Some(ref id_file) = conn.identity_file {
            println!("   Identity: {}", id_file);
        }
        println!();
    }
}

fn connect_action(c: &Context) {
    if c.args.is_empty() {
        eprintln!("Error: Please provide a connection name");
        return;
    }

    let connection_name = &c.args[0];
    let config = load_config();

    let connection = config.connections.iter().find(|c| c.name == *connection_name);

    match connection {
        Some(conn) => {
            println!("Connecting to {}...\n", conn.name);
            
            let mut ssh_cmd = StdCommand::new("ssh");
            
            if let Some(ref id_file) = conn.identity_file {
                ssh_cmd.arg("-i").arg(id_file);
            }

            if conn.port != 22 {
                ssh_cmd.arg("-p").arg(conn.port.to_string());
            }

            let target = format!("{}@{}", conn.user, conn.host);
            ssh_cmd.arg(target);

            // Execute SSH in interactive mode
            ssh_cmd.stdin(Stdio::inherit());
            ssh_cmd.stdout(Stdio::inherit());
            ssh_cmd.stderr(Stdio::inherit());

            match ssh_cmd.status() {
                Ok(status) => {
                    if !status.success() {
                        eprintln!("\nSSH connection failed with exit code: {:?}", status.code());
                    }
                }
                Err(e) => {
                    eprintln!("Error executing SSH: {}", e);
                    eprintln!("Make sure SSH is installed and accessible in your PATH");
                }
            }
        }
        None => {
            eprintln!("Error: Connection '{}' not found", connection_name);
            println!("\nAvailable connections:");
            for conn in config.connections.iter() {
                println!("  - {}", conn.name);
            }
        }
    }
}

fn remove_action(c: &Context) {
    if c.args.is_empty() {
        eprintln!("Error: Please provide a connection name");
        return;
    }

    let connection_name = &c.args[0];
    let mut config = load_config();

    if !config.connections.iter().any(|c| c.name == *connection_name) {
        eprintln!("Error: Connection '{}' not found", connection_name);
        return;
    }

    print!("Are you sure you want to remove '{}'? (y/N): ", connection_name);
    io::stdout().flush().unwrap();
    
    let mut response = String::new();
    io::stdin().read_line(&mut response).unwrap();
    
    if response.trim().to_lowercase() == "y" {
        config.connections.retain(|c| c.name != *connection_name);
        
        match save_config(&config) {
            Ok(_) => println!("✓ SSH connection '{}' removed successfully!", connection_name),
            Err(e) => eprintln!("Error removing connection: {}", e),
        }
    } else {
        println!("Cancelled.");
    }
}

fn edit_action(c: &Context) {
    if c.args.is_empty() {
        eprintln!("Error: Please provide a connection name");
        return;
    }

    let connection_name = &c.args[0];
    let config = load_config();

    let existing_connection = config.connections.iter().find(|c| c.name == *connection_name);

    match existing_connection {
        Some(conn) => {
            println!("Editing SSH connection '{}'...\n", conn.name);
            println!("Press Enter to keep current value.\n");

            let name_input = prompt_input("Connection name:", Some(&conn.name));
            let name = if name_input.is_empty() { conn.name.clone() } else { name_input };

            let user_input = prompt_input("SSH username:", Some(&conn.user));
            let user = if user_input.is_empty() { conn.user.clone() } else { user_input };

            let host_input = prompt_input("Host or IP address:", Some(&conn.host));
            let host = if host_input.is_empty() { conn.host.clone() } else { host_input };

            let port_input = prompt_input("SSH port:", Some(&conn.port.to_string()));
            let port = if port_input.is_empty() {
                conn.port
            } else {
                port_input.parse::<u16>().unwrap_or_else(|_| {
                    eprintln!("Invalid port, keeping current value");
                    conn.port
                })
            };

            let identity_file = conn.identity_file.as_ref().map(|s| s.as_str()).unwrap_or("none");
            let id_file_input = prompt_input("Identity file:", Some(identity_file));
            let new_identity_file = if id_file_input.is_empty() || id_file_input == "none" {
                None
            } else {
                Some(id_file_input)
            };

            // Load fresh config to avoid borrow conflicts
            let mut config = load_config();
            
            // Remove old connection
            config.connections.retain(|c| c.name != *connection_name);
            
            // If name changed, handle potential conflicts
            if name != *connection_name && config.connections.iter().any(|c| c.name == name) {
                eprintln!("\nError: A connection with name '{}' already exists", name);
                // Restore original connection
                config.connections.push(conn.clone());
                return;
            }

            // Add updated connection
            let updated_connection = SshConnection {
                name: name.clone(),
                user,
                host,
                port,
                identity_file: new_identity_file,
            };

            config.connections.push(updated_connection);
            
            match save_config(&config) {
                Ok(_) => println!("\n✓ SSH connection '{}' updated successfully!", name),
                Err(e) => eprintln!("\nError updating connection: {}", e),
            }
        }
        None => {
            eprintln!("Error: Connection '{}' not found", connection_name);
            println!("\nAvailable connections:");
            for conn in config.connections.iter() {
                println!("  - {}", conn.name);
            }
        }
    }
}



