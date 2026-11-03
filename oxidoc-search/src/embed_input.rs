//! Embeds text one document per encoder forward.
//!
//! The builder and the browser query path both embed through this module, so a
//! document and a query of the same text get the same vector.
//!
//! [`EmbeddingPipeline::embed_text`] wraps the tokens in the model's `[CLS]` /
//! `[SEP]` template and cuts them to `max_position_embeddings`. Each call runs
//! one forward over one text, so peak memory is that of one document.

use boostr::DequantOps;
use boostr::model::encoder::{EmbeddingPipeline, EncoderClient};
use numr::prelude::*;

/// Embeds `text` in one forward pass.
///
/// Returns `hidden_size` zeros when `text` has no tokens. The pipeline would
/// embed the bare `[CLS] [SEP]` pair instead, and that vector matches every
/// empty document equally.
pub fn embed_one<R, C>(
    pipeline: &EmbeddingPipeline<R>,
    client: &C,
    text: &str,
) -> boostr::error::Result<Vec<f32>>
where
    R: Runtime<DType = DType>,
    C: EncoderClient<R>,
    R::Client: TensorOps<R> + ScalarOps<R> + IndexingOps<R> + DequantOps<R>,
{
    if pipeline.tokenizer().encode_raw(text).is_empty() {
        return Ok(vec![0.0; pipeline.config().hidden_size]);
    }
    pipeline.embed_text(client, text)
}

/// Embeds each text in its own forward pass and returns the vectors in input order.
///
/// Peak memory is that of one document. `EmbeddingPipeline::embed_texts` in
/// boostr 0.10.0 runs every text of a padded-path model in one forward, so its
/// attention scores grow with the number of texts.
pub fn embed_each<R, C>(
    pipeline: &EmbeddingPipeline<R>,
    client: &C,
    texts: &[&str],
) -> boostr::error::Result<Vec<Vec<f32>>>
where
    R: Runtime<DType = DType>,
    C: EncoderClient<R>,
    R::Client: TensorOps<R> + ScalarOps<R> + IndexingOps<R> + DequantOps<R>,
{
    texts
        .iter()
        .map(|text| embed_one(pipeline, client, text))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use boostr::model::encoder::{Encoder, EncoderConfig, Pooling};
    use numr::runtime::cpu::{CpuClient, CpuDevice, CpuRuntime};
    use splintr::{AnyTokenizer, GgufVocab};

    const HIDDEN: usize = 8;
    const MAX_POSITIONS: usize = 16;
    /// Plain words in the test vocabulary, spelled `w0` .. `w{WORDS-1}`.
    const WORDS: usize = 20;
    const SPECIALS: [&str; 4] = ["[PAD]", "[UNK]", "[CLS]", "[SEP]"];

    /// A BERT WordPiece tokenizer built the way a GGUF file's vocab is loaded.
    ///
    /// Every word `w{i}` is one token, so test texts control their token count.
    fn tokenizer() -> AnyTokenizer {
        let tokens = SPECIALS
            .iter()
            .map(|s| s.to_string())
            .chain((0..WORDS).map(|i| format!("w{i}")))
            .collect();
        let vocab = GgufVocab {
            model: "bert".into(),
            tokens,
            ..Default::default()
        };
        splintr::from_gguf_vocab(vocab).unwrap()
    }

    /// Deterministic, non-uniform values so outputs depend on token and position.
    fn ramp(len: usize, scale: f32) -> Vec<f32> {
        (0..len)
            .map(|i| ((i * 7 % 11) as f32 - 5.0) * scale)
            .collect()
    }

    fn pipeline() -> (EmbeddingPipeline<CpuRuntime>, CpuClient) {
        let device = CpuDevice::new();
        let client = CpuClient::new(device.clone());
        let tokenizer = tokenizer();
        let vocab = SPECIALS.len() + WORDS;
        let config = EncoderConfig {
            vocab_size: vocab,
            hidden_size: HIDDEN,
            num_hidden_layers: 1,
            num_attention_heads: 2,
            intermediate_size: 16,
            max_position_embeddings: MAX_POSITIONS,
            ..Default::default()
        };
        let d = &device;
        let tensor = |values: Vec<f32>, shape: &[usize]| {
            Tensor::from_slice(&values, shape, d).map_err(boostr::error::Error::Numr)
        };
        let encoder = Encoder::from_weights(config, Pooling::Mean, |name| match name {
            "embeddings.word_embeddings.weight" => {
                tensor(ramp(vocab * HIDDEN, 0.1), &[vocab, HIDDEN])
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
            n if n.ends_with("LayerNorm.weight") || n == "embeddings.layer_norm.weight" => {
                tensor(vec![1.0; HIDDEN], &[HIDDEN])
            }
            n if n.ends_with(".weight") => tensor(ramp(HIDDEN * HIDDEN, 0.05), &[HIDDEN, HIDDEN]),
            _ => tensor(vec![0.0; HIDDEN], &[HIDDEN]),
        })
        .unwrap();
        (EmbeddingPipeline::new(encoder, tokenizer, device), client)
    }

    fn words(count: usize) -> String {
        (0..count)
            .map(|i| format!("w{}", i % WORDS))
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
    fn text_is_wrapped_in_cls_and_sep() {
        let (pipeline, client) = pipeline();
        let tokenizer = pipeline.tokenizer();
        let cls = tokenizer.special_token_id("[CLS]").unwrap();
        let sep = tokenizer.special_token_id("[SEP]").unwrap();
        let mut ids = vec![cls];
        ids.extend(tokenizer.encode_raw(&words(3)));
        ids.push(sep);
        let ids: Vec<i64> = ids.into_iter().map(i64::from).collect();

        let input =
            Tensor::<CpuRuntime>::from_slice(&ids, &[1, ids.len()], &CpuDevice::new()).unwrap();
        let expected = pipeline
            .encoder()
            .embed_inference(&client, &input, None)
            .unwrap()
            .to_vec::<f32>();
        let vector = embed_one(&pipeline, &client, &words(3)).unwrap();
        assert!(max_diff(&vector, &expected) < 1e-6);
    }

    #[test]
    fn long_text_embeds_as_its_first_tokens_inside_cls_and_sep() {
        let (pipeline, client) = pipeline();
        let long = embed_one(&pipeline, &client, &words(100)).unwrap();
        // `[CLS]` and `[SEP]` take two of the `MAX_POSITIONS` slots.
        let prefix = embed_one(&pipeline, &client, &words(MAX_POSITIONS - 2)).unwrap();
        let shorter = embed_one(&pipeline, &client, &words(MAX_POSITIONS - 3)).unwrap();
        assert_eq!(long.len(), HIDDEN);
        assert!(max_diff(&long, &prefix) < 1e-6);
        assert!(max_diff(&long, &shorter) > 1e-6);
    }

    #[test]
    fn vector_does_not_depend_on_other_texts() {
        let (pipeline, client) = pipeline();
        let short = words(3);
        let long = words(12);
        let alone = embed_each(&pipeline, &client, &[&short]).unwrap();
        let together = embed_each(&pipeline, &client, &[&short, &long]).unwrap();
        assert_eq!(together.len(), 2);
        assert!(max_diff(&alone[0], &together[0]) < 1e-6);
    }

    #[test]
    fn empty_text_gives_zero_vector() {
        let (pipeline, client) = pipeline();
        let vector = embed_one(&pipeline, &client, "   ").unwrap();
        assert_eq!(vector, vec![0.0; HIDDEN]);
    }
}
