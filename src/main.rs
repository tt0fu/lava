use anyhow::Result;
use lava::{app::App, config::Config};
use std::path::Path;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<String>>();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "{}",
            "Usage: lava [<path/to/config.jsonc>] [--print-config | -p] [--help | -h]"
        );
        return Ok(());
    }

    let (config, config_path) = match args.iter().skip(1).find(|a| !a.starts_with("-")) {
        Some(path_arg) => {
            let path = Path::new(path_arg);
            (Config::from_jsonc(path)?, path)
        }
        None => (Config::default(), Path::new("")),
    };

    if args.iter().any(|a| a == "--print-config" || a == "-p") {
        println!("{}", config.to_jsonc()?);
        return Ok(());
    }

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(&event_loop, &config, &config_path.to_path_buf())?;

    event_loop.run_app(&mut app)?;

    Ok(())
}
