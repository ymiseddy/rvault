use std::{process::{Command, Stdio}, path::PathBuf};
use std::fmt::Display;
use std::io::Write;
use walkdir::WalkDir;
use crate::errors::VaultError;

#[derive(Debug, Clone)]
pub struct KeyIdName {
    pub id: String,
    pub name: String
}

impl Display for KeyIdName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.id, self.name)
    }
}

pub fn decrypt_file(filename: &PathBuf, password: Option<String>) -> Result<String, VaultError> {
    let mut cmd = Command::new("gpg");

     cmd.arg("--decrypt");

    if let Some(phrase) = password {
        cmd.arg("--batch")
            .arg("--pinentry-mode")
            .arg("loopback")
            .arg("--passphrase")
            .arg(phrase);
    } 
    let res = cmd.arg(filename)
                 .output();
    let output = match res {
        Ok(output) => output,
        Err(err) => {
            return Err(VaultError::IoError(err));
        },
    };
    if output.status.code().unwrap() != 0 {
        eprintln!("Error: {}", String::from_utf8(output.stderr).unwrap());
        return Err(VaultError::DecryptionError("Failed to decrypt file."));
    }

    let message = String::from_utf8(output.stdout).unwrap();
    return Ok(message);
}

pub fn get_secret_keys() -> Result<Vec<KeyIdName>, VaultError> {
    let output = Command::new("gpg")
        .arg("--list-secret-keys")
        .arg("--with-colons")
        .output()?;
    let mut keys: Vec<KeyIdName> = Vec::new();
    
    let mut initial = true;
    let mut current_key = KeyIdName {
        id: "".to_string(),
        name: "".to_string()
    };
    
    output.stdout.split(|&x| x == b'\n').for_each(|x| {
        let line = match String::from_utf8(x.to_vec()) {
            Ok(line) => line,
            Err(_) => return
        };

        if line.starts_with("sec") {
            match initial {
                true => initial = false,
                false => {
                    keys.push(current_key.to_owned());
                    current_key.id.clear();
                    current_key.name.clear();
                }
            }
            let parts = line.split(':').collect::<Vec<&str>>();
            let id = match parts.get(4) {
                Some(id) => id,
                None => return
            };
            current_key.id = id.to_string();
        }

        if line.starts_with("uid") {
            let parts = line.split(':').collect::<Vec<&str>>();
            let name = match parts.get(9) {
                Some(name) => name,
                None => return
            };
            if !current_key.name.is_empty() {
                current_key.name.push_str(", ");
            }
            current_key.name.push_str(name);
        }
    });

    // Push the last key
    if !initial {
        keys.push(current_key);
    } 

    return Ok(keys);
}

pub fn get_vault_dir() -> Option<PathBuf> {
    let home_dir = dirs::home_dir()?;
    let vault_dir = home_dir.join(".vault");
    return Some(vault_dir);
}

pub fn list_passwords(vault_dir: &PathBuf) -> Vec<String> {
    let mut keys = Vec::new();
    for entry in WalkDir::new(vault_dir)
        .follow_links(false)
        .into_iter()
        .filter(|e| e.is_ok()) {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let filename = path.to_str().unwrap();
                if filename.ends_with(".gpg") {
                    let key = filename
                        .trim_start_matches(vault_dir.to_str().unwrap())
                        .trim_start_matches('/')
                        .trim_start_matches('\\')
                        .trim_end_matches(".gpg");
                    keys.push(key.to_string());
                }
            }
        }


    return keys;
}

pub fn write_password(path: &PathBuf, password: &str) -> Result<(), VaultError> {
    let recipient="ymiseddy@gmail.com";
    let mut process = Command::new("gpg")
        .arg("--batch")
        .arg("--encrypt")
        .arg("--recipient")
        .arg(recipient)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdin = process.stdin.as_mut().unwrap();
    stdin.write_all(password.as_bytes())?;
    let output = process.wait_with_output()?;
    
    if output.status.code().unwrap() != 0 {
        eprintln!("Error: {}", String::from_utf8(output.stderr).unwrap());
        return Err(VaultError::DecryptionError("Failed to encrypt file."));
    }

    let mut file = std::fs::File::create(path)?;
    file.write_all(&output.stdout)?;

    return Ok(());
}






