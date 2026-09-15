use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{EvidenceSource, LiteraryReviewReport, ReviewDimension, ReviewFinding, ReviewSeverity};

pub const ALIGNMENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AlignmentConfig {
    /// Maximum number of consecutive source or target segments in one matched block.
    pub max_block_size: usize,
    /// Cost for leaving one segment unmatched. Lower values make omission/addition
    /// hypotheses easier to surface; this is evidence, not an automatic rejection.
    pub gap_penalty: f32,
    /// Small regularizer that prefers simpler 1:1 blocks when semantic evidence is tied.
    pub block_penalty: f32,
    /// Hard safety bound for either side of one alignment request.
    pub max_segments: usize,
}

impl Default for AlignmentConfig {
    fn default() -> Self {
        Self {
            max_block_size: 3,
            gap_penalty: 0.45,
            block_penalty: 0.06,
            max_segments: 512,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedSpan {
    pub start: usize,
    pub len: usize,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlignmentInput {
    pub schema_version: u32,
    pub unit_id: String,
    pub embedding_model: String,
    pub source_count: usize,
    pub target_count: usize,
    pub source_spans: Vec<EmbeddedSpan>,
    pub target_spans: Vec<EmbeddedSpan>,
    pub config: AlignmentConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignmentKind {
    Matched,
    SourceOnly,
    TargetOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlignmentBlock {
    pub kind: AlignmentKind,
    pub source_indices: Vec<usize>,
    pub target_indices: Vec<usize>,
    /// Raw cosine similarity for matched blocks. Gaps have no semantic similarity.
    pub similarity: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlignmentResult {
    pub schema_version: u32,
    pub unit_id: String,
    pub embedding_model: String,
    pub source_count: usize,
    pub target_count: usize,
    pub total_cost: f32,
    pub mean_matched_similarity: Option<f32>,
    pub blocks: Vec<AlignmentBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlignmentError {
    InvalidInput(String),
    MissingSpan {
        side: &'static str,
        start: usize,
        len: usize,
    },
    NoPath,
    UnitMismatch {
        report: String,
        alignment: String,
    },
}

impl fmt::Display for AlignmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(formatter, "invalid alignment input: {message}"),
            Self::MissingSpan { side, start, len } => {
                write!(
                    formatter,
                    "missing {side} embedding span start={start} len={len}"
                )
            }
            Self::NoPath => write!(formatter, "semantic alignment produced no monotonic path"),
            Self::UnitMismatch { report, alignment } => write!(
                formatter,
                "alignment unit_id '{alignment}' does not match review report '{report}'"
            ),
        }
    }
}

impl std::error::Error for AlignmentError {}

#[derive(Debug, Clone, Copy)]
struct Previous {
    i: usize,
    j: usize,
    source_take: usize,
    target_take: usize,
    similarity: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
struct Cell {
    cost: f32,
    previous: Option<Previous>,
    tie_rank: u16,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            cost: f32::INFINITY,
            previous: None,
            tie_rank: u16::MAX,
        }
    }
}

/// Monotonic block alignment over caller-provided multilingual span embeddings.
///
/// The algorithm is intentionally model-independent. An optional tool can use BGE-M3
/// (or another explicitly selected multilingual encoder) to create embeddings, while
/// this Rust core owns validation, deterministic dynamic programming, gap evidence,
/// and review integration. No model score can approve or rewrite translation text.
pub fn align_embeddings(input: &AlignmentInput) -> Result<AlignmentResult, AlignmentError> {
    let dimension = validate_input(input)?;
    let source = span_map(&input.source_spans);
    let target = span_map(&input.target_spans);
    require_span_coverage(
        "source",
        input.source_count,
        input.config.max_block_size,
        &source,
    )?;
    require_span_coverage(
        "target",
        input.target_count,
        input.config.max_block_size,
        &target,
    )?;

    let width = input.target_count + 1;
    let mut cells = vec![Cell::default(); (input.source_count + 1) * width];
    cells[0] = Cell {
        cost: 0.0,
        previous: None,
        tie_rank: 0,
    };

    for i in 0..=input.source_count {
        for j in 0..=input.target_count {
            let current = cells[i * width + j];
            if !current.cost.is_finite() {
                continue;
            }

            if i < input.source_count {
                update_cell(
                    &mut cells,
                    width,
                    i + 1,
                    j,
                    current.cost + input.config.gap_penalty,
                    200,
                    Previous {
                        i,
                        j,
                        source_take: 1,
                        target_take: 0,
                        similarity: None,
                    },
                );
            }
            if j < input.target_count {
                update_cell(
                    &mut cells,
                    width,
                    i,
                    j + 1,
                    current.cost + input.config.gap_penalty,
                    201,
                    Previous {
                        i,
                        j,
                        source_take: 0,
                        target_take: 1,
                        similarity: None,
                    },
                );
            }

            let max_source_take = input.config.max_block_size.min(input.source_count - i);
            let max_target_take = input.config.max_block_size.min(input.target_count - j);
            for source_take in 1..=max_source_take {
                for target_take in 1..=max_target_take {
                    let source_embedding =
                        source
                            .get(&(i, source_take))
                            .ok_or(AlignmentError::MissingSpan {
                                side: "source",
                                start: i,
                                len: source_take,
                            })?;
                    let target_embedding =
                        target
                            .get(&(j, target_take))
                            .ok_or(AlignmentError::MissingSpan {
                                side: "target",
                                start: j,
                                len: target_take,
                            })?;
                    debug_assert_eq!(source_embedding.len(), dimension);
                    debug_assert_eq!(target_embedding.len(), dimension);
                    let similarity = cosine_similarity(source_embedding, target_embedding);
                    let complexity = source_take + target_take - 2;
                    let transition_cost = (1.0 - similarity.max(0.0))
                        + input.config.block_penalty * complexity as f32;
                    let tie_rank = match (source_take, target_take) {
                        (1, 1) => 0,
                        _ => 10u16
                            .saturating_add(complexity.min(100) as u16)
                            .saturating_add(source_take.abs_diff(target_take).min(100) as u16),
                    };
                    update_cell(
                        &mut cells,
                        width,
                        i + source_take,
                        j + target_take,
                        current.cost + transition_cost,
                        tie_rank,
                        Previous {
                            i,
                            j,
                            source_take,
                            target_take,
                            similarity: Some(similarity),
                        },
                    );
                }
            }
        }
    }

    let final_cell = cells[input.source_count * width + input.target_count];
    if !final_cell.cost.is_finite() {
        return Err(AlignmentError::NoPath);
    }

    let mut blocks = Vec::new();
    let (mut i, mut j) = (input.source_count, input.target_count);
    while i != 0 || j != 0 {
        let previous = cells[i * width + j]
            .previous
            .ok_or(AlignmentError::NoPath)?;
        let kind = match (previous.source_take, previous.target_take) {
            (0, _) => AlignmentKind::TargetOnly,
            (_, 0) => AlignmentKind::SourceOnly,
            _ => AlignmentKind::Matched,
        };
        blocks.push(AlignmentBlock {
            kind,
            source_indices: (previous.i..previous.i + previous.source_take).collect(),
            target_indices: (previous.j..previous.j + previous.target_take).collect(),
            similarity: previous.similarity,
        });
        i = previous.i;
        j = previous.j;
    }
    blocks.reverse();
    let blocks = merge_adjacent_gaps(blocks);

    let similarities = blocks
        .iter()
        .filter_map(|block| block.similarity)
        .collect::<Vec<_>>();
    let mean_matched_similarity = if similarities.is_empty() {
        None
    } else {
        Some(similarities.iter().sum::<f32>() / similarities.len() as f32)
    };

    Ok(AlignmentResult {
        schema_version: ALIGNMENT_SCHEMA_VERSION,
        unit_id: input.unit_id.clone(),
        embedding_model: input.embedding_model.clone(),
        source_count: input.source_count,
        target_count: input.target_count,
        total_cost: final_cell.cost,
        mean_matched_similarity,
        blocks,
    })
}

/// Add only omission/addition evidence from an already validated alignment.
/// Matched semantic similarity is deliberately not converted into a literary verdict;
/// semantic fidelity, voice, register and subtext remain separate review dimensions.
pub fn attach_alignment_evidence(
    report: &mut LiteraryReviewReport,
    alignment: &AlignmentResult,
) -> Result<(), AlignmentError> {
    if report.unit_id != alignment.unit_id {
        return Err(AlignmentError::UnitMismatch {
            report: report.unit_id.clone(),
            alignment: alignment.unit_id.clone(),
        });
    }
    report.record_dimension(ReviewDimension::OmissionAddition);
    report.advisory_notes.push(format!(
        "Monotonic semantic alignment evidence generated with '{}'; gaps are review signals, not proof of an omission/addition and never human approval.",
        alignment.embedding_model
    ));

    for block in &alignment.blocks {
        match block.kind {
            AlignmentKind::SourceOnly => report.findings.push(
                ReviewFinding::new(
                    &report.unit_id,
                    ReviewDimension::OmissionAddition,
                    ReviewSeverity::Warning,
                    EvidenceSource::SemanticAlignment,
                    format!(
                        "semantic alignment left {} source segment(s) without a target counterpart; possible omission requires review",
                        block.source_indices.len()
                    ),
                )
                .with_indices(block.source_indices.iter().copied(), std::iter::empty())
                .with_revision_proposal(
                    "compare the unaligned source span with neighboring translated spans before revising",
                ),
            ),
            AlignmentKind::TargetOnly => report.findings.push(
                ReviewFinding::new(
                    &report.unit_id,
                    ReviewDimension::OmissionAddition,
                    ReviewSeverity::Warning,
                    EvidenceSource::SemanticAlignment,
                    format!(
                        "semantic alignment left {} target segment(s) without a source counterpart; possible addition requires review",
                        block.target_indices.len()
                    ),
                )
                .with_indices(std::iter::empty(), block.target_indices.iter().copied())
                .with_revision_proposal(
                    "compare the unaligned target span with neighboring source spans before revising",
                ),
            ),
            AlignmentKind::Matched => {}
        }
    }
    Ok(())
}

fn validate_input(input: &AlignmentInput) -> Result<usize, AlignmentError> {
    if input.schema_version != ALIGNMENT_SCHEMA_VERSION {
        return Err(AlignmentError::InvalidInput(format!(
            "unsupported schema version {}; expected {}",
            input.schema_version, ALIGNMENT_SCHEMA_VERSION
        )));
    }
    if input.unit_id.trim().is_empty() {
        return Err(AlignmentError::InvalidInput("unit_id is empty".into()));
    }
    if input.embedding_model.trim().is_empty() {
        return Err(AlignmentError::InvalidInput(
            "embedding_model is empty".into(),
        ));
    }
    if input.source_count > input.config.max_segments
        || input.target_count > input.config.max_segments
    {
        return Err(AlignmentError::InvalidInput(format!(
            "segment count exceeds configured limit {}",
            input.config.max_segments
        )));
    }
    if input.config.max_block_size == 0 || input.config.max_block_size > 8 {
        return Err(AlignmentError::InvalidInput(
            "max_block_size must be between 1 and 8".into(),
        ));
    }
    for (name, value) in [
        ("gap_penalty", input.config.gap_penalty),
        ("block_penalty", input.config.block_penalty),
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(AlignmentError::InvalidInput(format!(
                "{name} must be finite and non-negative"
            )));
        }
    }

    let mut dimension = None;
    for (side, count, spans) in [
        ("source", input.source_count, &input.source_spans),
        ("target", input.target_count, &input.target_spans),
    ] {
        let mut seen = HashMap::new();
        for span in spans {
            if span.len == 0 || span.len > input.config.max_block_size {
                return Err(AlignmentError::InvalidInput(format!(
                    "{side} span length {} is outside configured bounds",
                    span.len
                )));
            }
            if span
                .start
                .checked_add(span.len)
                .is_none_or(|end| end > count)
            {
                return Err(AlignmentError::InvalidInput(format!(
                    "{side} span start={} len={} exceeds segment count {count}",
                    span.start, span.len
                )));
            }
            if span.embedding.is_empty() || span.embedding.iter().any(|value| !value.is_finite()) {
                return Err(AlignmentError::InvalidInput(format!(
                    "{side} span start={} len={} has an empty or non-finite embedding",
                    span.start, span.len
                )));
            }
            if seen.insert((span.start, span.len), ()).is_some() {
                return Err(AlignmentError::InvalidInput(format!(
                    "duplicate {side} span start={} len={}",
                    span.start, span.len
                )));
            }
            match dimension {
                Some(expected) if expected != span.embedding.len() => {
                    return Err(AlignmentError::InvalidInput(format!(
                        "embedding dimension mismatch: expected {expected}, got {}",
                        span.embedding.len()
                    )));
                }
                None => dimension = Some(span.embedding.len()),
                _ => {}
            }
        }
    }

    if input.source_count == 0 && input.target_count == 0 {
        return Ok(dimension.unwrap_or(1));
    }
    dimension.ok_or_else(|| AlignmentError::InvalidInput("no embeddings supplied".into()))
}

fn span_map(spans: &[EmbeddedSpan]) -> HashMap<(usize, usize), &[f32]> {
    spans
        .iter()
        .map(|span| ((span.start, span.len), span.embedding.as_slice()))
        .collect()
}

fn require_span_coverage(
    side: &'static str,
    count: usize,
    max_block_size: usize,
    spans: &HashMap<(usize, usize), &[f32]>,
) -> Result<(), AlignmentError> {
    for start in 0..count {
        for len in 1..=max_block_size.min(count - start) {
            if !spans.contains_key(&(start, len)) {
                return Err(AlignmentError::MissingSpan { side, start, len });
            }
        }
    }
    Ok(())
}

fn update_cell(
    cells: &mut [Cell],
    width: usize,
    i: usize,
    j: usize,
    new_cost: f32,
    tie_rank: u16,
    previous: Previous,
) {
    const EPSILON: f32 = 1.0e-6;
    let cell = &mut cells[i * width + j];
    let cheaper = new_cost + EPSILON < cell.cost;
    let tied_but_preferred = (new_cost - cell.cost).abs() <= EPSILON && tie_rank < cell.tie_rank;
    if cheaper || tied_but_preferred {
        cell.cost = new_cost;
        cell.previous = Some(previous);
        cell.tie_rank = tie_rank;
    }
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }
    let (mut dot, mut left_norm, mut right_norm) = (0.0f32, 0.0f32, 0.0f32);
    for (left_value, right_value) in left.iter().zip(right) {
        dot += left_value * right_value;
        left_norm += left_value * left_value;
        right_norm += right_value * right_value;
    }
    let denominator = left_norm.sqrt() * right_norm.sqrt();
    if denominator <= f32::EPSILON {
        0.0
    } else {
        (dot / denominator).clamp(-1.0, 1.0)
    }
}

fn merge_adjacent_gaps(blocks: Vec<AlignmentBlock>) -> Vec<AlignmentBlock> {
    let mut merged: Vec<AlignmentBlock> = Vec::with_capacity(blocks.len());
    for block in blocks {
        if let Some(previous) = merged.last_mut() {
            if previous.kind == block.kind
                && matches!(
                    block.kind,
                    AlignmentKind::SourceOnly | AlignmentKind::TargetOnly
                )
            {
                previous.source_indices.extend(block.source_indices);
                previous.target_indices.extend(block.target_indices);
                continue;
            }
        }
        merged.push(block);
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{review_native, NativeReviewConfig};

    fn span(start: usize, len: usize, embedding: &[f32]) -> EmbeddedSpan {
        EmbeddedSpan {
            start,
            len,
            embedding: embedding.to_vec(),
        }
    }

    fn input(
        source_count: usize,
        target_count: usize,
        source_spans: Vec<EmbeddedSpan>,
        target_spans: Vec<EmbeddedSpan>,
        config: AlignmentConfig,
    ) -> AlignmentInput {
        AlignmentInput {
            schema_version: ALIGNMENT_SCHEMA_VERSION,
            unit_id: "chapter-1".into(),
            embedding_model: "synthetic-test-embedding".into(),
            source_count,
            target_count,
            source_spans,
            target_spans,
            config,
        }
    }

    #[test]
    fn identity_alignment_prefers_one_to_one_blocks() {
        let config = AlignmentConfig {
            max_block_size: 1,
            ..Default::default()
        };
        let result = align_embeddings(&input(
            2,
            2,
            vec![span(0, 1, &[1.0, 0.0]), span(1, 1, &[0.0, 1.0])],
            vec![span(0, 1, &[1.0, 0.0]), span(1, 1, &[0.0, 1.0])],
            config,
        ))
        .unwrap();
        assert_eq!(result.blocks.len(), 2);
        assert!(result
            .blocks
            .iter()
            .all(|block| block.kind == AlignmentKind::Matched));
        assert_eq!(result.blocks[0].source_indices, vec![0]);
        assert_eq!(result.blocks[0].target_indices, vec![0]);
    }

    #[test]
    fn many_to_one_is_supported_when_combined_span_is_semantically_stronger() {
        let config = AlignmentConfig {
            max_block_size: 2,
            gap_penalty: 0.45,
            block_penalty: 0.04,
            ..Default::default()
        };
        let diagonal = 0.70710677;
        let result = align_embeddings(&input(
            2,
            1,
            vec![
                span(0, 1, &[1.0, 0.0]),
                span(1, 1, &[0.0, 1.0]),
                span(0, 2, &[diagonal, diagonal]),
            ],
            vec![span(0, 1, &[diagonal, diagonal])],
            config,
        ))
        .unwrap();
        assert_eq!(result.blocks.len(), 1);
        assert_eq!(result.blocks[0].kind, AlignmentKind::Matched);
        assert_eq!(result.blocks[0].source_indices, vec![0, 1]);
        assert_eq!(result.blocks[0].target_indices, vec![0]);
    }

    #[test]
    fn a_semantically_unmatched_source_segment_becomes_omission_evidence() {
        let config = AlignmentConfig {
            max_block_size: 2,
            gap_penalty: 0.25,
            block_penalty: 0.12,
            ..Default::default()
        };
        let result = align_embeddings(&input(
            2,
            1,
            vec![
                span(0, 1, &[1.0, 0.0]),
                span(1, 1, &[-1.0, 0.0]),
                span(0, 2, &[0.0, 1.0]),
            ],
            vec![span(0, 1, &[1.0, 0.0])],
            config,
        ))
        .unwrap();
        assert!(result.blocks.iter().any(|block| {
            block.kind == AlignmentKind::SourceOnly && block.source_indices == vec![1]
        }));

        let mut report = review_native(
            "chapter-1",
            "source one\n\nsource two",
            "هدف یک",
            NativeReviewConfig::default(),
        );
        attach_alignment_evidence(&mut report, &result).unwrap();
        assert!(report.findings.iter().any(|finding| {
            finding.evidence_source == EvidenceSource::SemanticAlignment
                && finding.source_indices == vec![1]
                && finding.summary.contains("possible omission")
        }));
    }

    #[test]
    fn missing_overlap_embedding_is_rejected_instead_of_silently_downgrading() {
        let config = AlignmentConfig {
            max_block_size: 2,
            ..Default::default()
        };
        let error = align_embeddings(&input(
            2,
            1,
            vec![span(0, 1, &[1.0]), span(1, 1, &[1.0])],
            vec![span(0, 1, &[1.0])],
            config,
        ))
        .unwrap_err();
        assert_eq!(
            error,
            AlignmentError::MissingSpan {
                side: "source",
                start: 0,
                len: 2
            }
        );
    }

    #[test]
    fn alignment_request_is_bounded() {
        let config = AlignmentConfig {
            max_segments: 1,
            max_block_size: 1,
            ..Default::default()
        };
        let error = align_embeddings(&input(
            2,
            1,
            vec![span(0, 1, &[1.0]), span(1, 1, &[1.0])],
            vec![span(0, 1, &[1.0])],
            config,
        ))
        .unwrap_err();
        assert!(matches!(error, AlignmentError::InvalidInput(_)));
    }
}
