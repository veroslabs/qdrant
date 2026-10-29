# Maintained dependency-only patch

This is the published `permutation_iterator` 0.1.2 crate from
https://crates.io/crates/permutation_iterator/0.1.2, repository
https://github.com/asimihsan/permutation-iterator-rs, commit
`ab5e5cbf525e081823b467586d44a8e2f7c2aa72`. The original Apache-2.0
LICENSE, author attribution, README, source, examples and upstream tests are
retained. Published archive SHA256:
`b55405179fe06e4e3820ddaf9f9b51cdff9e7496af9554acdb2b1921a86ca9cb`.
`upstream-sha256.json` records the original files, including both manifests.

The sole upstream implementation change is dependency metadata in normalized
Cargo.toml: rand 0.7.2 becomes 0.8.8 (compatible 0.8 security floor).
Cargo.toml.orig remains the untouched upstream provenance copy. The root
Cargo patch selects this maintained local source under the original package
name/version; Cargo.lock records the actual patched rand. No advisory is
dismissed and no package is hidden or renamed.

The source uses rand only to obtain a 32-byte thread_rng key; that API remains
available in 0.8. The Feistel/Blake2 algorithm, constructors, iterator and fixed
key outputs are byte-identical. Seeded output fixtures are generated with the
published unmodified crate, then checked against this dependency patch. New
checks are separate from upstream tests. Random constructors are checked for
range, completeness and uniqueness, not reproducible OS entropy. This is not
a new cryptographic assurance claim.

Maintain this patch until an upstream maintained release preserves the same API
and algorithm while using a supported rand. Re-evaluate upstream changes and
remove this directory and root patch together only after equivalent consumer
tests pass. Do not substitute the unreleased upstream algorithm rewrite.

Verification: `cargo test -p permutation_iterator --locked`,
`cargo test -p quantization --lib --locked`; the upstream ignored statistical
test remains explicitly opt-in, as it was in the published crate. No server,
production database or deployment is required.

To reproduce the golden fixture without using this patched dependency, unpack
the hash-verified published crate into a temporary directory, copy
`tools/generate-security-golden.rs` to its `examples/security_golden.rs`,
resolve rand to 0.7.3 (the prior Qdrant lock version), and run
`cargo run --manifest-path <temporary>/Cargo.toml --example security_golden`.
Compare stdout byte-for-byte with `tests/security-golden.txt`. It contains
2,368 domain/key pairs: domains 1..64 plus 65,127,128,129,255,256,257,1023,1024,1025,
and keys 0..31. The regression compares every output in order for both fixed-key
constructors and verifies uniqueness/range. Random constructor checks are
separate and do not require equal entropy-derived keys.
