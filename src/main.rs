//mod audio;
//use audio::Output;

mod app;
mod input;
mod widgets;

use app::{App, AppState};
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();

    let mut state = AppState::new();
    let result = App::run_loop(terminal, &mut state);

    ratatui::restore();
    return result;
}


// fn main() {
//     let out: Output = Output::initialise().expect("failed to create output");
//     // Produce a sinusoid of maximum amplitude.
//     let sample_rate = 44100 as f32;
//     let mut sample_clock: f32 = 0.0;
//     let next_value = move || {
//         sample_clock = (sample_clock + 1.0) % sample_rate;
//         return (sample_clock * 220.0 * 2.0 * std::f32::consts::PI / sample_rate).sin() as f64;
//     };
//     out.play(Box::new(next_value));
// }
