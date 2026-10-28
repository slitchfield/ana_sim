# Todo

## Bugs
- [ ] Fix z-matrix stamps overwriting instead of accumulating (`=` → `+=` in `netlist.rs` z assembly); two current sources into one node currently clobber each other
- [ ] Fix `solve_dc_mna` accumulating into `x_mat` (`+=`), which doubles results on a second solve; assign instead
- [ ] Remove the dead local `z_mat` in `initialize_dc_mna` (never stamped, added as zeros)
- [ ] Fix `cc_voltage_source::tests::basic_function_ungrounded`, which builds a VCCS instead of a CCVS

## Correctness guards
- [ ] Validate node numbering is contiguous 1..n (node IDs are used directly as matrix indices)
- [ ] Validate `source_num` across voltage sources/CCVSs is exactly 1..m and unique; validate `dep_source_num` references an existing source
- [ ] Implement the existing "sensing/output nodes don't overlap illegally" TODOs (VCCS, CCCS, CCVS)
- [ ] Return a `Result` from `solve_dc_mna` instead of `assert!`/`.expect()`

## Cleanup
- [ ] Route z-matrix assembly through `DCComponent::get_zmat_stamps` like G/B/C/D instead of matching on variants
- [ ] Move per-component node/aux-variable knowledge into the components (e.g. `nodes()` / `aux_count()`) instead of hard-coding in `num_nodes`/`num_aux_variables`
- [ ] Factor the repeated stamp-into-block loop in `initialize_dc_mna` into a helper
- [ ] Remove unused CCVS fields `source_sensing_node`/`sink_sensing_node`, commented-out field names, and `1.0 as _` casts
- [ ] Remove or gate debug `eprintln!` calls
- [x] Drop `Box<DMatrix>` wrappers (`DMatrix` is already heap-allocated)
- [ ] Delete template `add()` + its test in `lib.rs` and the dead `components/junction.rs`
- [ ] Delete stale "handle dependent sources" TODOs in `netlist.rs`
- [ ] Fix comments: z is (N+M)×1; IVS C-matrix comment should say M×N and "C" not "B"
- [ ] Mark test modules `#[cfg(test)]`; remove `#[allow(unused_imports)]` and unused `Arc` imports; audit `#[allow(dead_code)]`
- [ ] `Cargo.toml`: move `assert_float_eq` to `[dev-dependencies]`; rename `mylib`/`mybin`

## Tests
- [ ] Extract a helper for the Qucs example circuit duplicated across netlist tests; merge the identical, assertion-free `adding_components` and `simple_vs_resistor_test`
- [ ] Add regression tests for the z-stamp accumulation and double-solve bugs
- [ ] Add an ungrounded CCCS test
- [ ] Add tests for the correctness guards above
