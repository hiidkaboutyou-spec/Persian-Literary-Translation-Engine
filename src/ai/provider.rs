pub trait AiProvider {
    fn name(&self) -> &str;

    fn translate(&self, context: &str, input: &str) -> Result<String, String>;
}

pub struct ProviderRequest {
    pub context: String,
    pub text: String,
}
