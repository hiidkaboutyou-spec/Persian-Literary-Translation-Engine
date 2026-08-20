pub trait Analyzer {
    fn analyze(&self, input: &str) -> String;
}

pub trait Translator {
    fn translate(&self, input: &str) -> String;
}

pub trait QualityChecker {
    fn check(&self, input: &str) -> Vec<String>;
}

pub trait Exporter {
    fn export(&self, input: &str, destination: &str) -> Result<(), String>;
}
