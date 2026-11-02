use super::message_builder::MessageBuilder;

pub trait BuildEmailHelper {
    fn build_email(&self, _builder: &MessageBuilder) -> Result<(), String> {
        // This handles ActionMailer integration which requires ruby callbacks
        // like `headers()` and `mail()`. We will stub it.
        todo!("ActionMailer integration");
    }
}
