use arboard::Clipboard;
use inquire::{Select, Password, Text, validator::{StringValidator, Validation, ErrorMessage}, CustomUserError};
use crossterm::event::{poll, read, Event, KeyEvent};
use std::time::{Duration, Instant};
use regex::Regex;
use crate::gpg_helpers::KeyIdName;
use crate::errors::VaultError;

pub fn pick_password(password_list: Vec<String>) -> Result<String, VaultError> {
    let key = Select::new("Pick a key", password_list)
        .prompt()?;
    return Ok(key);
}

pub fn prompt_for_password() -> Result<String, VaultError> {
    let password = Password::new("Password: ")
        .prompt()?;
    return Ok(password);
}

pub fn prompt_for_otpauth() -> Result<String, VaultError> {
    let otpauth = Text::new("otpauth: ")
        .prompt()?;
    return Ok(otpauth);
}

pub fn maybe_prompt_for_password(ask_password: bool) -> Option<String> {
    if ask_password {
        return Some(Password::new("Password: ")
                    .without_confirmation()
                    .prompt()
                    .expect("Failed to read password."));
    } else {
        return None;
    }
}

pub fn prompt_for_filename() -> Result<String, VaultError> {
    let filename = Text::new("Filename: ")
        .with_validator(FilenameValidator{})
        .prompt()?;
    return Ok(filename);
}


pub fn copy_to_clipboard_and_wait(text: &str) {
    let mut clipboard = match Clipboard::new() {
        Ok(clipboard) => clipboard,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return;
        }
    };
    match clipboard.set_text(text) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return;
        }
    }

    let start_time = Instant::now();
    let ten_secs = Duration::from_secs(10);

    println!("Copied to clipboard for 10 seconds - press enter to terminate early.");
    loop {
        // Calculate the remaining time
        let elapsed = Instant::now().duration_since(start_time);
        let remaining = if elapsed >= ten_secs {
            break;
        } else {
            ten_secs - elapsed
        };
        
        // Poll for a key event
        match poll(remaining) {
            Ok(true) => {
                let event = read().unwrap();
                if let Event::Key(KeyEvent { code: _, .. }) = event {
                    break;
                }
            }
            Ok(false) => {
                // Timeout, no key was pressed
                break;
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                break;
            }
        }
    }
    clipboard.clear().unwrap();
    println!("Done.");
}

#[derive(Clone)]
struct FilenameValidator{}

impl StringValidator for FilenameValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        let re = Regex::new(r"^[a-zA-Z0-9_\- ]+$").unwrap();
        if input.contains('/') {
            return Ok(Validation::Invalid(ErrorMessage::Custom("Filename cannot contain slashes.".to_string())));
        }
        if !re.is_match(input) {
            return Ok(Validation::Invalid(ErrorMessage::Custom("Filename can only contain alphanumeric characters, spaces, underscores and dashes.".to_string())));
        }
        return Ok(Validation::Valid);
    }
}

pub fn pick_key(keys: Vec<KeyIdName>) -> Result<String, VaultError> {
    let key = Select::new("Pick a key", keys)
        .prompt()?;
    return Ok(key.id);
}


