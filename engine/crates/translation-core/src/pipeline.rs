#[derive(Debug, Clone)]
pub enum PipelineStage {
    DocumentAnalysis,
    ContextBuilding,
    Translation,
    QualityReview,
    Export,
}

#[derive(Debug, Clone)]
pub struct TranslationPipeline {
    pub stages: Vec<PipelineStage>,
}

impl TranslationPipeline {
    pub fn default_literary_pipeline() -> Self {
        Self {
            stages: vec![
                PipelineStage::DocumentAnalysis,
                PipelineStage::ContextBuilding,
                PipelineStage::Translation,
                PipelineStage::QualityReview,
                PipelineStage::Export,
            ],
        }
    }
}
