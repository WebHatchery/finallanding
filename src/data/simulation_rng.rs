use macroquad_toolkit::rng::LegacyLcg64;

#[derive(Clone, Debug)]
pub struct SimulationRng {
    stream: LegacyLcg64<1>,
}

impl Default for SimulationRng {
    fn default() -> Self {
        Self::with_seed(0x5446_4c5f_4352_4153)
    }
}

impl SimulationRng {
    pub fn with_seed(seed: u64) -> Self {
        Self {
            stream: LegacyLcg64::new(seed),
        }
    }

    pub fn range_i32(&mut self, min: i32, max: i32) -> i32 {
        assert!(min < max, "invalid random range");
        min + (self.next_u32() % (max - min) as u32) as i32
    }

    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        assert!(min < max, "invalid random range");
        let unit = self.next_u32() as f32 / u32::MAX as f32;
        min + (max - min) * unit
    }

    fn next_u32(&mut self) -> u32 {
        self.stream.next_u32()
    }
}

#[cfg(test)]
mod tests;
