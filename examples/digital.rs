extern crate bela;

use bela::*;

struct State {
    idx: usize,
}

fn main() {
    go().unwrap();
}

fn go() -> Result<(), error::Error> {
    let mut setup = |context: &mut Context, _user_data: &mut State| -> Result<(), error::Error> {
        println!("Setting up");
        context.pin_mode(0, 0, DigitalDirection::OUTPUT);
        Ok(())
    };

    let mut cleanup = |_context: &mut Context, _user_data: &mut State| {
        println!("Cleaning up");
    };

    // Generates impulses on the first digital port every 100ms for 10ms
    let mut render = |context: &mut Context, state: &mut State| {
        let tenms_in_frames = (context.digital_sample_rate() / 100.) as usize;
        let hundreadms_in_frames = (tenms_in_frames * 10) as usize;
        for f in 0..context.digital_frames() {
            let v = if state.idx < tenms_in_frames { 1 } else { 0 };
            context.digital_write_once(f, 0, v);
            state.idx += 1;
            if state.idx > hundreadms_in_frames {
                state.idx = 0;
            }
        }
    };

    let state = State {
        idx: 0,
    };

    let user_data = AppData::new(state, &mut render, Some(&mut setup), Some(&mut cleanup));

    let mut bela_app = Bela::new(user_data);
    let mut settings = InitSettings::default();
    bela_app.run(&mut settings)
}
