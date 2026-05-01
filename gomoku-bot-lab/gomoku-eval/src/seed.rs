const SPLITMIX_GOLDEN_RATIO: u64 = 0x9e37_79b9_7f4a_7c15;

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(SPLITMIX_GOLDEN_RATIO);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

pub fn derive_seed(base_seed: u64, components: impl IntoIterator<Item = u64>) -> u64 {
    let mut seed = splitmix64(base_seed);
    for component in components {
        seed = splitmix64(seed ^ component);
    }
    seed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_seed_is_stable_and_order_sensitive() {
        assert_eq!(derive_seed(7, [1, 2, 3]), derive_seed(7, [1, 2, 3]));
        assert_ne!(derive_seed(7, [1, 2, 3]), derive_seed(7, [3, 2, 1]));
        assert_ne!(derive_seed(7, [1, 2, 3]), derive_seed(8, [1, 2, 3]));
    }
}
