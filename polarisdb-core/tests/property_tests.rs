use polarisdb_core::distance::{dot_product, euclidean_distance_squared};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_dot_product_optim_matches_naive(
        // Constrain to typical normalized embedding range [-1.0, 1.0]
        a in proptest::collection::vec(-1.0f32..1.0f32, 0..100),
        b in proptest::collection::vec(-1.0f32..1.0f32, 0..100)
    ) {
        let len = std::cmp::min(a.len(), b.len());
        let a = &a[..len];
        let b = &b[..len];

        let optim = dot_product(a, b);
        let naive: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

        prop_assert!((optim - naive).abs() < 1e-4);
    }

    #[test]
    fn test_euclidean_optim_matches_naive(
        a in proptest::collection::vec(-1.0f32..1.0f32, 0..100),
        b in proptest::collection::vec(-1.0f32..1.0f32, 0..100)
    ) {
        let len = std::cmp::min(a.len(), b.len());
        let a = &a[..len];
        let b = &b[..len];

        let optim = euclidean_distance_squared(a, b);

        let naive: f32 = a.iter()
            .zip(b.iter())
            .map(|(x, y)| {
                let diff = x - y;
                diff * diff
            })
            .sum();

        prop_assert!((optim - naive).abs() < 1e-3);
    }

    #[test]
    fn test_cosine_optim_matches_naive(
        a in proptest::collection::vec(-1.0f32..1.0f32, 0..100),
        b in proptest::collection::vec(-1.0f32..1.0f32, 0..100)
    ) {
        let len = std::cmp::min(a.len(), b.len());
        let a = &a[..len];
        let b = &b[..len];

        // Skip zero vectors to avoid NaN
        if dot_product(a, a) < 1e-6 || dot_product(b, b) < 1e-6 {
            return Ok(());
        }

        use polarisdb_core::distance::cosine_distance;
        let optim = cosine_distance(a, b);

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        let naive = if norm_a * norm_b == 0.0 {
            1.0
        } else {
            1.0 - (dot / (norm_a * norm_b))
        };

        prop_assert!((optim - naive).abs() < 1e-4);
    }
}
