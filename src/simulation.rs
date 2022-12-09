use crate::{boid::Boid, buffer::Buffer, get_random_usize_range};
use alloc::vec::Vec;
use micromath::vector::F32x2;
use uefi::{
    proto::{console::gop::GraphicsOutput, rng::Rng},
    Result,
};

#[derive(Debug)]
pub struct Simulation {
    boids: Vec<Boid>,
    width: usize,
    height: usize,
    buffer: Buffer,
}

impl Simulation {
    pub fn new(rng: &mut Rng, width: usize, height: usize) -> Self {
        let mut boids = Vec::new();

        for _ in 0..200 {
            let position = F32x2 {
                x: get_random_usize_range(rng, 0, width) as f32,
                y: get_random_usize_range(rng, 0, height) as f32,
            };
            let velocity = F32x2 {
                x: get_random_usize_range(rng, 2, 4) as f32,
                y: get_random_usize_range(rng, 2, 4) as f32,
            };

            let boid = Boid::new(position, velocity);
            boids.push(boid);
        }

        Self {
            boids,
            width,
            height,
            buffer: Buffer::new(width, height),
        }
    }

    pub fn run(&mut self, gop: &mut GraphicsOutput) -> Result {
        loop {
            let flock = self.boids.clone();
            self.buffer.clear();

            for boid in self.boids.iter_mut() {
                boid.flock(&flock);
                boid.update(self.width, self.height);
                boid.draw(&mut self.buffer)?;
            }

            self.buffer.blit(gop)?;
        }
    }
}
