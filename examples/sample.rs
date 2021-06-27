extern crate bela;
extern crate sample;

use bela::*;
use sample::Signal;
use std::{thread, time};

fn main() {
    go().unwrap();
}

fn go() -> Result<(), error::Error> {
    let mut setup = |_context: &mut Context,
                     _user_data: &mut Option<Box<dyn Signal<Frame = f64>>>|
     -> Result<(), error::Error> {
        println!("Setting up");
        Ok(())
    };

    let mut cleanup =
        |_context: &mut Context, _user_data: &mut Option<Box<dyn Signal<Frame = f64>>>| {
            println!("Cleaning up");
        };

    // Generates a sine wave with the period of whatever the audio frame
    // size is.
    let mut render = |context: &mut Context, synth: &mut Option<Box<dyn Signal<Frame = f64>>>| {
        let audio_out_channels = context.audio_out_channels();
        assert_eq!(audio_out_channels, 2);
        let audio_out = context.audio_out();
        let audio_out_frames: &mut [[f32; 2]] =
            sample::slice::to_frame_slice_mut(audio_out).unwrap();

        for frame in audio_out_frames.iter_mut() {
            for samp in frame.iter_mut() {
                let val = synth.as_mut().unwrap().next();
                *samp = val as f32;
            }
        }
    };

    let sig = sample::signal::rate(44_100.0).const_hz(440.0).sine();

    let synth: Option<Box<dyn Signal<Frame = f64>>> = Some(Box::new(sig));

    let user_data = AppData::new(synth, &mut render, Some(&mut setup), Some(&mut cleanup));

    let mut bela_app = Bela::new(user_data);
    let mut settings = InitSettings::default();
    bela_app.init_audio(&mut settings)?;
    bela_app.start_audio()?;

    while !bela_app.should_stop() {
        thread::sleep(time::Duration::new(1, 0));
    }

    bela_app.stop_audio();
    bela_app.cleanup_audio();

    Ok(())
}
