use inquire::InquireError;
use totp_rs::TOTP;
use std::path::PathBuf;
use crate::gpg_helpers;
use crate::ui;
use json;


#[derive(Debug)]
pub struct VaultConfig {
    pub vault: PathBuf,
    pub ask_password: bool,
    pub key_id: String,
}


pub fn list(config: &VaultConfig) {
    let available_secrets = gpg_helpers::list_passwords(&config.vault);
    for secret in available_secrets {
        println!("{}", secret);
    }
}


pub fn remove(config: &VaultConfig, name: &Option<String>) {
    let path = get_secret_path(name, config).expect("Failed to get secret path.");
    std::fs::remove_file(path).expect("Failed to remove file."); 
}


pub fn read_id(config: &VaultConfig) -> Result<String, std::io::Error> {
    // Ensure vault exists and is a directory
    if !config.vault.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Vault is not initialized."));
    }
    if !config.vault.is_dir() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Vault is not a directory."));
    }

    // Read ID file
    let id_path = config.vault.join(".rvault");
    let idjson = std::fs::read_to_string(id_path)?;
    let result = json::parse(&idjson);

    // If the result is ok
    if let Ok(result) = result {
        if result["id"].is_null() {  
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Vault is not initialized."));
        }

        let id = result["id"].to_string();
        return Ok(id);
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to parse ID file."));
    }
}


pub fn otp(_config: &VaultConfig, _filename: &Option<PathBuf>) {
    // Need to finish this.
    todo!();
}


pub fn init(config: &VaultConfig) {
    let vault = &config.vault;
    if !vault.exists() {
        std::fs::create_dir_all(vault).expect("Failed to create vault directory.");
    }

    let secret_keys = gpg_helpers::get_secret_keys().expect("Failed to get secret keys.");
    
    // Pick secret key to use
    let chosen_key = ui::pick_key(secret_keys).expect("Failed to pick secret key.");

    let mut id_config = json::JsonValue::new_object();
    id_config["id"] = chosen_key.into();

    let id_path = vault.join(".rvault");
    let idjson = id_config.dump();
    std::fs::write(id_path, idjson).expect("Failed to write ID file.");


    println!("Vault initialized.");
}


pub fn add(config: &VaultConfig, name: &Option<String>) {
    let name = match name {
        Some(name) => name.to_owned(),
        None => ui::prompt_for_filename().expect("Failed to prompt for filename."),
    };
    let path = config.vault.join(name + ".gpg");
    let pw = ui::prompt_for_password().expect("Failed to prompt for password.");
    gpg_helpers::write_password(&path, &pw).expect("Failed to write password.");
}


pub fn show(config: &VaultConfig, name: &Option<String>) {

    let pw = extract(name, config).expect("Failed to extract password.");
    println!("{}", pw);
}


pub fn copy(config: &VaultConfig, name: &Option<String>) {
    let pw = extract(name, config).expect("Failed to extract password.");
    ui::copy_to_clipboard_and_wait(&pw);
}


fn extract(name: &Option<String>, config: &VaultConfig) -> Result<String, InquireError> {
    let pw_path = get_secret_path(name, config)?;
    let gpg_password = ui::maybe_prompt_for_password(config.ask_password);
    let mut pw = gpg_helpers::decrypt_file(&pw_path, gpg_password).expect("Failed to decrypt password.");
    
    if pw.starts_with("otpauth://") {
        let otp = TOTP::from_url_unchecked(&pw).expect("Failed to parse OTP URL.");
        pw = otp.generate_current().expect("Failed to generate OTP code.");
    }

    Ok(pw)
}


fn get_secret_path(name: &Option<String>, config: &VaultConfig) -> Result<PathBuf, InquireError> {
    let pw_path: PathBuf;
    if let Some(name) = name {
        pw_path = config.vault.join(name.to_owned() + ".gpg");
    } else {
        let available_secrets = gpg_helpers::list_passwords(&config.vault);
        let pw_name = ui::pick_password(available_secrets)?;
        pw_path = config.vault.join(pw_name + ".gpg");
    }
    Ok(pw_path)
}


