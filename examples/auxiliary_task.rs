//! Produces a sine wave while printing "this is a string" repeatedly,
//! appending "LOL" to every iteration.
//! 
extern crate bela;
extern crate sample;

use std::{thread, time};
use bela::*;
use sample::{Signal, Sample};

struct PrintTask<F> {
    callback: F,
    args: String,
}

impl<F> Auxiliary for PrintTask<F>
where F: FnMut(&mut String),
      for<'r> F: FnMut(&'r mut String)
{
    type Callback = F;
    type Args = String;

    fn destructure(&mut self) -> (&mut F, &mut Self::Args) {
        let PrintTask {
            callback,
            args,
        } = self;

        (callback, args)
    }
}

struct AppData<'a> {
    render: &'a Fn(&mut Context, &mut AppData<'a>),
    setup: Option<&'a Fn(&mut Context, &mut AppData<'a>) -> Result<(), error::Error>>,
    cleanup: Option<&'a Fn(&mut Context, &mut AppData<'a>)>,
    synth: Option<Box<Signal<Frame=[f64; 1]>>>,
    frame_index: usize,
    tasks: Vec<CreatedTask>,
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

type BelaApp<'a> = Bela<AppData<'a>>;

fn main() {
    go().unwrap();
}

fn go() -> Result<(), error::Error> {
    let what_to_print = "this is a string".to_string();
    let mut print_task = PrintTask {
        callback: |args: &mut String| {
            args.push_str("LOL");
            println!("{}", args);
        },
        args: what_to_print,
    };

    let setup = |_context: &mut Context, user_data: &mut AppData| -> Result<(), error::Error> {
        println!("Setting up");
        user_data.tasks.push(BelaApp::create_auxiliary_task(&print_task, 10, "printing_stuff"));
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

        if user_data.frame_index % 1024 == 0 {
            for task in user_data.tasks.iter() {
                BelaApp::schedule_auxiliary_task(task);
            }
        }

        user_data.frame_index = user_data.frame_index.wrapping_add(1);
    };

    let sig = sample::signal::rate(44_100.0)
        .const_hz(440.0)
        .sine();

    let user_data = AppData {
        render: &render,
        setup: Some(&setup),
        cleanup: Some(&cleanup),
        synth: Some(Box::new(sig)),
        frame_index: 0,
        tasks: Vec::new(),
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
