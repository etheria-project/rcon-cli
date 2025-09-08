use clap::Parser;
use rcon_cli::{
    cli::{Cli, Commands, OutputFormatter},
    client::RconConfig,
    RconClient, RconError,
};
use std::io::{self, Write};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::info;

#[tokio::main]
async fn main() {
    let result = run().await;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Validate CLI arguments
    if let Err(e) = cli.validate() {
        eprintln!("Invalid arguments: {}", e);
        std::process::exit(1);
    }

    // Initialize logging
    if let Err(e) = rcon_cli::init_logging(cli.log_level()) {
        eprintln!("Failed to initialize logging: {}", e);
        // Continue anyway, logging is not critical
    }

    // Create output formatter
    let formatter = OutputFormatter::new(cli.format.clone(), cli.use_colors());

    // Parse the address, converting localhost to 127.0.0.1
    let address = cli
        .parse_address()
        .map_err(|e| {
            eprintln!("Invalid address: {}", e);
            std::process::exit(1);
        })
        .unwrap();

    // Create RCON configuration
    let config =
        RconConfig::new(address, cli.password.clone()).with_timeout(cli.timeout_duration());

    info!("Starting RCON CLI v{}", rcon_cli::VERSION);

    // Execute the appropriate command
    match &cli.command {
        Commands::Exec { command, show_time } => {
            execute_single_command(&config, command, *show_time, &formatter).await?;
        }
        Commands::Interactive {
            prompt,
            history,
            history_size,
        } => {
            run_interactive_mode(&config, prompt, *history, *history_size, &formatter).await?;
        }
        Commands::Ping { count, interval } => {
            run_ping_command(&config, *count, *interval, &formatter).await?;
        }
        Commands::Info { detailed } => {
            run_info_command(&config, *detailed, &formatter).await?;
        }
        Commands::Players { show_uuids } => {
            run_players_command(&config, *show_uuids, &formatter).await?;
        }
    }

    Ok(())
}

