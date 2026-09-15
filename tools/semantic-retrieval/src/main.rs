use std::env;
use std::error::Error;
use std::io::{self, Read};
use std::path::PathBuf;

use memory_engine::{
    SemanticMode, SemanticRerankRequest, SemanticRerankResponse, SemanticScore,
    SEMANTIC_PROTOCOL_VERSION,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("semantic-retrieval-tool: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let request: SemanticRerankRequest = serde_json::from_str(&input)?;
    validate_request(&request)?;

    #[cfg(feature = "bge")]
    let response = bge::rank(&request)?;

    #[cfg(not(feature = "bge"))]
    let response = {
        let _ = request;
        return Err(
            "this binary was built without semantic models; rebuild with `--features bge`".into(),
        );
    };

    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}

fn validate_request(request: &SemanticRerankRequest) -> Result<(), Box<dyn Error>> {
    if request.schema_version != SEMANTIC_PROTOCOL_VERSION {
        return Err(format!(
            "unsupported protocol version {}; expected {}",
            request.schema_version, SEMANTIC_PROTOCOL_VERSION
        )
        .into());
    }
    if request.query.trim().is_empty() {
        return Err("query must not be empty".into());
    }
    if request.candidates.is_empty() {
        return Err("candidates must not be empty".into());
    }
    if request.candidates.len() > 512 {
        return Err("candidate count exceeds the hard safety limit of 512".into());
    }
    if request.max_results == 0 || request.max_results > request.candidates.len() {
        return Err("max_results must be between 1 and candidate count".into());
    }
    let mut ids = std::collections::HashSet::new();
    for candidate in &request.candidates {
        if candidate.id.trim().is_empty() || candidate.text.trim().is_empty() {
            return Err("candidate IDs and texts must not be empty".into());
        }
        if !ids.insert(candidate.id.as_str()) {
            return Err(format!("duplicate candidate id {}", candidate.id).into());
        }
    }
    Ok(())
}

fn model_cache_dir() -> PathBuf {
    env::var_os("PERSIAN_TRANSLATOR_MODEL_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".tools/models/fastembed"))
}

#[cfg(feature = "bge")]
mod bge {
    use super::*;
    use fastembed::{
        Bgem3Embedding, Bgem3InitOptions, Bgem3Model, RerankInitOptions, RerankerModel,
        TextRerank,
    };

    pub fn rank(request: &SemanticRerankRequest) -> Result<SemanticRerankResponse, Box<dyn Error>> {
        match request.mode {
            SemanticMode::Dense => dense_rank(request),
            SemanticMode::Rerank => cross_encoder_rank(request),
            SemanticMode::DenseThenRerank => dense_then_rerank(request),
        }
    }

    fn dense_rank(
        request: &SemanticRerankRequest,
    ) -> Result<SemanticRerankResponse, Box<dyn Error>> {
        let ranked = dense_candidates(request)?;
        Ok(response(
            request,
            "gpahal/bge-m3-onnx-int8",
            ranked.into_iter().take(request.max_results).collect(),
        ))
    }

    fn cross_encoder_rank(
        request: &SemanticRerankRequest,
    ) -> Result<SemanticRerankResponse, Box<dyn Error>> {
        let indices = (0..request.candidates.len()).collect::<Vec<_>>();
        let ranked = rerank_subset(request, &indices)?;
        Ok(response(
            request,
            "rozgo/bge-reranker-v2-m3",
            ranked.into_iter().take(request.max_results).collect(),
        ))
    }

    fn dense_then_rerank(
        request: &SemanticRerankRequest,
    ) -> Result<SemanticRerankResponse, Box<dyn Error>> {
        let dense = dense_candidates(request)?;
        let rerank_pool = request
            .max_results
            .saturating_mul(4)
            .max(16)
            .min(request.candidates.len());
        let indices = dense
            .iter()
            .take(rerank_pool)
            .map(|(index, _)| *index)
            .collect::<Vec<_>>();
        let ranked = rerank_subset(request, &indices)?;
        Ok(response(
            request,
            "gpahal/bge-m3-onnx-int8 + rozgo/bge-reranker-v2-m3",
            ranked.into_iter().take(request.max_results).collect(),
        ))
    }

    fn dense_candidates(
        request: &SemanticRerankRequest,
    ) -> Result<Vec<(usize, f32)>, Box<dyn Error>> {
        let cache = model_cache_dir().join("bge-m3");
        let mut model = Bgem3Embedding::try_new(
            Bgem3InitOptions::new(Bgem3Model::BGEM3Q)
                .with_cache_dir(cache)
                .with_show_download_progress(false),
        )?;

        let mut texts = Vec::with_capacity(request.candidates.len() + 1);
        texts.push(request.query.as_str());
        texts.extend(request.candidates.iter().map(|candidate| candidate.text.as_str()));
        let output = model.embed(texts, Some(16))?;
        let query = output
            .dense
            .first()
            .ok_or("BGE-M3 returned no query embedding")?;
        if output.dense.len() != request.candidates.len() + 1 {
            return Err("BGE-M3 returned an unexpected embedding count".into());
        }

        let mut ranked = output
            .dense
            .iter()
            .skip(1)
            .enumerate()
            .map(|(index, embedding)| (index, cosine_similarity(query, embedding)))
            .collect::<Vec<_>>();
        ranked.sort_by(|(index_a, score_a), (index_b, score_b)| {
            score_b
                .total_cmp(score_a)
                .then_with(|| request.candidates[*index_a].id.cmp(&request.candidates[*index_b].id))
        });
        Ok(ranked)
    }

    fn rerank_subset(
        request: &SemanticRerankRequest,
        candidate_indices: &[usize],
    ) -> Result<Vec<(usize, f32)>, Box<dyn Error>> {
        let cache = model_cache_dir().join("bge-reranker-v2-m3");
        let mut model = TextRerank::try_new(
            RerankInitOptions::new(RerankerModel::BGERerankerV2M3)
                .with_cache_dir(cache)
                .with_show_download_progress(false),
        )?;
        let documents = candidate_indices
            .iter()
            .map(|index| request.candidates[*index].text.as_str())
            .collect::<Vec<_>>();
        let results = model.rerank(request.query.as_str(), documents, false, Some(16))?;

        let mut ranked = results
            .into_iter()
            .map(|result| {
                let original_index = candidate_indices
                    .get(result.index)
                    .copied()
                    .ok_or("reranker returned an out-of-range index")?;
                Ok((original_index, result.score))
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
        ranked.sort_by(|(index_a, score_a), (index_b, score_b)| {
            score_b
                .total_cmp(score_a)
                .then_with(|| request.candidates[*index_a].id.cmp(&request.candidates[*index_b].id))
        });
        Ok(ranked)
    }

    fn response(
        request: &SemanticRerankRequest,
        model: &str,
        ranked: Vec<(usize, f32)>,
    ) -> SemanticRerankResponse {
        SemanticRerankResponse {
            schema_version: SEMANTIC_PROTOCOL_VERSION,
            model: model.to_string(),
            mode: request.mode,
            scores: ranked
                .into_iter()
                .enumerate()
                .map(|(zero_rank, (index, score))| SemanticScore {
                    id: request.candidates[index].id.clone(),
                    score,
                    rank: zero_rank + 1,
                })
                .collect(),
        }
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let (mut dot, mut norm_a, mut norm_b) = (0.0_f32, 0.0_f32, 0.0_f32);
        for (left, right) in a.iter().zip(b) {
            dot += left * right;
            norm_a += left * left;
            norm_b += right * right;
        }
        let denominator = norm_a.sqrt() * norm_b.sqrt();
        if denominator <= f32::EPSILON {
            0.0
        } else {
            (dot / denominator).clamp(-1.0, 1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory_engine::SemanticCandidate;

    #[test]
    fn rejects_duplicate_candidate_ids_without_loading_models() {
        let request = SemanticRerankRequest::new(
            "query",
            vec![
                SemanticCandidate {
                    id: "same".into(),
                    text: "one".into(),
                },
                SemanticCandidate {
                    id: "same".into(),
                    text: "two".into(),
                },
            ],
            1,
        );
        assert!(validate_request(&request).is_err());
    }

    #[test]
    fn default_cache_is_project_local_and_ignored_by_policy() {
        env::remove_var("PERSIAN_TRANSLATOR_MODEL_CACHE");
        assert_eq!(model_cache_dir(), PathBuf::from(".tools/models/fastembed"));
    }
}
