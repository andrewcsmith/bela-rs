extern crate bela;

use std::{thread, time, slice};
use bela::*;

struct AppData<'a> {
    render: &'a Fn(&mut Context, &mut AppData<'a>),
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
    let render = |context: &mut Context, _user_data: &mut AppData| {
        unsafe {
            let context = context.context_ptr();
            let n_frames = (*context).audioFrames;
            let n_channels = (*context).audioOutChannels;

            let audio_out: &mut [f32] = slice::from_raw_parts_mut((*context).audioOut as *mut f32, (n_frames * n_channels) as usize);

            let len = audio_out.len();
            for (idx, samp) in audio_out.iter_mut().enumerate() {
                *samp = idx as f32 / len as f32;
            }
        }
    };

    let user_data = AppData {
        render: &render
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
