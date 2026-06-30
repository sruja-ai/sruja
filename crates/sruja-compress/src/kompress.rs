//! ONNX-backed Kompress backend (`kompress-small` / `kompress-v2-base`).
//!
//! `kompress-small` (279 MB, Apache-2.0) is the recommended default: 13–29 ms
//! ONNX latency, 7.4/10 agent-trace quality vs LLMLingua-2's 6.2. Loads
//! lazily on first call; stays resident for the session.
//!
//! Requires the `kompress` feature. Weights + `tokenizer.json` are fetched from
//! HuggingFace at config time (not at build) and pointed at via [`KompressConfig`].
//!
//! # Forward pass
//!
//! The model is ModernBERT + a dual head (per-token classifier + 1-D span conv)
//! producing `final_scores ∈ [0,1]` per subword. Thresholding yields the keep
//! mask; kept subwords decode to the compressed (extractive) text.
//!
//! The exact input/output tensor names and the head arithmetic must be read
//! from the model's `config.json` / the ONNX graph — they are data-driven here,
//! not hard-coded, so the same backend serves `-small` and `-v2-base`.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::{count_tokens, BackendId, CompressContext, CompressError, Compressed, TextCompressor};

use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;
use tokenizers::Tokenizer;

/// Configuration for the ONNX-backed Kompress backend.
#[derive(Debug, Clone)]
pub struct KompressConfig {
    /// Directory containing `tokenizer.json` + the ONNX model file (e.g. `model.onnx`).
    pub model_dir: PathBuf,
    pub variant: KompressVariant,
    /// P(keep) threshold in [0,1]. Higher = more aggressive compression.
    pub threshold: f64,
    /// Native ModernBERT context window. Inputs longer than this are chunked with overlap.
    pub max_length: usize,
    /// Overlap tokens between chunks so boundaries aren't dropped.
    pub chunk_overlap: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KompressVariant {
    /// ~279 MB, 13–29 ms ONNX. Recommended default.
    Small,
    /// ~600 MB, ~84 ms MPS. Available if you have more runtime budget.
    V2Base,
}

impl Default for KompressConfig {
    fn default() -> Self {
        Self {
            model_dir: PathBuf::from("models/kompress-small"),
            variant: KompressVariant::Small,
            threshold: 0.5,
            max_length: 8192,
            chunk_overlap: 256,
        }
    }
}

/// Lazily-loaded ONNX session + tokenizer. Cheap to clone (Arc inside).
#[derive(Clone)]
pub struct KompressBackend {
    cfg: KompressConfig,
    state: Arc<LazyState>,
}

struct LazyState {
    session: Mutex<Option<Session>>,
    tokenizer: Mutex<Option<Tokenizer>>,
}

impl KompressBackend {
    pub fn new(cfg: KompressConfig) -> Self {
        Self {
            cfg,
            state: Arc::new(LazyState {
                session: Mutex::new(None),
                tokenizer: Mutex::new(None),
            }),
        }
    }

    pub fn config(&self) -> &KompressConfig {
        &self.cfg
    }

    fn tokenizer(&self) -> Result<Tokenizer, CompressError> {
        let mut guard = self
            .state
            .tokenizer
            .lock()
            .map_err(|e| CompressError::Load(format!("tokenizer lock poisoned: {e}")))?;
        if guard.is_none() {
            let path = self.cfg.model_dir.join("tokenizer.json");
            let mut tokenizer = Tokenizer::from_file(&path)
                .map_err(|e| CompressError::Load(format!("failed to load tokenizer: {e}")))?;
            tokenizer
                .with_truncation(Some(tokenizers::TruncationParams {
                    max_length: self.cfg.max_length,
                    ..Default::default()
                }))
                .map_err(|e| CompressError::Load(format!("failed to configure tokenizer: {e}")))?;
            *guard = Some(tokenizer);
        }
        Ok(guard.as_ref().unwrap().clone())
    }

    fn with_session<R>(
        &self,
        f: impl FnOnce(&mut Session) -> Result<R, CompressError>,
    ) -> Result<R, CompressError> {
        let mut guard = self
            .state
            .session
            .lock()
            .map_err(|e| CompressError::Load(format!("session lock poisoned: {e}")))?;
        if guard.is_none() {
            let model_path = self.cfg.model_dir.join("model.onnx");
            let session = Session::builder()
                .map_err(|e| CompressError::Load(format!("session builder failed: {e}")))?
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .map_err(|e| CompressError::Load(format!("optimization level failed: {e}")))?
                .with_intra_threads(4)
                .map_err(|e| CompressError::Load(format!("intra threads failed: {e}")))?
                .commit_from_file(&model_path)
                .map_err(|e| CompressError::Load(format!("failed to load ONNX model: {e}")))?;
            *guard = Some(session);
        }
        f(guard.as_mut().unwrap())
    }

    fn threshold_for_ratio(&self, _ratio: f64) -> f64 {
        self.cfg.threshold
    }

