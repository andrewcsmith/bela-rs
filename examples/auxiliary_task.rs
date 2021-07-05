//! Produces a sine wave while printing "this is a string" repeatedly,
//! appending "LOL" to every iteration.
//!
//! There's an example here for both the stack-allocated and a Boxed closure.
//!
extern crate bela;
extern crate sample;

use bela::*;

struct MyData {
    frame_index: usize,
    tasks: Vec<CreatedTask>,
}

type BelaApp<'a> = Bela<AppData<'a, MyData>>;

fn main() {
    go().unwrap();
}

fn go() -> Result<(), error::Error> {
    let mut setup = |_context: &mut Context, user_data: &mut MyData| -> Result<(), error::Error> {
        println!("Setting up");
        let print_task = Box::new(|| {
            println!("this is a string");
        });

        let another_print_task = Box::new(|| {
            println!("this is another string");
        });

        user_data.tasks.push(BelaApp::create_auxiliary_task(
            print_task,
            10,
            &std::ffi::CString::new("printing_stuff").unwrap(),
        ));
        user_data.tasks.push(BelaApp::create_auxiliary_task(
            another_print_task,
            10,
            &std::ffi::CStr::from_bytes_with_nul(b"printing_more_stuff\0").unwrap(),
        ));
        Ok(())
    };

    let mut cleanup = |_context: &mut Context, _user_data: &mut MyData| {
        println!("Cleaning up");
    };

    let mut render = |_context: &mut Context, user_data: &mut MyData| {
        if user_data.frame_index % 1024 == 0 {
            for task in user_data.tasks.iter() {
                BelaApp::schedule_auxiliary_task(task).unwrap();
            }
        }

        user_data.frame_index = user_data.frame_index.wrapping_add(1);
    };

    let my_data = MyData {
        tasks: Vec::new(),
        frame_index: 0,
    };

    let user_data = AppData::new(my_data, &mut render, Some(&mut setup), Some(&mut cleanup));

    let mut settings = InitSettings::default();
    Bela::new(user_data).run(&mut settings)
}
