use std::error::Error;
use std::io::{self, Read};
#[cfg(any(feature = "bge", test))]
use std::{env, path::PathBuf};

use literary_review_engine::{AlignmentConfig, ALIGNMENT_SCHEMA_VERSION};
#[cfg(feature = "bge")]
use literary_review_engine::{align_embeddings, AlignmentInput, EmbeddedSpan};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AlignmentToolRequest {
    schema_version: u32,
    unit_id: String,
    source_segments: Vec<String>,
    target_segments: Vec<String>,
    #[serde(default)]
    config: AlignmentConfig,
}

#[cfg(any(feature = "bge", test))]
#[derive(Debug, Clone, Copy)]
enum Side {
    Source,
    Target,
}

#[cfg(any(feature = "bge", test))]
#[derive(Debug, Clone)]
struct SpanSpec {
    side: Side,
    start: usize,
    len: usize,
    text: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("literary-alignment-tool: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let request: AlignmentToolRequest = serde_json::from_str(&input)?;
    validate_request(&request)?;
    run_request(request)
}

#[cfg(feature = "bge")]
fn run_request(request: AlignmentToolRequest) -> Result<(), Box<dyn Error>> {
    let alignment_input = bge::embed_request(&request)?;
    let result = align_embeddings(&alignment_input)?;
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}

#[cfg(not(feature = "bge"))]
fn run_request(_request: AlignmentToolRequest) -> Result<(), Box<dyn Error>> {
    Err("this binary was built without BGE embeddings; rebuild with `--features bge`".into())
}

fn validate_request(request: &AlignmentToolRequest) -> Result<(), Box<dyn Error>> {
    if request.schema_version != ALIGNMENT_SCHEMA_VERSION {
        return Err(format!(
            "unsupported protocol version {}; expected {}",
            request.schema_version, ALIGNMENT_SCHEMA_VERSION
        )
        .into());
    }
    if request.unit_id.trim().is_empty() {
        return Err("unit_id must not be empty".into());
    }
    if request.source_segments.len() > request.config.max_segments
        || request.target_segments.len() > request.config.max_segments
    {
        return Err(format!(
            "segment count exceeds configured limit {}",
            request.config.max_segments
        )
        .into());
    }
    if request.config.max_block_size == 0 || request.config.max_block_size > 8 {
        return Err("max_block_size must be between 1 and 8".into());
    }
    if request
        .source_segments
        .iter()
        .chain(&request.target_segments)
        .any(|segment| segment.trim().is_empty())
    {
        return Err("source/target segments must not be empty".into());
    }
    let total_chars = request
        .source_segments
        .iter()
        .chain(&request.target_segments)
        .map(|segment| segment.chars().count())
        .sum::<usize>();
    if total_chars > 500_000 {
        return Err("alignment request exceeds the 500000-character safety limit".into());
    }
    Ok(())
}

#[cfg(any(feature = "bge", test))]
fn span_specs(segments: &[String], side: Side, max_block_size: usize) -> Vec<SpanSpec> {
    let mut spans = Vec::new();
    for start in 0..segments.len() {
        for len in 1..=max_block_size.min(segments.len() - start) {
            spans.push(SpanSpec {
                side,
                start,
                len,
                text: segments[start..start + len].join("\n"),
            });
        }
    }
    spans
}

#[cfg(any(feature = "bge", test))]
fn model_cache_dir() -> PathBuf {
    env::var_os("PERSIAN_TRANSLATOR_MODEL_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".tools/models/fastembed"))
}

#[cfg(feature = "bge")]
mod bge {
    use super::*;
    use fastembed::{Bgem3Embedding, Bgem3InitOptions, Bgem3Model};

    const MODEL_PROVENANCE: &str = "gpahal/bge-m3-onnx-int8 via fastembed 6.1.0";

    pub fn embed_request(request: &AlignmentToolRequest) -> Result<AlignmentInput, Box<dyn Error>> {
        let mut specs = span_specs(
            &request.source_segments,
            Side::Source,
            request.config.max_block_size,
        );
        specs.extend(span_specs(
            &request.target_segments,
            Side::Target,
            request.config.max_block_size,
        ));

        if specs.is_empty() {
            return Ok(AlignmentInput {
                schema_version: ALIGNMENT_SCHEMA_VERSION,
                unit_id: request.unit_id.clone(),
                embedding_model: MODEL_PROVENANCE.to_string(),
                source_count: request.source_segments.len(),
                target_count: request.target_segments.len(),
                source_spans: Vec::new(),
                target_spans: Vec::new(),
                config: request.config,
            });
        }

        let cache = model_cache_dir().join("bge-m3");
        let mut model = Bgem3Embedding::try_new(
            Bgem3InitOptions::new(Bgem3Model::BGEM3Q)
                .with_cache_dir(cache)
                .with_show_download_progress(false),
        )?;
        let texts = specs.iter().map(|spec| spec.text.as_str()).collect::<Vec<_>>();
        let output = model.embed(texts, Some(16))?;
        if output.dense.len() != specs.len() {
            return Err(format!(
                "BGE-M3 returned {} embeddings for {} requested spans",
                output.dense.len(),
                specs.len()
            )
            .into());
        }

        let mut source_spans = Vec::new();
        let mut target_spans = Vec::new();
        for (spec, embedding) in specs.into_iter().zip(output.dense) {
            let span = EmbeddedSpan {
                start: spec.start,
                len: spec.len,
                embedding,
            };
            match spec.side {
                Side::Source => source_spans.push(span),
                Side::Target => target_spans.push(span),
            }
        }

        Ok(AlignmentInput {
            schema_version: ALIGNMENT_SCHEMA_VERSION,
            unit_id: request.unit_id.clone(),
            embedding_model: MODEL_PROVENANCE.to_string(),
            source_count: request.source_segments.len(),
            target_count: request.target_segments.len(),
            source_spans,
            target_spans,
            config: request.config,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> AlignmentToolRequest {
        AlignmentToolRequest {
            schema_version: ALIGNMENT_SCHEMA_VERSION,
            unit_id: "chapter-1".into(),
            source_segments: vec!["He laughed.".into(), "She left.".into()],
            target_segments: vec!["خندید.".into(), "او رفت.".into()],
            config: AlignmentConfig {
                max_block_size: 2,
                ..Default::default()
            },
        }
    }

    #[test]
    fn span_generation_is_complete_and_bounded() {
        let request = request();
        let spans = span_specs(
            &request.source_segments,
            Side::Source,
            request.config.max_block_size,
        );
        assert_eq!(spans.len(), 3);
        assert!(matches!(spans[0].side, Side::Source));
        assert_eq!(spans[0].start, 0);
        assert_eq!(spans[0].len, 1);
        assert_eq!(spans[1].start, 0);
        assert_eq!(spans[1].len, 2);
        assert_eq!(spans[1].text, "He laughed.\nShe left.");
        assert_eq!(spans[2].start, 1);
        assert_eq!(spans[2].len, 1);
    }

    #[test]
    fn request_limits_apply_without_loading_a_model() {
        let mut request = request();
        request.config.max_segments = 1;
        assert!(validate_request(&request).is_err());
    }

    #[test]
    fn empty_segments_are_rejected_before_model_loading() {
        let mut request = request();
        request.target_segments[0].clear();
        assert!(validate_request(&request).is_err());
    }

    #[test]
    fn default_cache_matches_existing_semantic_retrieval_policy() {
        env::remove_var("PERSIAN_TRANSLATOR_MODEL_CACHE");
        assert_eq!(model_cache_dir(), PathBuf::from(".tools/models/fastembed"));
    }
}
