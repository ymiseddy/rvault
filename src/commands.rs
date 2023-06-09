use totp_rs::TOTP;
use std::path::PathBuf;
use std::io::Read;
use crate::gpg_helpers;
use crate::ui;
use crate::errors::VaultError;


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


pub fn read_id(config: &VaultConfig) -> Result<String, VaultError> {
    // Ensure vault exists and is a directory
    if !config.vault.exists() {
        return Err(VaultError::InitializationError("Vault is not initialized"));
    }
    if !config.vault.is_dir() {
        return Err(VaultError::InitializationError("Vault is not a directory."));
    }

    // Read ID file
    let id_path = config.vault.join(".rvault");
    let idjson = std::fs::read_to_string(id_path)?;
    let result = json::parse(&idjson);

    // If the result is ok
    if let Ok(result) = result {
        if result["id"].is_null() {  
            return Err(VaultError::InitializationError("Vault is not initialized."));
        }

        let id = result["id"].to_string();
        Ok(id)
    } else {
        Err(VaultError::InitializationError("Failed to parse ID file."))
    }
}


fn read_otpauth() -> Result<String, VaultError> {
    if atty::is(atty::Stream::Stdin) {
        // Prompt for otpauth
        let otpauth = ui::prompt_for_otpauth()?;
        Ok(otpauth)
    } else {
        // Read from stdin
        let mut otpauth = Vec::new();
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        handle.read_to_end(&mut otpauth)?;

        let strauth = match String::from_utf8(otpauth) {
            Ok(otpauth) => otpauth,
            Err(_) => {
                // TODO: Hacky way to return an error here.
                return Err(VaultError::Other("Failed to parse otpauth."));
            }
        };
        Ok(strauth)
    }
}

fn decode_image_file(image_file: &str) -> String {
    let img = image::open(image_file).expect("Failed to open image.");
    let decoder = bardecoder::default_decoder();
    let results = decoder.decode(&img);

    if results.is_empty() {
        panic!("Failed to decode image.");
    }

    results.into_iter().next()
                        .expect("Failed to decode image")
                        .expect("Failed to decode image")
}

pub fn otp(config: &VaultConfig, share_token: &Option<String>) {

    let mut otpauth: String = match share_token {
        Some(share_token) => share_token.to_string(),
        None => read_otpauth().expect("Failed to read otpauth."),
    };

    if otpauth.ends_with(".png") {
        // Decode as image.
        otpauth = decode_image_file(&otpauth);
    }

    let totp = TOTP::from_url_unchecked(&otpauth).expect("Failed to parse otpauth.");
    let issuer = totp.issuer.unwrap();
    let account = totp.account_name;
    let filename = format!("{account}.gpg");

    let path = config.vault.join("otp").join(issuer);

    let filepath = path.join(filename);

    if !path.exists() {
        std::fs::create_dir_all(&path).expect("Failed to create OTP directory.");
    }

    gpg_helpers::write_password(&filepath, &otpauth).expect("Failed to write password.");
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


fn extract(name: &Option<String>, config: &VaultConfig) -> Result<String, VaultError> {
    let pw_path = get_secret_path(name, config)?;
    let gpg_password = ui::maybe_prompt_for_password(config.ask_password);
    let mut pw =  gpg_helpers::decrypt_file(&pw_path, gpg_password)?;
    
    if pw.starts_with("otpauth://") {
        let otp = TOTP::from_url_unchecked(&pw).expect("Failed to parse OTP URL.");
        pw = otp.generate_current().expect("Failed to generate OTP code.");
    }

    Ok(pw)
}


fn get_secret_path(name: &Option<String>, config: &VaultConfig) -> Result<PathBuf, VaultError> {
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


