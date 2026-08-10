use candle_core::{Tensor, D};
use rand::distributions::{Distribution, WeightedIndex};
use rand::thread_rng;

pub enum SamplingStrategy {
    Greedy,
    TopK(usize),
    TopP(f32),
}

pub struct Sampler {
    strategy: SamplingStrategy,
    temperature: f64,
}

impl Sampler {
    pub fn new(strategy: SamplingStrategy, temperature: f64) -> Self {
        Self {
            strategy,
            temperature,
        }
    }

    pub fn sample(&self, logits: &Tensor) -> candle_core::Result<u32> {
        // 1. Apply Temperature
        let scaled_logits = if self.temperature != 1.0 {
            logits.affine(1.0 / self.temperature, 0.0)?
        } else {
            logits.clone()
        };

        // 2. Softmax to get probabilities
        let probs = candle_nn::ops::softmax(&scaled_logits, D::Minus1)?;
        let mut probs_vec: Vec<f32> = probs.flatten_all()?.to_vec1()?;

        // 3. Apply Sampling Strategy
        match self.strategy {
            SamplingStrategy::Greedy => {
                let max_idx = probs_vec
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                return Ok(max_idx as u32);
            }
            SamplingStrategy::TopK(k) => {
                // Zero out all but the top K probabilities
                let mut indices: Vec<usize> = (0..probs_vec.len()).collect();
                indices.sort_by(|&a, &b| {
                    probs_vec[b]
                        .partial_cmp(&probs_vec[a])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                for &i in indices.iter().skip(k) {
                    probs_vec[i] = 0.0;
                }
            }
            SamplingStrategy::TopP(p) => {
                // Nucleus sampling: keep smallest set of tokens whose cumulative prob >= p
                let mut indices: Vec<usize> = (0..probs_vec.len()).collect();
                indices.sort_by(|&a, &b| {
                    probs_vec[b]
                        .partial_cmp(&probs_vec[a])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                let mut cum_prob = 0.0;
                for &i in &indices {
                    cum_prob += probs_vec[i];
                    if cum_prob >= p {
                        break;
                    }
                }
                // Zero out tokens outside the nucleus
                let mut keep = false;
                let mut cum_check = 0.0;
                for &i in &indices {
                    if keep {
                        probs_vec[i] = 0.0;
                    }
                    cum_check += probs_vec[i];
                    if cum_check >= p {
                        keep = true;
                    }
                }
            }
        }

        // 4. Multinomial Sampling using rand
        let dist = WeightedIndex::new(&probs_vec)
            .map_err(|e| candle_core::Error::Msg(format!("WeightedIndex error: {}", e)))?;
        let mut rng = thread_rng();
        Ok(dist.sample(&mut rng) as u32)
    }
}
