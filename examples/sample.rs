extern crate bela;
extern crate sample;

use std::{thread, time};
use bela::*;
use sample::{Signal, Sample};

struct AppData<'a> {
    render: &'a Fn(&mut Context, &mut AppData<'a>),
    setup: Option<&'a Fn(&mut Context, &mut AppData<'a>) -> Result<(), error::Error>>,
    cleanup: Option<&'a Fn(&mut Context, &mut AppData<'a>)>,
    synth: Option<Box<Signal<Frame=[f64; 1]>>>,
}

impl<'a> UserData<'a> for AppData<'a> {
    fn render_fn(&self) -> &'a Fn(&mut Context, &mut AppData<'a>) {
        self.render
    }

    fn set_render_fn(&mut self, callback: &'a (Fn(&mut Context, &mut AppData<'a>) + 'a)) {
        self.render = callback;
    }

    fn setup_fn(&self) -> Option<&'a Fn(&mut Context, &mut AppData<'a>) -> Result<(), error::Error>> {
        self.setup
    }

    fn set_setup_fn(&mut self, callback: Option<&'a (Fn(&mut Context, &mut AppData<'a>) -> Result<(), error::Error> + 'a)>) {
        self.setup = callback;
    }

    fn cleanup_fn(&self) -> Option<&'a Fn(&mut Context, &mut AppData<'a>)> {
        self.cleanup
    }

    fn set_cleanup_fn(&mut self, callback: Option<&'a (Fn(&mut Context, &mut AppData<'a>) + 'a)>) {
        self.cleanup = callback;
    }
}

fn main() {
    go().unwrap();
}

fn go() -> Result<(), error::Error> {
    let setup = |_context: &mut Context, user_data: &mut AppData| -> Result<(), error::Error> {
        println!("Setting up");
        Ok(())
    };

    let cleanup = |_context: &mut Context, _user_data: &mut AppData| {
        println!("Cleaning up");
    };

    // Generates a sawtooth wave with the period of whatever the audio frame
    // size is.
    let render = |context: &mut Context, user_data: &mut AppData| {
        let AppData {
            ref mut synth,
            ..
        } = *user_data;

        let audio_frames = context.audio_frames();
        let audio_out_channels = context.audio_out_channels();
        let audio_out = context.audio_out();
        assert_eq!(audio_out_channels, 2);
        let audio_out_frames: &mut [[f32; 2]] = sample::slice::to_frame_slice_mut(audio_out).unwrap();

        for frame in audio_out_frames.iter_mut() {
            let val = synth.as_mut().unwrap().next();
            for samp in frame.iter_mut() {
                *samp = val[0] as f32;
            }
        }
    };

    let sig = sample::signal::rate(44_100.0)
        .const_hz(440.0)
        .sine();

    let user_data = AppData {
        render: &render,
        setup: Some(&setup),
        cleanup: Some(&cleanup),
        synth: Some(Box::new(sig)),
    };

    let mut bela_app: Bela<AppData> = Bela::new(user_data);
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
