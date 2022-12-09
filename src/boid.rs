use micromath::vector::{F32x2, Vector};
use uefi::proto::console::gop::{BltOp, BltPixel, GraphicsOutput};
use uefi::Result;

#[derive(PartialEq, Clone, Debug)]
struct Rectangle {
    width: f32,
    height: f32,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Boid {
    body: Rectangle,
    position: F32x2,
    velocity: F32x2,
    acceleration: F32x2,
}

const MAX_FORCE: f32 = 0.2;
const MAX_VELOCITY: f32 = 5.0;

impl Boid {
    pub fn new(position: F32x2, velocity: F32x2) -> Self {
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

    pub fn flock(&mut self, boids: &[Boid]) {
        let alignment = self.alignment(boids) * 1.5;
        let cohesion = self.cohesion(boids) * 1.0;
        let separation = self.separation(boids) * 2.0;

        self.acceleration += alignment;
        self.acceleration += cohesion;
        self.acceleration += separation;
    }

    pub fn update(&mut self, width: usize, height: usize) {
        self.position += self.velocity;
        self.velocity += self.acceleration;

        let mag = self.velocity.magnitude();
        if mag > MAX_VELOCITY {
            self.velocity *= MAX_VELOCITY / mag;
        }

        self.acceleration *= 0.0;

        self.edges(width, height);
    }

    pub fn draw(&mut self, gop: &mut GraphicsOutput) -> Result {
        let color = BltPixel::new(255, 0, 0);

        gop.blt(BltOp::VideoFill {
            color,
            dest: (self.position.x as usize, self.position.y as usize),
            dims: (self.body.width as usize, self.body.height as usize),
        })?;

        Ok(())
    }
}
