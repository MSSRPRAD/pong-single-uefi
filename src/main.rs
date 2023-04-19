#![no_main]
#![no_std]

extern crate alloc;

use std::io::BufReader;

use alloc::vec;
use alloc::vec::Vec;
use log::info;
use uefi::prelude::*;
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::Result;

#[derive(Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

struct Buffer {
    width: usize,
    height: usize,
    pixels: Vec<BltPixel>,
}

impl Buffer {
    /// Create a new `Buffer`.
    fn new(width: usize, height: usize) -> Self {
        Buffer {
            width,
            height,
            pixels: vec![BltPixel::new(0, 0, 0); width * height],
        }
    }

    /// Get a single pixel.
    fn pixel(&mut self, x: usize, y: usize) -> Option<&mut BltPixel> {
        self.pixels.get_mut(y * self.width + x)
    }

    /// Blit the buffer to the framebuffer.
    fn blit(&self, gop: &mut GraphicsOutput) -> Result {
        gop.blt(BltOp::BufferToVideo {
            buffer: &self.pixels,
            src: BltRegion::Full,
            dest: (0, 0),
            dims: (self.width, self.height),
        })
    }
}

#[entry]
fn main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    info!("Hello world!");
    let bt = system_table.boot_services();
    // Open graphics output protocol.
    let gop_handle = bt.get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop = bt.open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
    let (width, height) = gop.current_mode_info().resolution();
    let mut buffer = Buffer::new(width, height);
    // Initialize the buffer with a simple gradient background.
    for y in 0..height {
        let r = ((y as f32) / ((height - 1) as f32)) * 255.0;
        for x in 0..width {
            let g = ((x as f32) / ((width - 1) as f32)) * 255.0;
            let pixel = buffer.pixel(x, y).unwrap();
            pixel.red = r as u8;
            pixel.green = g as u8;
            pixel.blue = 255;
        }
    }

    
    loop {
        // Draw the buffer to the screen.
        buffer.blit(&mut gop);
    }
    system_table.boot_services().stall(10_000_000);
    Status::SUCCESS
}
