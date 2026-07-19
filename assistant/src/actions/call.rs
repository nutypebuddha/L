use super::contacts::find_contact;
use super::termux_command;

pub async fn call_contact(contact: &str) -> String {
    let found = find_contact(contact).await;
    let number = match found {
        Some(c) => c.number,
        None => {
            return format!("I couldn't find a contact named '{contact}' in your contacts.");
        }
    };

    let result = termux_command("termux-telephony-call", &[&number]).await;

    if result.is_empty() || !result.starts_with("failed") {
        format!("Calling {contact}...")
    } else {
        format!("Failed to call {contact}: {result}")
    }
}
