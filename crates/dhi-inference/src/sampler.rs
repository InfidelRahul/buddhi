use candle_core::{Result, Tensor};
use rand::seq::SliceRandom;

pub enum SamplingStrategy {
    Greedy,
    TopK(usize),
    TopP(f64),
}

pub struct Sampler {
    strategy: SamplingStrategy,
}

impl Sampler {
    pub fn new(strategy: SamplingStrategy) -> Self {
        Self { strategy }
    }

    pub fn sample(&self, logits: &Tensor) -> Result<u32> {
        let logits_vec: Vec<f32> = logits.flatten_all()?.to_vec1()?;

        match self.strategy {
            SamplingStrategy::Greedy => {
                // Find argmax
                let mut max_idx = 0;
                let mut max_val = f32::MIN;
                for (i, &val) in logits_vec.iter().enumerate() {
                    if val > max_val {
                        max_val = val;
                        max_idx = i;
                    }
                }
                Ok(max_idx as u32)
            }
            SamplingStrategy::TopK(k) => {
                // Simplified Top-K: sort indices by logit value
                let mut indices: Vec<usize> = (0..logits_vec.len()).collect();
                indices.sort_by(|&a, &b| {
                    logits_vec[b]
                        .partial_cmp(&logits_vec[a])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let top_k: Vec<usize> = indices.into_iter().take(k).collect();

                // Simple uniform sampling from top-k for skeleton
                let mut rng = rand::thread_rng();
                let chosen = top_k.choose(&mut rng).unwrap_or(&0);
                Ok(*chosen as u32)
            }
            SamplingStrategy::TopP(_p) => {
                // Placeholder for Top-P (nucleus) sampling
                // For skeleton, fall back to greedy
                let mut max_idx = 0;
                let mut max_val = f32::MIN;
                for (i, &val) in logits_vec.iter().enumerate() {
                    if val > max_val {
                        max_val = val;
                        max_idx = i;
                    }
                }
                Ok(max_idx as u32)
            }
        }
    }
}
