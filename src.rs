use std::path::PathBuf;

  use zk_engine::{
    TraceSliceValues,
    args::{WASMArgsBuilder, WASMCtx},
    run::prove_execution,
    utils::logging::init_logger,
  };

  fn main() -> anyhow::Result<()> {
    init_logger();

    // Configure the arguments needed for WASM execution
    //
    // Here we are configuring the path to the WASM file
    let args = WASMArgsBuilder::default()
      .file_path(PathBuf::from("wasm/gradient_boosting.wasm"))
      .invoke(Some(String::from("_start")))
      .trace_slice_values(TraceSliceValues::new(0, 100_000))
      .build();

    // Create a WASM execution context for proving.
    let mut wasm_ctx = WASMCtx::new_from_file(args)?;

    let batched_config = true;
    prove_execution(&mut wasm_ctx, batched_config)
  }