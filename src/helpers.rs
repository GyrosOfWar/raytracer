use once_cell::sync::Lazy;
use rand::{rngs::SmallRng, Rng, SeedableRng};

static RNG: Lazy<SmallRng> = Lazy::new(|| SmallRng::from_entropy());
