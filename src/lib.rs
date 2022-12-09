#![no_std]
#![feature(abi_efiapi)]

extern crate alloc;

use alloc::vec::Vec;
use core::mem;
use log::debug;
use micromath::vector::{F32x2, Vector};
use uefi::proto::console::gop::{BltOp, BltPixel, GraphicsOutput};
use uefi::proto::rng::Rng;
use uefi::table::boot::BootServices;
use uefi::Result;

#[derive(PartialEq, Clone)]
struct Rectangle {
    width: f32,
    height: f32,
}

#[derive(PartialEq, Clone)]
struct Boid {
    body: Rectangle,
    position: F32x2,
    velocity: F32x2,
    acceleration: F32x2,
}

const MAX_FORCE: f32 = 0.2;
const MAX_VELOCITY: f32 = 5.0;

impl Boid {
    fn new(position: F32x2, velocity: F32x2) -> Self {
        Self {
            body: Rectangle {
                width: 5.0,
                height: 5.0,
            },
            acceleration: Default::default(),
            position,
            velocity,
        }
    }

    fn edges(&mut self, width: usize, height: usize) {
        if (self.position.x + self.body.width) >= width as f32 {
            self.position.x = self.body.width;
        } else if (self.position.x) <= 0.0 {
            self.position.x = width as f32 - self.body.width;
        }

        if (self.position.y + self.body.height) >= height as f32 {
            self.position.y = self.body.height;
        } else if (self.position.y) <= self.body.height {
            self.position.y = height as f32 - self.body.height;
        }
    }

    fn alignment(&self, boids: &[Boid]) -> F32x2 {
        let perception_radius = 25.0;
        let mut steering = F32x2::default();
        let mut total = 0.0;

        for boid in boids.iter() {
            let d = self.position.distance(boid.position);

            if boid != self && d < perception_radius {
                steering += boid.velocity;
                total += 1.0;
            }
        }

        if total > 0.0 {
            steering *= 1.0 / total;
            let mag = steering.magnitude();
            steering *= MAX_VELOCITY / mag;
            steering -= self.velocity;

            let mag = steering.magnitude();
            if mag > MAX_FORCE {
                steering *= MAX_FORCE / mag;
            }
        }

        steering
    }

    fn separation(&self, boids: &[Boid]) -> F32x2 {
        let perception_radius = 24.0;
        let mut steering = F32x2::default();
        let mut total = 0.0;

        for boid in boids.iter() {
            let d = self.position.distance(boid.position);

            if boid != self && d < perception_radius {
                let mut diff = self.position - boid.position;
                diff *= 1.0 / (d * d);
                steering += diff;
                total += 1.0;
            }
        }

        if total > 0.0 {
            steering *= 1.0 / total;
            let mag = steering.magnitude();
            steering *= MAX_VELOCITY / mag;
            steering -= self.velocity;

            let mag = steering.magnitude();
            if mag > MAX_FORCE {
                steering *= MAX_FORCE / mag;
            }
        }

        steering
    }

    fn cohesion(&self, boids: &[Boid]) -> F32x2 {
        let perception_radius = 50.0;
        let mut steering = F32x2::default();
        let mut total = 0.0;

        for boid in boids.iter() {
            let d = self.position.distance(boid.position);

            if boid != self && d < perception_radius {
                steering += boid.position;
                total += 1.0;
            }
        }

        if total > 0.0 {
            steering *= 1.0 / total;
            steering -= self.position;
            let mag = steering.magnitude();
            steering *= MAX_VELOCITY / mag;
            steering -= self.velocity;

            let mag = steering.magnitude();
            if mag > MAX_FORCE {
                steering *= MAX_FORCE / mag;
            }
        }

        steering
    }

    fn flock(&mut self, boids: &[Boid]) {
        let alignment = self.alignment(boids) * 1.5;
        let cohesion = self.cohesion(boids) * 1.0;
        let separation = self.separation(boids) * 2.0;

        self.acceleration += alignment;
        self.acceleration += cohesion;
        self.acceleration += separation;
    }

    fn update(&mut self, width: usize, height: usize) {
        self.position += self.velocity;
        self.velocity += self.acceleration;

        let mag = self.velocity.magnitude();
        if mag > MAX_VELOCITY {
            self.velocity *= MAX_VELOCITY / mag;
        }

        self.acceleration *= 0.0;

        self.edges(width, height);
    }

    fn draw(&mut self, gop: &mut GraphicsOutput) -> Result {
        let color = BltPixel::new(255, 0, 0);

        gop.blt(BltOp::VideoFill {
            color,
            dest: (self.position.x as usize, self.position.y as usize),
            dims: (self.body.width as usize, self.body.height as usize),
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
    let mut buf = [0; mem::size_of::<usize>()];
    rng.get_rng(None, &mut buf).expect("get_rng failed");
    let number = usize::from_le_bytes(buf);
    (number % (end - start + 1)) + start
}
