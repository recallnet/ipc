// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod json {
    use fendermint_materializer::manifest::Manifest;
    use quickcheck::Arbitrary;
    use recall_fendermint_testing::golden_json;
    golden_json! { "manifest/json", manifest, Manifest::arbitrary }
}

mod yaml {
    use fendermint_materializer::manifest::Manifest;
    use quickcheck::Arbitrary;
    use recall_fendermint_testing::golden_yaml;
    golden_yaml! { "manifest/yaml", manifest, Manifest::arbitrary }
}

mod toml {
    use fendermint_materializer::manifest::Manifest;
    use quickcheck::Arbitrary;
    use recall_fendermint_testing::golden_toml;
    golden_toml! { "manifest/toml", manifest, Manifest::arbitrary }
}
