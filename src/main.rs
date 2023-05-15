#![allow(clippy::needless_return)]

use std::path::PathBuf;
use clap::{Parser, Subcommand};
mod gpg_helpers;
mod ui;
mod commands;


#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {

    /// Use a specific vault path instead of the default
    #[arg(short, long, value_name = "Vault path")]
    vault: Option<PathBuf>,

    /// Ask for the password on from the terminal instead of relying on GPG agent
    #[arg(short, long)]
    ask_password: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}


#[derive(Subcommand)]
enum Commands {

    /// List all passwords in the vault
    #[clap(name = "list", alias = "l")]
    List { 
    },

    /// Initialize a new vault
    #[clap(name = "init", alias = "i")]
    Init {},

    /// Add a new password to the vault
    #[clap(name = "add", alias = "a")]
    Add {
        /// Name of the password - if not present, a prompt will be shown.
        name: Option<String>,
    },

    /// Remove a password from the vault
    #[clap(name = "remove", alias = "r")]
    Remove {
        /// The name of the password to remove. If not present, a prompt will be shown.
        name: Option<String>,
    },

    /// Copy a password to the clipboard
    #[clap(name = "clip", alias = "c")]
    Clip {
        /// The name of the password to copy. If not present, a prompt will be shown.
        name: Option<String>,
    },

    /// Show a password to stdout (not recommended)
    #[clap(name = "show", alias = "s")]
    Show {
        /// The name of the password to show. If not present, a prompt will be shown.
        name: Option<String>,
    },

    /// Add a one-time password shared token to the vault.
    #[clap(name = "otp")]
    Otp {
        /// specify a UUL otpauth://... or be prompted from stdin.
        share_token: Option<String>,
    },

}



fn main() {
    let args = Cli::parse();

    let vault = match args.vault {
        Some(vault) => vault,
        None => gpg_helpers::get_vault_dir().unwrap(),
    };

    let ask_password = args.ask_password;

    let mut config = commands::VaultConfig {
        vault: vault.to_owned(),
        ask_password,
        key_id: "".to_string(),
    };
    let id = commands::read_id(&config);
    
    match id {
        Ok(id) => {
            // TODO: This feels wrong. 
            config = commands::VaultConfig {
                vault,
                ask_password,
                key_id: id,
            };
        },
        Err(_) => 
            {
                // Vault is not initialized -- only valid command is Init
                
                // TODO: This whole block feels clunky - Need to figure out a better way.
                if let Some(command) = &args.command {
                    match command {
                        Commands::Init {} => {},
                        _ => {
                            println!("No key ID found. Please run `rvault init` first.");
                            std::process::exit(1);
                        }
                    }
                }

            }
        }
    
    match &args.command {
        Some(command) => match command {
            Commands::List {} => {
                commands::list(&config);
            },
            Commands::Add { name } => {
                commands::add(&config, name);
            },
            Commands::Remove { name } => {
                commands::remove(&config, name);
            },
            Commands::Clip { name } => {
                commands::copy(&config, name);
            },
            Commands::Show { name } => {
                commands::show(&config, name);
            },
            Commands::Otp { share_token } => {
                commands::otp(&config, share_token);
            },
            Commands::Init {} => {
                commands::init(&config);
            },
        },
        None => {
            println!("No command given");
        },

    }
}
