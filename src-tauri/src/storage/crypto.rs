use anyhow::Result;
use keyring::Entry;

const SERVICE_NAME: &str = "database-structure-sync";

pub fn store_password(connection_id: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, connection_id)?;
    entry.set_password(password)?;
    Ok(())
}

pub fn get_password(connection_id: &str) -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, connection_id)?;
    let password = entry.get_password()?;
    Ok(password)
}

pub fn delete_password(connection_id: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, connection_id)?;
    entry.delete_credential()?;
    Ok(())
}