async fn execute_single_command(
    config: &RconConfig,
    command: &str,
    show_time: bool,
    formatter: &OutputFormatter,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = connect_with_retry(config, formatter).await?;

    let start_time = Instant::now();

    match client.execute_command(command).await {
        Ok(response) => {
            let formatted_response = formatter.format_response(&response);
            println!("{}", formatted_response);

            if show_time {
                let elapsed = start_time.elapsed();
                let time_info =
                    formatter.format_info(&format!("Executed in {:.2}ms", elapsed.as_millis()));
                eprintln!("{}", time_info);
            }
        }
        Err(e) => {
            let error_msg = formatter.format_error(&e.to_string());
            eprintln!("{}", error_msg);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn run_interactive_mode(
    config: &RconConfig,
    prompt: &str,
    _history: bool,
    _history_size: usize,
    formatter: &OutputFormatter,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = connect_with_retry(config, formatter).await?;

    println!(
        "{}",
        formatter
            .format_info("Entering interactive mode. Type 'quit', 'exit', or Ctrl+C to leave.")
    );

    loop {
        print!("{}", prompt);
        io::stdout().flush()?;

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                }

                if input == "quit" || input == "exit" {
                    break;
                }

                // Handle special commands
                match input {
                    "help" => {
                        show_interactive_help(formatter);
                        continue;
                    }
                    "status" => {
                        show_connection_status(&mut client, formatter).await;
                        continue;
                    }
                    "reconnect" => {
                        match reconnect(&mut client, config, formatter).await {
                            Ok(_) => {
                                println!("{}", formatter.format_info("Reconnected successfully"));
                            }
                            Err(e) => {
                                eprintln!("{}", formatter.format_error(&e.to_string()));
                            }
                        }
                        continue;
                    }
                    _ => {}
                }

                // Execute the command
                match client.execute_command(input).await {
                    Ok(response) => {
                        if !response.is_empty() {
                            let formatted_response = formatter.format_response(&response);
                            println!("{}", formatted_response);
                        }
                    }
                    Err(RconError::Network(_)) | Err(RconError::Disconnected) => {
                        eprintln!(
                            "{}",
                            formatter.format_error("Connection lost. Attempting to reconnect...")
                        );

                        match reconnect(&mut client, config, formatter).await {
                            Ok(_) => {
                                eprintln!(
                                    "{}",
                                    formatter.format_info("Reconnected. Retrying command...")
                                );

                                match client.execute_command(input).await {
                                    Ok(response) => {
                                        if !response.is_empty() {
                                            let formatted_response =
                                                formatter.format_response(&response);
                                            println!("{}", formatted_response);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("{}", formatter.format_error(&e.to_string()));
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "{}",
                                    formatter.format_error(&format!("Failed to reconnect: {}", e))
                                );
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", formatter.format_error(&e.to_string()));
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", formatter.format_error(&format!("Input error: {}", e)));
                break;
            }
        }
    }

    println!("{}", formatter.format_info("Goodbye!"));
    Ok(())
}

async fn run_ping_command(
    config: &RconConfig,
    count: u32,
    interval: u64,
    formatter: &OutputFormatter,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = connect_with_retry(config, formatter).await?;
    let interval_duration = Duration::from_secs(interval);

    println!(
        "{}",
        formatter.format_info(&format!("Pinging {} {} time(s)", config.address, count))
    );

    let mut successful_pings = 0;
    let mut total_time = Duration::ZERO;

    for i in 1..=count {
        let start_time = Instant::now();

        match client.ping().await {
            Ok(_) => {
                let elapsed = start_time.elapsed();
                total_time += elapsed;
                successful_pings += 1;

                let ping_info = format!("Ping {}: Connected in {:.2}ms", i, elapsed.as_millis());
                println!("{}", formatter.format_info(&ping_info));
            }
            Err(e) => {
                let error_msg = format!("Ping {}: Failed - {}", i, e);
                eprintln!("{}", formatter.format_error(&error_msg));
            }
        }

        if i < count {
            sleep(interval_duration).await;
        }
    }

    // Print summary
    let success_rate = (successful_pings as f64 / count as f64) * 100.0;
    let avg_time = if successful_pings > 0 {
        total_time.as_millis() as f64 / successful_pings as f64
    } else {
        0.0
    };

    let summary = format!(
        "Summary: {}/{} successful ({:.1}%), average: {:.2}ms",
        successful_pings, count, success_rate, avg_time
    );
    println!("{}", formatter.format_info(&summary));

    Ok(())
}

async fn run_info_command(
    config: &RconConfig,
    detailed: bool,
    formatter: &OutputFormatter,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = connect_with_retry(config, formatter).await?;

    // Get basic server info
    let commands = if detailed {
        vec!["list", "version", "seed", "difficulty", "gamerule"]
    } else {
        vec!["list", "version"]
    };

    for command in commands {
        match client.execute_command(command).await {
            Ok(response) => {
                let section_header =
                    formatter.format_info(&format!("=== {} ===", command.to_uppercase()));
                println!("{}", section_header);

                let formatted_response = formatter.format_response(&response);
                println!("{}", formatted_response);
                println!();
            }
            Err(e) => {
                let error_msg = format!("Failed to get {}: {}", command, e);
                eprintln!("{}", formatter.format_error(&error_msg));
            }
        }
    }

    Ok(())
}

async fn run_players_command(
    config: &RconConfig,
    _show_uuids: bool,
    formatter: &OutputFormatter,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = connect_with_retry(config, formatter).await?;

    match client.execute_command("list uuids").await {
        Ok(response) => {
            let formatted_response = formatter.format_response(&response);
            println!("{}", formatted_response);
        }
        Err(_) => {
            // Fallback to basic list command
            match client.execute_command("list").await {
                Ok(response) => {
                    let formatted_response = formatter.format_response(&response);
                    println!("{}", formatted_response);
                }
                Err(e) => {
                    let error_msg = formatter.format_error(&e.to_string());
                    eprintln!("{}", error_msg);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

async fn connect_with_retry(
    config: &RconConfig,
    formatter: &OutputFormatter,
) -> Result<RconClient, Box<dyn std::error::Error>> {
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY: Duration = Duration::from_secs(1);

    for attempt in 1..=MAX_RETRIES {
        match RconClient::connect(config.clone()).await {
            Ok(client) => {
                if attempt > 1 {
                    let success_msg = formatter.format_info("Connected successfully");
                    eprintln!("{}", success_msg);
                }
                return Ok(client);
            }
            Err(e) => {
                if attempt < MAX_RETRIES {
                    let retry_msg =
                        format!("Connection attempt {} failed: {}. Retrying...", attempt, e);
                    eprintln!("{}", formatter.format_error(&retry_msg));
                    sleep(RETRY_DELAY).await;
                } else {
                    return Err(e.into());
                }
            }
        }
    }

    unreachable!()
}

async fn reconnect(
    client: &mut RconClient,
    config: &RconConfig,
    _formatter: &OutputFormatter,
) -> Result<(), Box<dyn std::error::Error>> {
    *client = RconClient::connect(config.clone()).await?;
    Ok(())
}

async fn show_connection_status(client: &mut RconClient, formatter: &OutputFormatter) {
    let status = if client.is_connected().await {
        "Connected"
    } else {
        "Disconnected"
    };

    let status_msg = format!(
        "Connection status: {} ({})",
        status,
        client.server_address()
    );
    println!("{}", formatter.format_info(&status_msg));
}

fn show_interactive_help(formatter: &OutputFormatter) {
    let help_text = r#"
Interactive Mode Commands:
  help         Show this help message
  status       Show connection status
  reconnect    Reconnect to the server
  quit/exit    Leave interactive mode

Any other input will be sent as a command to the server.

Common Minecraft commands:
  list         Show online players
  time set day Set time to day
  weather clear Clear weather
  gamemode creative <player>  Set player to creative mode
  tp <player1> <player2>      Teleport player1 to player2
"#;

    println!("{}", formatter.format_info(help_text));
}
