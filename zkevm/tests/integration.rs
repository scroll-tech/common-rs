use std::sync::Once;

use types::eth::BlockResult;
use zkevm::{
    circuit::TargetCircuit,
    prover::Prover,
    utils::{get_block_result_from_file, read_env_var},
};

const PARAMS_DIR: &str = "/home/ubuntu/common-rs/zkevm/release-0711/test_params";
const SEED_PATH: &str = "/home/ubuntu/common-rs/zkevm/release-0711/test_seed";
const ALL_TESTS: &[&str] = &[
    "empty", "greeter", "multiple", "native", "single", "dao", "nft", "sushi",
];

static ENV_LOGGER: Once = Once::new();

fn parse_trace_path_from_env(mode: &str) -> &'static str {
    let trace_path = match mode {
        "empty" => "./tests/trace-empty.json",
        "greeter" => "./tests/trace-greeter.json",
        "multiple" => "./tests/trace-multiple-erc20.json",
        "native" => "./tests/trace-native-transfer.json",
        "single" => "./tests/trace-single-erc20.json",
        "single_legacy" => "./tests/trace-single-erc20-legacy.json",
        "dao" => "./tests/trace-dao.json",
        "nft" => "./tests/trace-nft.json",
        "sushi" => "./tests/trace-masterchef.json",
        _ => "./tests/trace-multiple-erc20.json",
    };
    log::info!("using mode {:?}, testing with {:?}", mode, trace_path);
    trace_path
}

fn init() {
    dotenv::dotenv().ok();
    ENV_LOGGER.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    });
}

#[test]
fn estimate_circuit_rows() {
    use zkevm::{
        circuit::{self, TargetCircuit, DEGREE},
        utils::{load_or_create_params, load_or_create_seed},
    };

    init();
    let _ = load_or_create_params(PARAMS_DIR, *DEGREE).unwrap();
    let _ = load_or_create_seed(SEED_PATH).unwrap();

    let block_result = load_block_result_for_test();

    log::info!("estimating used rows for current block");
    log::info!(
        "evm circuit: {}",
        circuit::EvmCircuit::estimate_rows(&block_result)
    );
    log::info!(
        "state circuit: {}",
        circuit::StateCircuit::estimate_rows(&block_result)
    );
    log::info!(
        "storage circuit: {}",
        circuit::ZktrieCircuit::estimate_rows(&block_result)
    );
    log::info!(
        "hash circuit: {}",
        circuit::PoseidonCircuit::estimate_rows(&block_result)
    );
}

#[cfg(feature = "prove_verify")]
#[test]
fn test_evm_prove_verify() {
    use zkevm::circuit::EvmCircuit;
    test_target_circuit_prove_verify::<EvmCircuit>();
}

#[cfg(feature = "prove_verify")]
#[test]
fn test_state_prove_verify() {
    use zkevm::circuit::StateCircuit;
    test_target_circuit_prove_verify::<StateCircuit>();
}

#[cfg(feature = "prove_verify")]
#[test]
fn test_storage_prove_verify() {
    use zkevm::circuit::ZktrieCircuit;
    test_target_circuit_prove_verify::<ZktrieCircuit>();
}

#[cfg(feature = "prove_verify")]
#[test]
fn test_hash_prove_verify() {
    use zkevm::circuit::PoseidonCircuit;
    test_target_circuit_prove_verify::<PoseidonCircuit>();
}

fn test_mock_prove_all_with_circuit<C: TargetCircuit>() {
    for test_case_name in ALL_TESTS {
        log::info!("test {} with circuit {}", test_case_name, C::name());
        let trace_path = parse_trace_path_from_env(test_case_name);
        let block_result = get_block_result_from_file(trace_path);
        log::info!(
            "test {} with circuit {} result: {:?}",
            test_case_name,
            C::name(),
            Prover::mock_prove_target_circuit::<C>(&block_result, false)
        );
    }
}

#[cfg(feature = "prove_verify")]
#[test]
fn test_mock_prove_all_target_circuits() {
    use zkevm::circuit::{EvmCircuit, PoseidonCircuit, StateCircuit, ZktrieCircuit};

    init();
    test_mock_prove_all_with_circuit::<EvmCircuit>();
    test_mock_prove_all_with_circuit::<StateCircuit>();
    test_mock_prove_all_with_circuit::<ZktrieCircuit>();
    test_mock_prove_all_with_circuit::<PoseidonCircuit>();
}

