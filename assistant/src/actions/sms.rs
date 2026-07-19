use super::contacts::find_contact;
use super::termux_command;

pub async fn send_message(contact: &str, message: &str) -> String {
    if message.is_empty() {
        return "What message should I send?".to_string();
    }

    // Resolve contact name to phone number
    let found = find_contact(contact).await;
    let number = match found {
        Some(c) => c.number,
        None => {
            return format!("I couldn't find a contact named '{contact}' in your contacts.");
        }
    };

    let result = termux_command("termux-sms-send", &["-n", &number, message]).await;

    if result.is_empty() || !result.starts_with("failed") {
        format!("Message sent to {contact}: \"{message}\"")
    } else {
        format!("Failed to send message to {contact}: {result}")
    }
}
