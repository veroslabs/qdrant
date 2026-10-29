//! Security dependency patch regressions, separate from byte-identical upstream tests.
use permutation_iterator::{Permutor, RandomPairPermutor};
use std::collections::HashSet;

#[test]
fn published_fixed_key_permutations_remain_exact() {
    let mut cases = 0;
    for line in include_str!("security-golden.txt").lines() {
        let mut fields = line.splitn(3, ':');
        let max: u64 = fields.next().unwrap().parse().unwrap();
        let key: u64 = fields.next().unwrap().parse().unwrap();
        let expected: Vec<u64> = fields.next().unwrap().split(',').map(|v| v.parse().unwrap()).collect();
        let actual: Vec<_> = Permutor::new_with_u64_key(max, key).collect();
        assert_eq!(actual, expected, "published baseline max={max}, key={key}");
        assert_eq!(Permutor::new_with_slice_key(max, permutation_iterator::u64_to_32slice(key)).collect::<Vec<_>>(), expected);
        assert_eq!(actual.iter().copied().collect::<HashSet<_>>().len(), max as usize);
        assert!(actual.iter().all(|value| *value < max));
        cases += 1;
    }
    assert_eq!(cases, 2368);
}

#[test]
fn entropy_seeded_constructors_keep_permutation_contracts() {
    for max in [1, 2, 3, 10, 65, 257, 1025] {
        for _ in 0..8 {
            let mut values: Vec<_> = Permutor::new(max).collect();
            values.sort_unstable();
            assert_eq!(values, (0..max).collect::<Vec<_>>());
        }
    }
    let pairs: HashSet<_> = RandomPairPermutor::new(7, 13).collect();
    assert_eq!(pairs.len(), 91);
    assert!(pairs.iter().all(|(a,b)| *a < 7 && *b < 13));
}
