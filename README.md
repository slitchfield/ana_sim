# ana_sim

A small analog circuit simulator written in Rust. It currently performs **DC operating-point analysis of linear circuits** using Modified Nodal Analysis (MNA), solving for node voltages and voltage-source branch currents.

The project is at an early stage: the solver core works and is covered by tests, but there is no command-line interface or netlist file format yet. See [Todo.md](Todo.md) for known bugs and planned cleanup.

## How it works

A circuit is described as a `Netlist` of `Component`s, each connected to integer node IDs. **Node `0` is ground.**

Solving is a two-step process:

1. `Netlist::initialize_dc_mna()` counts the non-ground nodes (`n`) and auxiliary current unknowns (`m`, one per voltage source), then assembles the MNA system `A·x = z`:

   ```
   A = | G  B |     x = | v |     z = | i |
       | C  D |         | j |         | e |
   ```

   - `G` (n×n): conductances
   - `B` (n×m), `C` (m×n): voltage-source connections
   - `D` (m×m): dependent-source coupling
   - `v`: node voltages, `j`: voltage-source branch currents
   - `i`: independent current injections, `e`: independent source voltages

   Each component contributes "stamps" (`Stamp(row, col, value)`, 1-indexed) to each block via the `DCComponent` trait in `src/lib.rs`, and the netlist sums them into `A` and `z`.

2. `Netlist::solve_dc_mna()` solves the system with nalgebra's full-pivot LU decomposition. `Netlist::get_node_voltages()` then returns `v` (entry `k-1` is the voltage at node `k`).

The method and the reference test circuit follow §3.1.5 of the [Qucs technical manual](http://qucs.github.io/docs/technical/technical.pdf).

## Supported components

| Component | Constructor | Notes |
|---|---|---|
| Resistor | `Resistor::new(a_node, b_node, resistance)` | |
| Independent voltage source | `IVoltageSource::new(source_num, positive_node, negative_node, voltage)` | Adds one auxiliary current unknown |
| Independent current source | `ICurrentSource::new(source_node, sink_node, current)` | Current flows from `source_node` through the source into `sink_node` |
| Voltage-controlled current source | `VCCurrentSource::new(sense_pos, sense_neg, source_node, sink_node, gain)` | Output current `gain · (V(sense_pos) − V(sense_neg))` |
| Current-controlled current source | `CCCurrentSource::new(dep_source_num, source_node, sink_node, gain)` | Controlled by the branch current of voltage source `dep_source_num` |
| Current-controlled voltage source | `CCVoltageSource::new(source_num, dep_source_num, _, _, positive_node, negative_node, gain)` | Adds one auxiliary current unknown; the two sensing-node arguments are currently unused |

Current-controlled sources sense current the SPICE way: insert a 0 V `IVoltageSource` in the branch to be measured and reference its `source_num`.

### Numbering rules

These are not yet validated, so violating them will panic or give wrong answers:

- Non-ground nodes must be numbered contiguously `1..=n`.
- `source_num` values across all `IVoltageSource`s and `CCVoltageSource`s must be unique and contiguous `1..=m`.

## Project layout

```
src/
  lib.rs            DCComponent trait
  netlist.rs        Netlist: MNA assembly and solve
  components/
    mod.rs          Component enum, Stamp
    resistor.rs
    independent_voltage_source.rs
    independent_current_source.rs
    vc_current_source.rs
    cc_current_source.rs
    cc_voltage_source.rs
  bin/main.rs       placeholder binary (prints "Hello, world!")
check.sh            runs check, fmt, clippy and tests
```

## Building

Requires a Rust toolchain (edition 2021) with `cargo`. Install via [rustup](https://rustup.rs) if needed.

```sh
cargo build
```

Dependencies (`nalgebra`, `assert_float_eq`) are fetched automatically.

## Running

The simulator is currently exercised through its test suite. The binary (`cargo run`) is a placeholder, and the `netlist` and `components` modules are private to the crate, so the library can't yet be used from outside it.

Run all tests:

```sh
cargo test
```

Run a single test and show the assembled `A` matrix and `z` vector (printed to stderr during assembly and solve):

```sh
cargo test netlist::tests::dc_mna_solve -- --nocapture
```

Run the full set of CI-style checks (build, `rustfmt`, `clippy -D warnings`, tests):

```sh
./check.sh
```

### Example

This is the reference circuit from the Qucs manual, as written in `src/netlist.rs` tests: a 1 V source on node 1, 5 Ω between nodes 1 and 2, 10 Ω from node 2 to ground, and a 1 A source driving current into node 2.

```rust
use crate::components::*;
use crate::netlist::Netlist;

let mut net = Netlist::new();
net.add_component(Component::IVoltageSource(
    independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0),
));
net.add_component(Component::Resistor(resistor::Resistor::new(1, 2, 5.0)));
net.add_component(Component::Resistor(resistor::Resistor::new(0, 2, 10.0)));
net.add_component(Component::ICurrentSource(
    independent_current_source::ICurrentSource::new(0, 2, 1.0),
));

net.initialize_dc_mna();
net.solve_dc_mna();

let v = net.get_node_voltages().unwrap();
// v[0] == 1.0 V (node 1), v[1] == 4.0 V (node 2)
```

To experiment with your own circuit, add a `#[test]` like this to a test module in `src/netlist.rs` and run it with `cargo test <test_name> -- --nocapture`.

## Limitations

- DC analysis of linear circuits only. No AC, transient or nonlinear analysis.
- Dense matrices only.
- No netlist parser or CLI.
- Input numbering is not validated, and errors panic rather than returning a `Result`.

See [Todo.md](Todo.md) for the full list.
