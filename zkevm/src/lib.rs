pub mod circuit;
pub mod io;
pub mod prover;
pub mod utils;
pub mod verifier;

pub mod proof {
    use crate::prover::AggCircuitProof;
    use serde_derive::{Deserialize, Serialize};
    use eth_types::Hash;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ZkProof {
        pub id: Hash,
        pub agg_proof: AggCircuitProof,
    }
}
