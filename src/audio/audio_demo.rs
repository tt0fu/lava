mod circular_buffer;
mod sample_buffer;

use self::circular_buffer::CircularBuffer;
use self::sample_buffer::SampleBuffer;

fn main() -> Result<(), anyhow::Error> {
    const STREAM_SIZE: usize = 4096;
    let mut sample_buffer = Stream::<STREAM_SIZE>::new();
    const BUFFER_SIZE: usize = 2048;
    let mut buffer = CircularBuffer::<f32, BUFFER_SIZE>::new(0.0);
    const SCREEN_WIDTH: usize = 150;
    const SCREEN_HEIGHT: usize = 50;

    loop {
        let samples = sample_buffer.get_samples();
        if samples.is_empty() {
            continue;
        }
        for sample in samples {
            buffer.push(sample);
        }
        let mut screen = [[' '; SCREEN_WIDTH]; SCREEN_HEIGHT];
        for i in 0..BUFFER_SIZE {
            let sample = buffer[i];
            let height = (((sample.clamp(-1.0, 1.0) / 2.0 + 0.5) * (SCREEN_HEIGHT as f32)).round()
                as usize)
                .min(SCREEN_HEIGHT - 1);
            screen[height][i * SCREEN_WIDTH / BUFFER_SIZE] = '#'
        }
        for line in screen {
            println!("{}", line.iter().collect::<String>())
        }
    }
}