#[cfg(feature = "prove_verify")]
#[test]
fn test_state_evm_connect() {
    // TODO: better code reuse
    use std::time::Instant;

    use halo2_proofs::{
        pairing::bn256::G1Affine,
        transcript::{Challenge255, PoseidonRead, TranscriptRead},
    };
    use zkevm::{
        circuit::{EvmCircuit, StateCircuit, DEGREE},
        prover::Prover,
        utils::{get_block_result_from_file, load_or_create_params, load_or_create_seed},
        verifier::Verifier,
    };

    init();

    log::info!("loading setup params");
    let _ = load_or_create_params(PARAMS_DIR, *DEGREE).unwrap();
    let _ = load_or_create_seed(SEED_PATH).unwrap();

    let trace_path = parse_trace_path_from_env("greeter");
    let block_result = get_block_result_from_file(trace_path);

    let mut prover = Prover::from_fpath(PARAMS_DIR, SEED_PATH);
    let verifier = Verifier::from_fpath(PARAMS_DIR, None);

    log::info!("start generating state_circuit proof");
    let now = Instant::now();
    let state_proof = prover
        .create_target_circuit_proof::<StateCircuit>(&block_result)
        .unwrap();
    log::info!(
        "finish generating state_circuit proof, elapsed: {:?}",
        now.elapsed()
    );

    log::info!("start verifying state_circuit proof");
    let now = Instant::now();
    assert!(verifier
        .verify_target_circuit_proof::<StateCircuit>(&state_proof)
        .is_ok());
    log::info!(
        "finish verifying state_circuit proof, elapsed: {:?}",
        now.elapsed()
    );

    log::info!("start generating evm_circuit proof");
    let now = Instant::now();
    let evm_proof = prover
        .create_target_circuit_proof::<EvmCircuit>(&block_result)
        .unwrap();
    log::info!(
        "finish generating evm_circuit proof, cost {:?}",
        now.elapsed()
    );

    log::info!("start verifying evm_circuit proof");
    let now = Instant::now();
    assert!(verifier
        .verify_target_circuit_proof::<EvmCircuit>(&evm_proof)
        .is_ok());
    log::info!(
        "finish verifying evm_circuit proof, cost {:?}",
        now.elapsed()
    );

    let rw_commitment_state = {
        let mut transcript =
            PoseidonRead::<_, _, Challenge255<G1Affine>>::init(&state_proof.proof[..]);
        transcript.read_point().unwrap()
    };
    log::info!("rw_commitment_state {:?}", rw_commitment_state);

    let rw_commitment_evm = {
        let mut transcript =
            PoseidonRead::<_, _, Challenge255<G1Affine>>::init(&evm_proof.proof[..]);
        transcript.read_point().unwrap()
    };
    log::info!("rw_commitment_evm {:?}", rw_commitment_evm);

    assert_eq!(rw_commitment_evm, rw_commitment_state);
    log::info!("Same commitment! Test passes!");
}

fn test_target_circuit_prove_verify<C: TargetCircuit>() {
    use std::time::Instant;

    use zkevm::{
        circuit::DEGREE,
        prover::Prover,
        utils::{load_or_create_params, load_or_create_seed},
        verifier::Verifier,
    };

    init();

    let block_result = load_block_result_for_test();

    let _ = load_or_create_params(PARAMS_DIR, *DEGREE).unwrap();
    let _ = load_or_create_seed(SEED_PATH).unwrap();

    log::info!("start generating {} proof", C::name());
    let now = Instant::now();
    let mut prover = Prover::from_fpath(PARAMS_DIR, SEED_PATH);
    let proof = prover
        .create_target_circuit_proof::<C>(&block_result)
        .unwrap();
    log::info!("finish generating proof, elapsed: {:?}", now.elapsed());

    log::info!("start verifying proof");
    let now = Instant::now();
    let verifier = Verifier::from_fpath(PARAMS_DIR, None);
    assert!(verifier.verify_target_circuit_proof::<C>(&proof).is_ok());
    log::info!("finish verifying proof, elapsed: {:?}", now.elapsed());
}

fn load_block_result_for_test() -> BlockResult {
    let mut trace_path = read_env_var("TRACE_FILE", "".to_string());
    if trace_path.is_empty() {
        trace_path =
            parse_trace_path_from_env(&read_env_var("MODE", "multiple".to_string())).to_string();
    }
    get_block_result_from_file(trace_path)
}
