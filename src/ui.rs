use arboard::Clipboard;
use inquire::{error::InquireError, Select, Password, Text, validator::{StringValidator, Validation, ErrorMessage}, CustomUserError};
use crossterm::event::{poll, read, Event, KeyEvent};
use std::time::{Duration, Instant};
use regex::Regex;
use crate::gpg_helpers::KeyIdName;

pub fn pick_password(password_list: Vec<String>) -> Result<String, InquireError> {
    let key = Select::new("Pick a key", password_list)
        .prompt()?;
    return Ok(key);
}

pub fn prompt_for_password() -> Result<String, InquireError> {
    let password = Password::new("Password: ")
        .prompt()?;
    return Ok(password);
}

pub fn prompt_for_otpauth() -> Result<String, InquireError> {
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

pub fn prompt_for_filename() -> Result<String, InquireError> {
    let filename = Text::new("Filename: ")
        .with_validator(FilenameValidator{})
        .prompt()?;
    return Ok(filename);
}


pub fn copy_to_clipboard_and_wait(text: &str) {
    let mut clipboard = Clipboard::new().unwrap();
    println!("Copied to the clipboard.");
    clipboard.set_text(text).unwrap();

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
    println!("Done.");
    clipboard.clear().unwrap();

}

#[derive(Clone)]
struct FilenameValidator{}

impl StringValidator for FilenameValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        let re = Regex::new(r"^[a-zA-Z0-9_\- ]+$").unwrap();
        if input.contains("/") {
            return Ok(Validation::Invalid(ErrorMessage::Custom("Filename cannot contain slashes.".to_string())));
        }
        if !re.is_match(input) {
            return Ok(Validation::Invalid(ErrorMessage::Custom("Filename can only contain alphanumeric characters, spaces, underscores and dashes.".to_string())));
        }
        return Ok(Validation::Valid);
    }
}

pub fn pick_key(keys: Vec<KeyIdName>) -> Result<String, InquireError> {
    let key = Select::new("Pick a key", keys)
        .prompt()?;
    return Ok(key.id);
}


