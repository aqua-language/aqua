use adjectives::ADJECTIVES;
use nouns::NOUNS;
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::hash::Hasher;

mod adjectives;
mod nouns;

fn rng() -> u64 {
    RandomState::new().build_hasher().finish()
}

pub fn generate_name() -> String {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let adjective = ADJECTIVES[rng() as usize % ADJECTIVES.len()];
    let noun = NOUNS[rng() as usize % NOUNS.len()];
    format!("{}-{}-{}", adjective, noun, time)
}
