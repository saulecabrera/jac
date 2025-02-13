# ParseTrace

`parsetrace` is a crate containing functionalities to parse, manipulate, and
pretty-print dynamic execution traces for
[Javy](https://github.com/bytecodealliance/javy) produced Wasm modules.

## Dynamic Trace Format

This program takes in a matching pair of JavaScript source file, and
a structured trace file generated from a special monitor on the [Wizard Wasm
engine](https://github.com/titzer/wizard-engine).

The trace file is a CSV file. Each row represent a single QuickJS opcode
execution. The file has the following columns:

### Function ID

A dynamically assigned id for each source js function, only used
to distinguish separate function/closure executions.

### PC Offset

The detected opcode offset, w.r.t. the function start.

### Opcode

Value of the QuickJS operator

### Cost

The amount of fuel used to execute this opcode. Fuel roughly correlates
to number of Wasm instructions executed.

### Wasm Func Trace

A stack of Wasm function invocations in order to execute this
QuickJS opcode. This provides a detailed breakdown of where the engine is
spending the most fuel when executing a quickjs opcode. The value for this field
is a list of Wasm execution markers" separated by "|". Each Wasm execution
marker has the format wasm_func_id:[S|E]:c_fuel. [S|E] marks the start/end of
the Wasm function, c_fuel marks the cumulative fuel level, relative to the start
of this quickjs opcode execution.

Certain values of the "opcode" column represent special events. When opcode
= "00", it signifies Wasm engine overhead that cannot be attributed to any
quickjs opcode. When opcode = "START|END", it represents the profiled
invocation/return of js functions.

## How to generate trace file

To generate a dynamic trace, you need to first create a static Wasm binary with
Javy, with:

`javy build -C dynamic=n file_to_js -o index.wasm`

Then, run the module in Wizard with `profile_bytecode` monitor enabled:

`wizeng '--monitors=profile_bytecode{output_folder=profile_result}' index.wasm`

`profile_result` is an optional parameter to store the dynamic trace results.
3 files are generated, but only the `*_guest_trace.csv` file is used by this
crate.

## Output report

This crate is intended to be used as a library, to match the recovered dynamic
trace events to parsed bytecodes from the source file. The printed report serves
as an example of the programmatic access this crate provides. To generate
a human readable report, run:

`cargo run -p jac-utils -- trace <JS> --trace <TRACE_FILE> -o OUTPUT`

The report annotates each profiled opcode with its canonical name, includes pc
offset, fuel consumption, and opcode immediate values, when applicable. The
proper function names are also recovered from the quickjs bytecode, and function
calls are properly indented. Below is an example snippet of the report:

```
....
34: GetField2 { filter } fuel_cost: 200
39: FClosure8 { lambda_fn_8 } fuel_cost: 1270
41: CallMethod { argc: 1 } fuel_cost: 2839
FUNCTION START lambda_fn_8:
  SYSTEM COST: 253
  SYSTEM COST: 44
  1: Null fuel_cost: 27
  2: StrictNeq fuel_cost: 118
  3: Return fuel_cost: 58
FUNCTION END lambda_fn_8
SYSTEM COST: 840
FUNCTION START lambda_fn_8:
  SYSTEM COST: 253
  SYSTEM COST: 44
  1: Null fuel_cost: 27
  2: StrictNeq fuel_cost: 118
  3: Return fuel_cost: 58
FUNCTION END lambda_fn_8
SYSTEM COST: 1084
FUNCTION START lambda_fn_8:
  SYSTEM COST: 253
  SYSTEM COST: 44
  1: Null fuel_cost: 27
  2: StrictNeq fuel_cost: 118
  3: Return fuel_cost: 58
FUNCTION END lambda_fn_8
SYSTEM COST: 763
FUNCTION START lambda_fn_8:
  SYSTEM COST: 253
  SYSTEM COST: 44
  1: Null fuel_cost: 27
  2: StrictNeq fuel_cost: 118
  3: Return fuel_cost: 58
FUNCTION END lambda_fn_8
SYSTEM COST: 766
44: PutLoc0 { operations } fuel_cost: 36
45: Object fuel_cost: 431
46: GetLocCheck { operations } fuel_cost: 60
49: DefineField { operations } fuel_cost: 754
54: Return fuel_cost: 58
....
```