    fn run_single(
        session: &mut Session,
        input_ids: &[i64],
        attention_mask: &[i64],
        threshold: f64,
    ) -> Result<Vec<bool>, CompressError> {
        let seq_len = input_ids.len();
        let shape = vec![1_i64, seq_len as i64];

        let ids_tensor = Tensor::from_array((shape.clone(), input_ids.to_vec())).map_err(|e| {
            CompressError::Inference(format!("failed to create input_ids tensor: {e}"))
        })?;
        let mask_tensor = Tensor::from_array((shape, attention_mask.to_vec())).map_err(|e| {
            CompressError::Inference(format!("failed to create attention_mask tensor: {e}"))
        })?;

        let inputs = ort::inputs! {
            "input_ids" => ids_tensor,
            "attention_mask" => mask_tensor,
        };

        let outputs = session
            .run(inputs)
            .map_err(|e| CompressError::Inference(format!("ONNX inference failed: {e}")))?;

        let scores_value = outputs
            .get("final_scores")
            .or_else(|| outputs.get("logits"))
            .or_else(|| outputs.get("scores"))
            .ok_or_else(|| {
                CompressError::Inference(
                    "ONNX output tensor not found: expected 'final_scores', 'logits', or 'scores'"
                        .to_string(),
                )
            })?;

        let scores = scores_value
            .try_extract_array::<f32>()
            .map_err(|e| CompressError::Inference(format!("failed to extract scores: {e}")))?;

        Ok(scores.iter().map(|&s| s >= threshold as f32).collect())
    }

    fn decode_kept(
        tokenizer: &Tokenizer,
        input_ids: &[i64],
        keep_mask: &[bool],
        original_content: &str,
        ctx: &CompressContext<'_>,
        backend: BackendId,
    ) -> Result<Compressed, CompressError> {
        let kept_ids: Vec<u32> = input_ids
            .iter()
            .zip(keep_mask.iter())
            .filter_map(|(&id, &keep)| if keep { Some(id as u32) } else { None })
            .collect();

        if kept_ids.is_empty() {
            let toks = count_tokens(original_content);
            return Ok(Compressed {
                text: original_content.to_string(),
                original_tokens: toks,
                compressed_tokens: toks,
                backend,
                ccr_handle: None,
            });
        }

        let compressed_text = tokenizer
            .decode(&kept_ids, true)
            .map_err(|e| CompressError::Inference(format!("decode failed: {e}")))?;

        let compressed_text = crate::restore_kept(original_content, &compressed_text, &ctx.keep);
        let compressed_tokens = count_tokens(&compressed_text);

        Ok(Compressed {
            text: compressed_text,
            original_tokens: count_tokens(original_content),
            compressed_tokens,
            backend,
            ccr_handle: None,
        })
    }
}

impl TextCompressor for KompressBackend {
    fn backend(&self) -> BackendId {
        match self.cfg.variant {
            KompressVariant::Small => BackendId::KompressSmall,
            KompressVariant::V2Base => BackendId::KompressV2Base,
        }
    }

    fn compress(
        &self,
        content: &str,
        ctx: &CompressContext<'_>,
    ) -> Result<Compressed, CompressError> {
        let original_tokens = count_tokens(content);
        if original_tokens < 64 {
            return Ok(Compressed {
                text: content.to_string(),
                original_tokens,
                compressed_tokens: original_tokens,
                backend: self.backend(),
                ccr_handle: None,
            });
        }

        let threshold = ctx
            .target_ratio
            .map(|r| self.threshold_for_ratio(r))
            .unwrap_or(self.cfg.threshold);

        let tokenizer = self.tokenizer()?;

        let encoded = tokenizer
            .encode(content, true)
            .map_err(|e| CompressError::Load(format!("tokenization failed: {e}")))?;

        let input_ids: Vec<i64> = encoded.get_ids().iter().map(|&id| id as i64).collect();
        let attention_mask: Vec<i64> = encoded
            .get_attention_mask()
            .iter()
            .map(|&m| m as i64)
            .collect();

        let seq_len = input_ids.len();
        let backend = self.backend();

        if seq_len > self.cfg.max_length {
            let max_len = self.cfg.max_length;
            let overlap = self.cfg.chunk_overlap;
            let step = max_len.saturating_sub(overlap).max(1);

            let mut keep_mask = vec![false; seq_len];

            self.with_session(|session| {
                for chunk_start in (0..seq_len).step_by(step) {
                    let chunk_end = (chunk_start + max_len).min(seq_len);
                    let chunk_len = chunk_end - chunk_start;
                    if chunk_len < 2 {
                        continue;
                    }

                    let chunk_ids = &input_ids[chunk_start..chunk_end];
                    let chunk_mask = &attention_mask[chunk_start..chunk_end];

                    let chunk_keep = Self::run_single(session, chunk_ids, chunk_mask, threshold)?;

                    for (i, keep) in chunk_keep.iter().enumerate() {
                        let global_idx = chunk_start + i;
                        if global_idx < seq_len {
                            keep_mask[global_idx] |= *keep;
                        }
                    }
                }
                Ok(())
            })?;

            Self::decode_kept(&tokenizer, &input_ids, &keep_mask, content, ctx, backend)
        } else {
            let keep_mask = self.with_session(|session| {
                Self::run_single(session, &input_ids, &attention_mask, threshold)
            })?;

            Self::decode_kept(&tokenizer, &input_ids, &keep_mask, content, ctx, backend)
        }
    }
}
