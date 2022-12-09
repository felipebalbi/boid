#![no_std]
#![feature(abi_efiapi)]

extern crate alloc;

use alloc::vec::Vec;
use core::mem;
use log::debug;
use uefi::proto::console::gop::{BltOp, BltPixel, GraphicsOutput};
use uefi::proto::rng::Rng;
use uefi::table::boot::BootServices;
use uefi::Result;

struct Rectangle {
    width: usize,
    height: usize,
}

struct Vec2D {
    x: isize,
    y: isize,
}

impl Default for Vec2D {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Vec2D {
    fn add(&mut self, other: &Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

struct Boid {
    body: Rectangle,
    position: Vec2D,
    velocity: Vec2D,
    acceleration: Vec2D,
}

impl Boid {
    fn new(position: Vec2D, velocity: Vec2D) -> Self {
        Self {
            body: Rectangle {
                width: 5,
                height: 5,
            },
            acceleration: Default::default(),
            position,
            velocity,
        }
    }

    fn edges(&mut self, width: usize, height: usize) {
        if (self.position.x + self.body.width as isize) >= width as isize {
            self.position.x = 0;
        } else if (self.position.x) <= 0 {
            self.position.x = width as isize;
        }

        if (self.position.y + self.body.height as isize) >= height as isize {
            self.position.y = 0;
        } else if (self.position.y) <= 0 {
            self.position.y = height as isize;
        }
    }

    fn alignment(&self, _boids: &[Boid]) {
        // let perception = 25;
        // let steering = Vec2D::default();

        // let total = 0;

        // for boid in boids.iter() {
        //     let d =
        // }
    }

    fn separation(&self) {}

    fn cohesion(&self) {}

    fn flock(&self) {}

    fn update(&mut self, width: usize, height: usize) {
        self.position.add(&self.velocity);
        self.velocity.add(&self.acceleration);

        self.edges(width, height);
    }

    fn draw(&self, gop: &mut GraphicsOutput) -> Result {
        let color = BltPixel::new(255, 0, 0);

        gop.blt(BltOp::VideoFill {
            color,
            dest: (self.position.x as usize, self.position.y as usize),
            dims: (self.body.width, self.body.height),
        })?;

        Ok(())
    }
}

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

    for _ in 0..100 {
        let position = Vec2D {
            x: get_random_usize_range(&mut rng, 0, width) as isize,
            y: get_random_usize_range(&mut rng, 0, height) as isize,
        };

        let velocity = Vec2D {
            x: get_random_usize_range(&mut rng, 2, 4) as isize,
            y: get_random_usize_range(&mut rng, 2, 4) as isize,
        };

        let boid = Boid::new(position, velocity);
        boids.push(boid);
    }

    loop {
        clear_screen(&mut gop, width, height)?;

        for boid in boids.iter_mut() {
            boid.update(width, height);
            boid.draw(&mut gop)?;
        }
        bt.stall(16_667);
    }

    // bt.stall(1_000_000);

    // Ok(())
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
    let mut buf = [0; mem::size_of::<usize>()];
    rng.get_rng(None, &mut buf).expect("get_rng failed");
    let number = usize::from_le_bytes(buf);
    (number % (end - start + 1)) + start
}
