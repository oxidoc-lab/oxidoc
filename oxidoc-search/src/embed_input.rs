//! Embeds text one document per encoder forward, cut to the model's position table.
//!
//! The builder and the browser query path both embed through this module, so a
//! document and a query of the same text get the same vector.

use boostr::model::encoder::{EmbeddingPipeline, EncoderClient};
use numr::prelude::*;
use splintr::Tokenize;

/// Token ids the encoder reads for `text`.
///
/// The ids are cut to the model's `max_position_embeddings`. A position past the
/// table has no learned embedding, and attention cost grows with the square of
/// the sequence length.
pub fn input_ids<R, T>(pipeline: &EmbeddingPipeline<R, T>, text: &str) -> Vec<i64>
where
    R: Runtime<DType = DType>,
    T: Tokenize,
{
    let max_positions = pipeline.config().max_position_embeddings;
    pipeline
        .tokenizer()
        .encode(text)
        .into_iter()
        .take(max_positions)
        .map(i64::from)
        .collect()
}

/// Embeds `text` in one forward pass over at most `max_position_embeddings` tokens.
///
/// Returns `hidden_size` zeros when `text` has no tokens.
pub fn embed_one<R, T, C>(
    pipeline: &EmbeddingPipeline<R, T>,
    client: &C,
    device: &R::Device,
    text: &str,
) -> boostr::error::Result<Vec<f32>>
where
    R: Runtime<DType = DType>,
    T: Tokenize,
    C: EncoderClient<R>,
    R::Client: TensorOps<R> + ScalarOps<R> + IndexingOps<R>,
{
    let ids = input_ids(pipeline, text);
    if ids.is_empty() {
        return Ok(vec![0.0; pipeline.config().hidden_size]);
    }
    let input = Tensor::<R>::from_slice(&ids, &[1, ids.len()], device);
    let embedding = pipeline.encoder().embed(client, &input)?;
    Ok(embedding.tensor().to_vec())
}

