use std::sync::LazyLock;
use std::sync::Mutex;

use adjectives::ADJECTIVES;
use nouns::NOUNS;
use rand::rngs::OsRng;
use rand::Rng;
mod adjectives;
mod nouns;

pub static NAME_GENERATOR: LazyLock<Mutex<NameGenerator>> =
    LazyLock::new(|| Mutex::new(NameGenerator::new()));

pub struct NameGenerator(OsRng);

impl NameGenerator {
    pub fn new() -> NameGenerator {
        NameGenerator(OsRng::default())
    }

    pub fn generate(&mut self) -> String {
        let adjective = ADJECTIVES[self.0.gen_range(0..ADJECTIVES.len())];
        let noun = NOUNS[self.0.gen_range(0..NOUNS.len())];
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        format!("{}-{}-{}", adjective, noun, time)
    }
}
