#![no_std]
#![feature(abi_efiapi)]

extern crate alloc;

mod boid;
mod simulation;

use crate::simulation::Simulation;
use core::mem;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::rng::Rng;
use uefi::table::boot::BootServices;
use uefi::Result;

pub fn run(bt: &BootServices) -> Result {
    // Open graphics output protocol.
    let gop_handle = bt.get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = bt.open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    // Open random number generator protocol.
    let rng_handle = bt.get_handle_for_protocol::<Rng>()?;
    let mut rng = bt.open_protocol_exclusive::<Rng>(rng_handle)?;

    // Get screen resolution
    let (width, height) = gop.current_mode_info().resolution();

    // Instantiate our Simulation
    let mut simulation = Simulation::new(&mut rng, width, height);
    simulation.run(bt, &mut gop)
}

fn get_random_usize(rng: &mut Rng) -> usize {
    let mut buf = [0; mem::size_of::<usize>()];
    rng.get_rng(None, &mut buf).expect("get_rng failed");
    usize::from_le_bytes(buf)
}

pub fn get_random_usize_range(rng: &mut Rng, start: usize, end: usize) -> usize {
    let number = get_random_usize(rng);
    (number % (end - start + 1)) + start
}
