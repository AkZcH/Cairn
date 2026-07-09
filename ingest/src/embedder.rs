//! Generates sentence embeddings using BAAI/bge-small-en-v1.5 (384-dim),
//! run locally via candle on CPU. Model files are loaded from disk rather
//! than fetched at runtime, see ingest/models/README.md for how to get
//! them. This avoids depending on hf-hub's download machinery, which has
//! had recurring redirect-handling bugs and describes itself as unstable.

use anyhow::{Context, Result};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use std::path::Path;
use tokenizers::Tokenizer;

pub struct Embedder {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl Embedder {
    /// Loads the model and tokenizer from a local directory containing
    /// config.json, tokenizer.json, and model.safetensors.
    pub fn load(model_dir: &Path) -> Result<Self> {
        let device = Device::Cpu;

        let config_path = model_dir.join("config.json");
        let tokenizer_path = model_dir.join("tokenizer.json");
        let weights_path = model_dir.join("model.safetensors");

        for path in [&config_path, &tokenizer_path, &weights_path] {
            if !path.exists() {
                anyhow::bail!(
                    "missing model file: {}\n\
                     Download it from https://huggingface.co/BAAI/bge-small-en-v1.5/tree/main \
                     into {}",
                    path.display(),
                    model_dir.display()
                );
            }
        }

        let config_str = std::fs::read_to_string(&config_path)
            .with_context(|| format!("reading {}", config_path.display()))?;
        let config: Config = serde_json::from_str(&config_str)?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path).map_err(anyhow::Error::msg)?;

        // SAFETY: from_mmaped_safetensors memory-maps the weights file rather
        // than loading it fully into RAM. Standard, documented candle usage.
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], DTYPE, &device)?
        };

        let model = BertModel::load(vb, &config).context("loading BERT model weights")?;

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    /// Embeds a single piece of text into a 384-dim vector: mean-pool
    /// token embeddings, then L2-normalize (BGE models expect normalized
    /// output for cosine similarity to behave correctly).
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(anyhow::Error::msg)?;

        let token_ids = encoding.get_ids();
        let token_ids = Tensor::new(token_ids, &self.device)?.unsqueeze(0)?;
        let token_type_ids = token_ids.zeros_like()?;

        let output = self.model.forward(&token_ids, &token_type_ids, None)?;

        let (_batch, seq_len, _hidden) = output.dims3()?;
        let pooled = (output.sum(1)? / (seq_len as f64))?;

        let norm = pooled.sqr()?.sum_keepdim(1)?.sqrt()?;
        let normalized = pooled.broadcast_div(&norm)?;

        let vec: Vec<f32> = normalized.squeeze(0)?.to_vec1()?;
        Ok(vec)
    }
}