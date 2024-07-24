#![allow(warnings)]
use javy::Config;
use javy::Runtime;
fn main() {
    let js_source = "var r = /* @__PURE__ */ ((t) => (
  (t[(t.Stdin = 0)] = \"Stdin\"),
  (t[(t.Stdout = 1)] = \"Stdout\"),
  (t[(t.Stderr = 2)] = \"Stderr\"),
  t
))(r || {});

// node_modules/javy/dist/fs/index.js
function o(i) {
  let r2 = new Uint8Array(1024),
    e = 0;
  for (;;) {
    const t = Javy.IO.readSync(i, r2.subarray(e));
    if (t < 0) throw Error(\"Error while reading from file descriptor\");
    if (t === 0) return r2.subarray(0, e + t);
    if (((e += t), e === r2.length)) {
      const n = new Uint8Array(r2.length * 2);
      n.set(r2), (r2 = n);
    }
  }
}
function l(i, r2) {
  for (; r2.length > 0; ) {
    const e = Javy.IO.writeSync(i, r2);
    if (e < 0) throw Error(\"Error while writing to file descriptor\");
    if (e === 0)
      throw Error(\"Could not write all contents in buffer to file descriptor\");
    r2 = r2.subarray(e);
  }
}

// node_modules/@shopify/shopify_function/run.ts
function run_default(userfunction) {
  const input_data = o(r.Stdin);
  const input_str = new TextDecoder(\"utf-8\").decode(input_data);
  const input_obj = JSON.parse(input_str);
  const output_obj = userfunction(input_obj);
  const output_str = JSON.stringify(output_obj);
  const output_data = new TextEncoder().encode(output_str);
  l(r.Stdout, output_data);
}

// extensions/delivery-customization/src/run.ts
function run(input) {
  let operations = input.cart.deliveryGroups
    .filter((group) => {
      return group.deliveryOptions.length > 1;
    })
    .map((group) => {
      return moveFirstDeliveryOptionToLastPosition(group);
    })
    .filter((operation) => {
      return operation !== null;
    });
  return { operations };
}
function moveFirstDeliveryOptionToLastPosition(group) {
  let firstHandle;
  let lastHandleIndex;
  if (group == void 0) {
    return null;
  }
  firstHandle = group.deliveryOptions[0]?.handle;
  if (firstHandle == null) {
    return null;
  }
  lastHandleIndex = group.deliveryOptions.length - 1;
  return {
    move: {
      deliveryOptionHandle: firstHandle,
      index: lastHandleIndex,
    },
  };
}

run_default(run);
";
    let mut config = Config::default();

    let runtime = Runtime::new(config).unwrap();
    let results = runtime.compile_to_bytecode("function.mjs", js_source);
    jacc::compile(&results.unwrap()).unwrap();
}
