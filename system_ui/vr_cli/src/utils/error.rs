// Error handling utilities for the CLI
use anyhow::{Result, Context, anyhow};
use colored::Colorize;
use std::io::{self, Write};
use std::fmt::Display;

/// Standardized error handling for CLI commands
pub fn handle_error<E: Display>(err: E, context: &str) -> Result<()> {
    eprintln!("{}: {} - {}", "Error".red().bold(), context, err);
    Err(anyhow!("{}: {}", context, err))
}

/// Prompt the user for confirmation
pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    let default_str = if default { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", prompt, default_str);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let input = input.trim().to_lowercase();
    
    if input.is_empty() {
        return Ok(default);
    }
    
    match input.as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => {
            println!("Please answer with 'y' or 'n'");
            confirm(prompt, default)
        }
    }
}

/// Prompt the user for a password without echoing
pub fn prompt_password(prompt: &str) -> Result<String> {
    print!("{}: ", prompt);
    io::stdout().flush()?;
    
    let password = rpassword::read_password()?;
    
    if password.is_empty() {
        return Err(anyhow!("Password cannot be empty"));
    }
    
    Ok(password)
}

/// Format output based on the specified format
pub fn format_output<T: serde::Serialize>(data: &T, format: &str) -> Result<String> {
    match format.to_lowercase().as_str() {
        "json" => {
            serde_json::to_string_pretty(data).context("Failed to serialize to JSON")
        },
        "toml" => {
            toml::to_string(data).context("Failed to serialize to TOML")
        },
        _ => {
            Err(anyhow!("Unsupported output format: {}", format))
        }
    }
}

/// Print a success message
pub fn print_success(message: &str) {
    println!("{}: {}", "Success".green().bold(), message);
}

/// Print a warning message
pub fn print_warning(message: &str) {
    println!("{}: {}", "Warning".yellow().bold(), message);
}

/// Print an info message
pub fn print_info(message: &str) {
    println!("{}: {}", "Info".blue().bold(), message);
}

/// Print a debug message (only if debug mode is enabled)
pub fn print_debug(message: &str) {
    if log::log_enabled!(log::Level::Debug) {
        println!("{}: {}", "Debug".magenta().bold(), message);
    }
}

/// Print a section header
pub fn print_section(title: &str) {
    println!("\n{}", title.green().bold());
    println!("{}", "=".repeat(title.len()).green());
}

/// Create a progress bar
pub fn create_progress_bar(total: u64, message: &str) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new(total);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    pb.set_message(message.to_string());
    pb
}
