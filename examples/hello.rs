extern crate bela;

use bela::*;

struct Phasor {
    idx: usize,
}

fn main() {
    go().unwrap();
}

fn go() -> Result<(), error::Error> {
    let mut setup = |_context: &mut Context, _user_data: &mut Phasor| -> Result<(), error::Error> {
        println!("Setting up");
        Ok(())
    };

    let mut cleanup = |_context: &mut Context, _user_data: &mut Phasor| {
        println!("Cleaning up");
    };

    // Generates a non-bandlimited sawtooth at 110Hz.
    let mut render = |context: &mut Context, phasor: &mut Phasor| {
        for (_, samp) in context.audio_out().iter_mut().enumerate() {
            let gain = 0.5;
            *samp = 2. * (phasor.idx as f32 * 110. / 44100.) - 1.;
            *samp *= gain;
            phasor.idx += 1;
            if phasor.idx as f32 > 44100. / 110. {
                phasor.idx = 0;
            }
        }
    };

    let phasor = Phasor { idx: 0 };

    let user_data = AppData::new(phasor, &mut render, Some(&mut setup), Some(&mut cleanup));

    let mut bela_app = Bela::new(user_data);
    let mut settings = InitSettings::default();
    bela_app.run(&mut settings)
}
