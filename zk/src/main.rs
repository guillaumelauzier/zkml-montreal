use ff::PrimeField;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{path::PathBuf, time::Instant};
use tracing::{debug, info};
use zk_engine::{
    args::{WASMArgsBuilder, WASMCtx},
    nova::{
        provider::{ipa_pc, PallasEngine},
        spartan::{self, snark::RelaxedR1CSSNARK},
        traits::Dual,
    },
    run::batched::public_values::PublicValues,
    run::batched::BatchedZKEProof,
    traits::{args::ZKWASMArgs, zkvm::ZKVM},
    utils::logging::init_logger,
    TraceSliceValues,
};

// Backend configs
type E1 = PallasEngine;
type EE1<E> = ipa_pc::EvaluationEngine<E>;
type EE2<E> = ipa_pc::EvaluationEngine<Dual<E>>;
type BS1<E> = spartan::batched::BatchedRelaxedR1CSSNARK<E, EE1<E>>;
type S1<E> = RelaxedR1CSSNARK<E, EE1<E>>;
type S2<E> = RelaxedR1CSSNARK<Dual<E>, EE2<E>>;

const CHUNK_SIZE: usize = 100_000;

struct ShardResult {
    proof: BatchedZKEProof<E1, BS1<E1>, S1<E1>, S2<E1>>,
    public_values: PV<E1, BS1, S1, S2>,
    elapsed_time: u64,
}

fn main() -> anyhow::Result<()> {
    init_logger();

    let start_time = Instant::now();

    let tasks_results = (0..4)
        .into_par_iter()
        .map(|i| {
            let task_start_time = Instant::now();
            info!("executing chunk {i}");

            // Configure the arguments needed for WASM execution
            //
            // Here we are configuring the path to the WASM file
            let args = WASMArgsBuilder::default()
                .file_path(PathBuf::from("wasm/gradient_boosting.wasm"))
                .invoke(Some(String::from("_start")))
                .trace_slice_values(TraceSliceValues::new(CHUNK_SIZE * i, CHUNK_SIZE * (i + 1)))
                .build();

            info!("{}", args.bytecode().map_err(|e| e.to_string())?.len());

            // Create a WASM execution context for proving.
            let mut wasm_ctx = WASMCtx::new_from_file(args).map_err(|e| e.to_string())?;

            let (proof, public_values) =
                BatchedZKEProof::<E1, BS1<E1>, S1<E1>, S2<E1>>::prove_wasm(&mut wasm_ctx)
                    .map_err(|e| e.to_string())?;

            let zi = public_values.execution().public_outputs();

            let task_end_time = Instant::now();
            let elapsed_time = (task_end_time - task_start_time).as_secs();
            info!("Task {i} wall clock time {elapsed_time} secs");
            Ok::<_, String>(ShardResult {
                proof,
                public_values,
                elapsed_time,
            })
        })
        .collect::<Vec<_>>();

    let end_time = Instant::now();
    let elapsed_time = (end_time - start_time).as_secs();

    let successful_results: Vec<_> = tasks_results
        .into_iter()
        .enumerate()
        .filter_map(|(i, r)| r.ok().map(|v| (i, v)))
        .collect();
    let tasks_elapsed_time: u64 = successful_results.iter().map(|r| r.1.elapsed_time).sum();

    for (i, proof, public_values) in successful_results
        .iter()
        .map(|r| (r.0, r.1.proof, r.1.public_values))
    {
        let pass = proof.verify(public_values)?;
        info!(
            "Shard {i} verification {}",
            if pass { "succeeded" } else { "failed" }
        );
    }

    info!("Total wall clock time: {elapsed_time} secs");
    info!("The sum of time spent by tasks: {tasks_elapsed_time} secs");

    Ok(())
}
