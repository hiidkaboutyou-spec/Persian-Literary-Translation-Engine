//! Literary Intelligence creates understanding.
//!
//! Translation consumes understanding.
//!
//! This crate represents literary meaning before translation begins.
//! It models narrative concepts such as characters, scenes, relationships,
//! literary rules, and translation decisions.
//!
//! This crate does not:
//! - translate text
//! - call AI providers
//! - store persistence data
//! - orchestrate workflows

pub mod errors;
pub mod models;
pub mod traits;

mod context_packet;
mod coreference_evidence;
mod manuscript_analyzer;
mod speaker_attribution;

pub use context_packet::{
    build_chapter_context_packet, build_chapter_context_packet_with_semantic,
    build_chapter_context_packet_with_semantic_and_coreference, manuscript_context_candidates,
    ChapterContextPacketBuild, ChapterContextPacketInput, NeighborContext,
};
pub use manuscript_analyzer::{
    AnalysisCanon, AnalysisConfig, DeterministicManuscriptAnalyzer, ManuscriptAnalyzer,
};
pub use models::manuscript_intelligence::*;

pub use speaker_attribution::{
    attribute_speakers, deterministic_speaker_context, AttributionMethod, QuoteSpan, QuoteStyle,
    SpeakerAttribution,
};

pub use coreference_evidence::{
    canonical_coreference_links, model_coreference_context,
    validate_response as validate_coreference_response, CanonicalCoreferenceLink,
    CoreferenceCluster, CoreferenceError, CoreferenceMention, CoreferenceRequest,
    CoreferenceResponse, CoreferenceSidecar, COREFERENCE_PROTOCOL_VERSION,
};
