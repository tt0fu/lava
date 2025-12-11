mod audio;
mod video;

use audio::{Analyzer, Stream};
use std::error::Error;
use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};
use winit::event_loop::EventLoop;

// fn get_braile(vecs: Vec<u8>) -> char {
//     let to_add: u32 = vecs.iter().map(|a| 1 << a).sum();
//     char::from_u32(('⠀' as u32) + to_add).unwrap()
// }

fn get_pixel(a: f32, b: f32) -> char {
    let map = [
        ['⠀', '⢀', '⢠', '⢰', '⢸'],
        ['⡀', '⣀', '⣠', '⣰', '⣸'],
        ['⡄', '⣄', '⣤', '⣴', '⣼'],
        ['⡆', '⣆', '⣦', '⣶', '⣾'],
        ['⡇', '⣇', '⣧', '⣷', '⣿'],
    ];
    map[(a.clamp(0.0, 1.0) * 4.0).round() as usize][(b.clamp(0.0, 1.0) * 4.0).round() as usize]
}

fn demo() {
    const STREAM_SIZE: usize = 4096;
    const SAMPLE_RATE: u32 = 48000;
    const BUFFER_SIZE: usize = 4096;
    const BIN_COUNT: usize = 512;
    const SCREEN_WIDTH: usize = 190;
    const SCREEN_HEIGHT: usize = 50;
    let mut stream = Stream::<STREAM_SIZE, 2>::new(SAMPLE_RATE, 4096);
    let mut ft = Analyzer::<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>::new();
    let mut gain = 1.0;

    loop {
        let samples = stream.get_samples();
        if samples.is_empty() {
            continue;
        }
        gain += 0.01;
        let mono = samples
            .iter()
            .map(|s| (s[0] + s[1]) / 2.0)
            .collect::<Vec<f32>>();
        for sample in &mono {
            let volume = (sample * gain).abs();
            if volume > 0.9 {
                gain /= volume / 0.9;
            }
            if (sample * gain).abs() > 1.0 {
                gain /= sample.abs();
            }
            ft.push(&(sample * gain));
        }

        let bins = ft.get_ft();
        let mut heights = [0.0; SCREEN_WIDTH * 2];
        let mut counts = [0; SCREEN_WIDTH * 2];
        for i in 0..BIN_COUNT {
            let bin = bins[i].length() * 1.5;
            let ind = i * SCREEN_WIDTH * 2 / BIN_COUNT;
            heights[ind] += bin.clamp(0.0, 1.0);
            counts[ind] += 1;
        }
        for line in (0..SCREEN_HEIGHT).rev() {
            for row in 0..SCREEN_WIDTH {
                let left_height = heights[row * 2] / (counts[row * 2] as f32);
                let right_height = heights[row * 2 + 1] / (counts[row * 2 + 1] as f32);
                let a = (left_height * (SCREEN_HEIGHT as f32) - (line as f32)).clamp(0.0, 1.0);
                let b = (right_height * (SCREEN_HEIGHT as f32) - (line as f32)).clamp(0.0, 1.0);
                print!("{}", get_pixel(a, b));
            }
            println!();
        }
    }
}

fn main() -> Result<(), impl Error> {
    let event_loop = EventLoop::new().unwrap();
    let mut app = video::App::new(&event_loop);

    event_loop.run_app(&mut app)
}
