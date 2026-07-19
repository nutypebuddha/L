use super::termux_command;

/// Get the full contact list from the device.
pub async fn list_contacts() -> Vec<Contact> {
    let output = termux_command("termux-contact-list", &[]).await;

    if output.is_empty() || output.starts_with("command failed") || output.starts_with("failed to")
    {
        return Vec::new();
    }

    // termux-contact-list outputs JSON array
    serde_json::from_str(&output).unwrap_or_default()
}

/// Find a contact by name (case-insensitive partial match).
pub async fn find_contact(name: &str) -> Option<Contact> {
    let contacts = list_contacts().await;
    let lower = name.to_lowercase();

    contacts
        .into_iter()
        .find(|c| c.name.to_lowercase().contains(&lower))
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Contact {
    pub name: String,
    pub number: String,
}
