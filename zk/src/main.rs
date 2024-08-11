use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{path::PathBuf, time::Instant};
use tracing::info;
use zk_engine::{
    args::{WASMArgsBuilder, WASMCtx},
    run::prove_execution,
    traits::args::ZKWASMArgs,
    utils::logging::init_logger,
    TraceSliceValues,
};

const CHUNK_SIZE: usize = 100_000;

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

            let batched_config = true;
            prove_execution(&mut wasm_ctx, batched_config).map_err(|e| e.to_string())?;

            let task_end_time = Instant::now();
            let task_elapsed_time = (task_end_time - task_start_time).as_secs();
            info!("Task {i} wall clock time {task_elapsed_time} secs");
            Ok::<u64, String>(task_elapsed_time)
        })
        .collect::<Vec<_>>();

    let tasks_elapsed_time: u64 = tasks_results.into_iter().flat_map(Result::ok).sum();

    let end_time = Instant::now();
    let elapsed_time = (end_time - start_time).as_secs();
    info!("Total wall clock time: {elapsed_time} secs");
    info!("The sum of time spent by tasks: {tasks_elapsed_time} secs");

    Ok(())
}
