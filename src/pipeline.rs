pub enum PipelineStage {
    Analyze,
    Translate,
    Edit,
    QualityCheck,
    Export,
}

pub struct TranslationPipeline;

impl TranslationPipeline {
    pub fn stages() -> Vec<PipelineStage> {
        vec![
            PipelineStage::Analyze,
            PipelineStage::Translate,
            PipelineStage::Edit,
            PipelineStage::QualityCheck,
            PipelineStage::Export,
        ]
    }
}
