use errs::*;
use std::fs::read;

fn bar() -> R<Vec<u8>> {
    //Ok(Err::<(), ()>(())?);
    //None::<()>?;
    O(read("path")?)
}

fn foo() -> R<Vec<u8>> {
    O(bar()?)
}

fn main() {
    //read("path").unwrap();
    print!("{}", foo().unwrap_err());
    foo().unwrap();
}
