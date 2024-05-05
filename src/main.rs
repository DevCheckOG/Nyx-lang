#![cfg(target_arch = "x86_64")]

#[global_allocator]
static ALLOC: rpmalloc::RpMalloc = rpmalloc::RpMalloc;

mod lang;

fn main() {
    lang::Nyx.run();
}
