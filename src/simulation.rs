use crate::{boid::Boid, get_random_usize_range};
use alloc::vec::Vec;
use micromath::vector::F32x2;
use uefi::{
    prelude::BootServices,
    proto::{
        console::gop::{BltOp, BltPixel, GraphicsOutput},
        rng::Rng,
    },
    Result,
};

#[derive(Debug)]
pub struct Simulation {
    boids: Vec<Boid>,
    width: usize,
    height: usize,
}

impl Simulation {
    pub fn new(rng: &mut Rng, width: usize, height: usize) -> Self {
        let mut boids = Vec::new();

        for _ in 0..100 {
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
        }
    }

    pub fn run(&mut self, bt: &BootServices, gop: &mut GraphicsOutput) -> Result {
        loop {
            let flock = self.boids.clone();

            self.clear_screen(gop)?;

            for boid in self.boids.iter_mut() {
                boid.flock(&flock);
                boid.update(self.width, self.height);
                boid.draw(gop)?;
            }

            bt.stall(16_667);
        }
    }

    fn clear_screen(&self, gop: &mut GraphicsOutput) -> Result {
        let color = BltPixel::new(0, 0, 0);

        gop.blt(BltOp::VideoFill {
            color,
            dest: (0, 0),
            dims: (self.width, self.height),
        })
    }
}
