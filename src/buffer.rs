use alloc::vec;
use alloc::vec::Vec;
use uefi::{
    proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput},
    Result,
};

#[derive(Debug)]
pub struct Buffer {
    width: usize,
    height: usize,
    pixels: Vec<Vec<BltPixel>>,
    current_buffer: usize,
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Self {
        Buffer {
            width,
            height,
            pixels: vec![
                vec![BltPixel::new(0, 0, 0); width * height],
                vec![BltPixel::new(0, 0, 0); width * height],
            ],
            current_buffer: 0,
        }
    }

    pub fn pixel(&mut self, x: usize, y: usize) -> Option<&mut BltPixel> {
        self.pixels[self.current_buffer].get_mut(y * self.width + x)
    }

    pub fn blit(&mut self, gop: &mut GraphicsOutput) -> Result {
        gop.blt(BltOp::BufferToVideo {
            buffer: &self.pixels[self.current_buffer],
            src: BltRegion::Full,
            dest: (0, 0),
            dims: (self.width, self.height),
        })
    }

    pub fn clear(&mut self) {
        if self.current_buffer == 0 {
            self.current_buffer = 1;
        } else {
            self.current_buffer = 0;
        }

        self.pixels[self.current_buffer] = vec![BltPixel::new(0, 0, 0); self.width * self.height];
    }
}
