#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LevelFrame {
    pub peak: f32,
    pub mean: f32,
    pub generation: u64,
    pub lost_samples: u64,
}

impl LevelFrame {
    pub fn from_pcm(samples: &[i16], generation: u64, lost_samples: u64) -> Self {
        if samples.is_empty() {
            return Self {
                peak: 0.0,
                mean: 0.0,
                generation,
                lost_samples,
            };
        }
        let scale = i16::MAX as f32;
        let mut peak = 0.0_f32;
        let mut sum = 0.0_f64;
        for sample in samples {
            let value = f32::from(*sample).abs() / scale;
            peak = peak.max(value);
            sum += f64::from(value);
        }
        Self {
            peak: peak.min(1.0),
            mean: (sum / samples.len() as f64) as f32,
            generation,
            lost_samples,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_bounded_level_and_keeps_loss_count() {
        let level = LevelFrame::from_pcm(&[0, i16::MAX, -(i16::MAX / 2)], 7, 1);
        assert_eq!(level.peak, 1.0);
        assert!((0.49..0.51).contains(&level.mean));
        assert_eq!((level.generation, level.lost_samples), (7, 1));
    }
}
