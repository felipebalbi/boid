#![no_std]
#![feature(abi_efiapi)]

extern crate alloc;

mod boid;

use crate::boid::Boid;
use alloc::vec::Vec;
use core::mem;
use log::debug;
use micromath::vector::F32x2;
use uefi::proto::console::gop::{BltOp, BltPixel, GraphicsOutput};
use uefi::proto::rng::Rng;
use uefi::table::boot::BootServices;
use uefi::Result;

pub fn run(bt: &BootServices) -> Result {
    debug!("Starting Boid Simulation");

    // Open graphics output protocol.
    let gop_handle = bt.get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = bt.open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    // Open random number generator protocol.
    let rng_handle = bt.get_handle_for_protocol::<Rng>()?;
    let mut rng = bt.open_protocol_exclusive::<Rng>(rng_handle)?;

    // Get screen resolution
    let (width, height) = gop.current_mode_info().resolution();
    let mut boids = Vec::new();

    for i in 0..100 {
        let posx = get_random_usize_range(&mut rng, 0, width) as f32;
        let posy = get_random_usize_range(&mut rng, 0, height) as f32;

        let velx = get_random_usize_range(&mut rng, 2, 4) as f32;
        let vely = get_random_usize_range(&mut rng, 2, 4) as f32;

        if posx == 0.0 || posy == 0.0 {
            debug!("Boid {} will be stuck at {} x {}", i, posx, posy);
        }

        let position = F32x2 { x: posx, y: posy };

        let velocity = F32x2 { x: velx, y: vely };

        let boid = Boid::new(position, velocity);
        boids.push(boid);
    }

    loop {
        let flock = boids.clone();

        clear_screen(&mut gop, width, height)?;
        for boid in boids.iter_mut() {
            boid.flock(&flock);
            boid.update(width, height);
            boid.draw(&mut gop)?;
        }
        bt.stall(16_667);
    }
}

fn clear_screen(gop: &mut GraphicsOutput, width: usize, height: usize) -> Result {
    let color = BltPixel::new(0, 0, 0);

    gop.blt(BltOp::VideoFill {
        color,
        dest: (0, 0),
        dims: (width, height),
    })
}

fn get_random_usize(rng: &mut Rng) -> usize {
    let mut buf = [0; mem::size_of::<usize>()];
    rng.get_rng(None, &mut buf).expect("get_rng failed");
    usize::from_le_bytes(buf)
}

fn get_random_usize_range(rng: &mut Rng, start: usize, end: usize) -> usize {
    let number = get_random_usize(rng);
    (number % (end - start + 1)) + start
}
