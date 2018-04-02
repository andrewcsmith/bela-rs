extern crate bela;

use std::{thread, time};
use bela::*;

struct AppData<'a> {
    render: &'a Fn(&mut Context, &mut AppData<'a>),
    frame_index: usize,
}

impl<'a> UserData<'a> for AppData<'a> {
    fn render_fn(&self) -> &'a Fn(&mut Context, &mut AppData<'a>) {
        self.render
    }

    fn set_render_fn(&mut self, callback: &'a (Fn(&mut Context, &mut AppData<'a>) + 'a)) {
        self.render = callback;
    }
}

fn main() {
    go().unwrap();
}

fn go() -> Result<(), error::Error> {
    // Generates a sawtooth wave with the period of whatever the audio frame
    // size is.
    let render = |context: &mut Context, user_data: &mut AppData| {
        let AppData {
            frame_index,
            ..
        } = user_data;

        let len = context.audio_out().len();
        for (idx, samp) in context.audio_out().iter_mut().enumerate() {
            *samp = (idx as f32 / len as f32) * (*frame_index % 10) as f32;
        }

        // We want to keep track of the frame index here
        *frame_index = frame_index.wrapping_add(1);
    };

    let user_data = AppData {
        render: &render,
        frame_index: 0,
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
