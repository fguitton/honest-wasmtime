#![cfg(all(
    feature = "component-model-memory64-only",
    feature = "cranelift",
    feature = "wat"
))]

use wasmtime::component::{Component, Linker, Val};
use wasmtime::{Engine, Result, Store};

// Upstream's seven dynamic-Val amplification cases, with 64-bit canonical ABI
// list headers. Records and variants must charge host allocation, not only the
// compact guest field widths. No WASI or asynchronous component feature is used.
#[test]
fn hostcall_fuel_accounts_for_memory64_val_allocations() -> Result<()> {
    let engine = Engine::default();
    let component = Component::new(&engine, include_str!("memory64_hostcall_fuel.wat"))?;
    for name in ["f1", "f2", "f3", "f4", "f5", "f6", "f7"] {
        for limited in [false, true] {
            let mut store = Store::new(&engine, ());
            if limited {
                store.set_hostcall_fuel(1000);
            }
            let instance = Linker::new(&engine).instantiate(&mut store, &component)?;
            let function = instance.get_func(&mut store, name).unwrap();
            let mut results = [Val::Bool(false)];
            let result = function.call(&mut store, &[], &mut results);
            if limited {
                let error = result.expect_err("host allocation amplification must exhaust fuel");
                assert!(format!("{error:?}").contains("fuel"), "{name}: {error:?}");
            } else {
                result?;
                assert!(matches!(&results[0], Val::List(values) if values.len() == 10));
                function.post_return(&mut store)?;
            }
        }
    }
    Ok(())
}