/// Embeds each text in its own forward pass and returns the vectors in input order.
///
/// Peak memory is that of one document. One batched forward over every text holds
/// the attention scores of all of them at once, and pads short texts without a mask.
pub fn embed_each<R, T, C>(
    pipeline: &EmbeddingPipeline<R, T>,
    client: &C,
    device: &R::Device,
    texts: &[&str],
) -> boostr::error::Result<Vec<Vec<f32>>>
where
    R: Runtime<DType = DType>,
    T: Tokenize,
    C: EncoderClient<R>,
    R::Client: TensorOps<R> + ScalarOps<R> + IndexingOps<R>,
{
    texts
        .iter()
        .map(|text| embed_one(pipeline, client, device, text))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use boostr::model::encoder::config::HiddenAct;
    use boostr::model::encoder::{Encoder, EncoderConfig, Pooling};
    use numr::runtime::cpu::{CpuClient, CpuDevice, CpuRuntime};
    use splintr::TokenizeError;

    const HIDDEN: usize = 8;
    const VOCAB: usize = 32;
    const MAX_POSITIONS: usize = 16;

    /// Maps each whitespace word to its length, so test texts pick their own ids.
    struct WordLengthTokenizer;

    impl Tokenize for WordLengthTokenizer {
        fn encode(&self, text: &str) -> Vec<u32> {
            text.split_whitespace()
                .map(|w| (w.len() % VOCAB) as u32)
                .collect()
        }

        fn decode(&self, _ids: &[u32]) -> std::result::Result<String, TokenizeError> {
            Ok(String::new())
        }

        fn vocab_size(&self) -> usize {
            VOCAB
        }
    }

    /// Deterministic, non-uniform values so outputs depend on token and position.
    fn ramp(len: usize, scale: f32) -> Vec<f32> {
        (0..len)
            .map(|i| ((i * 7 % 11) as f32 - 5.0) * scale)
            .collect()
    }

    fn pipeline() -> (
        EmbeddingPipeline<CpuRuntime, WordLengthTokenizer>,
        CpuClient,
        CpuDevice,
    ) {
        let device = CpuDevice::new();
        let client = CpuClient::new(device.clone());
        let config = EncoderConfig {
            vocab_size: VOCAB,
            hidden_size: HIDDEN,
            num_hidden_layers: 1,
            num_attention_heads: 2,
            intermediate_size: 16,
            max_position_embeddings: MAX_POSITIONS,
            layer_norm_eps: 1e-12,
            hidden_act: HiddenAct::Gelu,
            type_vocab_size: 0,
        };
        let d = &device;
        let tensor = |values: Vec<f32>, shape: &[usize]| Tensor::from_slice(&values, shape, d);
        let encoder = Encoder::from_weights(config, Pooling::Mean, |name| {
            let weight = match name {
                "embeddings.word_embeddings.weight" => {
                    tensor(ramp(VOCAB * HIDDEN, 0.1), &[VOCAB, HIDDEN])
                }
                "embeddings.position_embeddings.weight" => {
                    tensor(ramp(MAX_POSITIONS * HIDDEN, 0.03), &[MAX_POSITIONS, HIDDEN])
                }
                n if n.ends_with("intermediate.dense.weight") => {
                    tensor(ramp(16 * HIDDEN, 0.02), &[16, HIDDEN])
                }
                n if n.ends_with("intermediate.dense.bias") => tensor(vec![0.0; 16], &[16]),
                n if n.ends_with("output.dense.weight") && !n.contains("attention") => {
                    tensor(ramp(HIDDEN * 16, 0.02), &[HIDDEN, 16])
                }
                n if n.ends_with("LayerNorm.weight") => tensor(vec![1.0; HIDDEN], &[HIDDEN]),
                n if n.ends_with(".weight") => {
                    tensor(ramp(HIDDEN * HIDDEN, 0.05), &[HIDDEN, HIDDEN])
                }
                _ => tensor(vec![0.0; HIDDEN], &[HIDDEN]),
            };
            Ok(weight)
        })
        .unwrap();
        let pipeline = EmbeddingPipeline::new(encoder, WordLengthTokenizer, device.clone());
        (pipeline, client, device)
    }

    fn words(count: usize) -> String {
        (0..count)
            .map(|i| "x".repeat(1 + i % 9))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn max_diff(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0, f32::max)
    }

    #[test]
    fn input_ids_stop_at_max_positions() {
        let (pipeline, _, _) = pipeline();
        assert_eq!(input_ids(&pipeline, &words(5)).len(), 5);
        assert_eq!(input_ids(&pipeline, &words(100)).len(), MAX_POSITIONS);
    }

    #[test]
    fn long_text_embeds_as_its_first_max_positions_tokens() {
        let (pipeline, client, device) = pipeline();
        let long = embed_one(&pipeline, &client, &device, &words(100)).unwrap();
        let prefix = embed_one(&pipeline, &client, &device, &words(MAX_POSITIONS)).unwrap();
        assert_eq!(long.len(), HIDDEN);
        assert!(max_diff(&long, &prefix) < 1e-6);
    }

    #[test]
    fn vector_does_not_depend_on_other_texts() {
        let (pipeline, client, device) = pipeline();
        let short = words(3);
        let long = words(12);
        let alone = embed_each(&pipeline, &client, &device, &[&short]).unwrap();
        let together = embed_each(&pipeline, &client, &device, &[&short, &long]).unwrap();
        assert_eq!(together.len(), 2);
        assert!(max_diff(&alone[0], &together[0]) < 1e-6);
    }

    #[test]
    fn empty_text_gives_zero_vector() {
        let (pipeline, client, device) = pipeline();
        let vector = embed_one(&pipeline, &client, &device, "   ").unwrap();
        assert_eq!(vector, vec![0.0; HIDDEN]);
    }
}
