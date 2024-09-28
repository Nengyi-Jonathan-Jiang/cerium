use crate::cerium_vm::{CeriumMemory, CeriumVM};

mod cerium_vm;
mod util;

type VM = CeriumVM;

fn main() {
    let mut memory = CeriumMemory::new();
    let a = memory.allocate(20).unwrap();
    let b = memory.allocate(30).unwrap();
    let c = memory.allocate(20).unwrap();
    
    println!("a = {:?}; b = {:?}, c = {:?}", a, b, c);
    
    memory.deallocate(c)
}
