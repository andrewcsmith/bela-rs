extern crate nix;
extern crate libc;
extern crate bela_sys;

use bela_sys::{BelaInitSettings, BelaContext};
use std::{mem, slice};

pub mod error;

/// The `Bela` struct is essentially built to ensure that the type parameter
/// `<T>` is consistent across all invocations of the setup, render, and cleanup
/// functions. This is because `<T>` is the `UserData` of the original Bela
/// library -- we want to ensure that the `UserData` we are initializing with
/// is the exact same as the one we are attempting to access with each function.
/// 
/// TODO: Bela needs to also wrap the various setup, render, and cleanup
/// functions and keep them in the same struct.
/// 
/// Called when audio is initialized.
/// 
/// ```rust
/// pub type SetupFn = FnOnce(&mut Context, T) -> bool;
/// ```
/// 
/// Called on every frame.
/// 
/// ```rust
/// pub type RenderFn = Fn(&mut Context, T);
/// ```
/// 
/// Called when audio is stopped.
/// 
/// ```rust
/// pub type CleanupFn = FnOnce(&mut Context, T) -> bool;
/// ```
pub struct Bela<T> {
    initialized: bool,
    user_data: T,
}

unsafe extern "C" fn render_trampoline<'a, T>(context: *mut BelaContext, user_data: *mut std::os::raw::c_void) 
where T: UserData<'a> + 'a
{
    let mut context = Context::new(context);
    let user_data: &mut T = mem::transmute(user_data);
    user_data.render_fn()(&mut context, user_data);
}

unsafe extern "C" fn setup_trampoline<'a, T>(context: *mut BelaContext, user_data: *mut std::os::raw::c_void) -> bool
where T: UserData<'a> + 'a
{
    let mut context = Context::new(context);
    let user_data: &mut T = mem::transmute(user_data);
    match user_data.setup_fn() {
        Some(func) => {
            match func(&mut context, user_data) {
                Ok(_) => true,
                Err(_) => false,
            }
        }
        None => {
            // Default to "success" if there's no function
            true
        }
    }
}

unsafe extern "C" fn cleanup_trampoline<'a, T>(context: *mut BelaContext, user_data: *mut std::os::raw::c_void)
where T: UserData<'a> + 'a
{
    let mut context = Context::new(context);
    let user_data: &mut T = mem::transmute(user_data);
    match user_data.cleanup_fn() {
        Some(func) => { func(&mut context, user_data); }, 
        None => { }
    }
}

impl<'a, T: UserData<'a> + 'a> Bela<T> {
    pub fn new(user_data: T) -> Self {
        Bela {
            initialized: false,
            user_data,
        }
    }

    pub fn set_render<F: 'a>(&mut self, func: &'a F) 
    where F: Fn(&mut Context, T),
          for<'r, 's> F: Fn(&'r mut Context, &'s mut T)
    {
        self.user_data.set_render_fn(func);
    }

    pub fn set_setup<F: 'a>(&mut self, func: &'a F) 
    where F: Fn(&mut Context, T) -> bool,
          for<'r, 's> F: Fn(&'r mut Context, &'s mut T) -> Result<(), error::Error>
    {
        self.user_data.set_setup_fn(Some(func));
    }

    pub fn set_cleanup<F: 'a>(&mut self, func: &'a F) 
    where F: Fn(&mut Context, T),
          for<'r, 's> F: Fn(&'r mut Context, &'s mut T)
    {
        self.user_data.set_cleanup_fn(Some(func));
    }

    pub fn init_audio(&mut self, settings: &mut InitSettings) -> Result<(), error::Error> {
        settings.settings.setup = Some(setup_trampoline::<T>);
        settings.settings.render = Some(render_trampoline::<T>);
        settings.settings.cleanup = Some(cleanup_trampoline::<T>);
        let out = unsafe {
            let ptr: *mut std::os::raw::c_void = mem::transmute(&mut self.user_data);
            bela_sys::Bela_initAudio(settings.settings_ptr(), ptr)
        };

        match out {
            0 => { 
                self.initialized = true;
                Ok(())
            },
            _ => Err(error::Error::Init),
        }
    }

    pub fn start_audio(&self) -> Result<(), error::Error> {
        if !self.initialized { 
            return Err(error::Error::Start); 
        }

        let out = unsafe {
            bela_sys::Bela_startAudio()
        };

        match out {
            0 => Ok(()),
            _ => Err(error::Error::Start),
        }
    }

    pub fn should_stop(&self) -> bool {
        unsafe {
            bela_sys::gShouldStop != 0
        }
    }

    pub fn stop_audio(&self) {
        unsafe { bela_sys::Bela_stopAudio(); }
    }

    pub fn cleanup_audio(&self) {
        unsafe { bela_sys::Bela_cleanupAudio(); }
    }
}

/// Wraps `BelaContext`
pub struct Context {
    context: *mut BelaContext,
}

impl Context {
    pub fn new(context: *mut BelaContext) -> Context {
        Context {
            context
        }
    }

    pub fn context_ptr(&mut self) -> *mut BelaContext {
        let ptr: *mut BelaContext = self.context;
        ptr
    }

    /// Access the audio output slice
    pub fn audio_out(&mut self) -> &mut [f32] {
        unsafe {
            let context = self.context_ptr();
            let n_frames = (*context).audioFrames;
            let n_channels = (*context).audioOutChannels;
            let audio_out_ptr = (*context).audioOut as *mut f32;
            slice::from_raw_parts_mut(audio_out_ptr, (n_frames * n_channels) as usize)
        }
    }

    pub fn audio_frames(&self) -> usize {
        unsafe {
            (*self.context).audioFrames as usize
        }
    }

    pub fn audio_out_channels(&self) -> usize {
        unsafe {
            (*self.context).audioOutChannels as usize
        }
    }
}

pub trait UserData<'a> {
    fn render_fn(&self) -> &'a Fn(&mut Context, &mut Self);
    fn set_render_fn(&mut self, &'a Fn(&mut Context, &mut Self));
    fn setup_fn(&self) -> Option<&'a Fn(&mut Context, &mut Self) -> Result<(), error::Error>>;
    fn set_setup_fn(&mut self, Option<&'a Fn(&mut Context, &mut Self) -> Result<(), error::Error>>);
    fn cleanup_fn(&self) -> Option<&'a Fn(&mut Context, &mut Self)>;
    fn set_cleanup_fn(&mut self, Option<&'a Fn(&mut Context, &mut Self)>);
}

/// Safe wrapper for `BelaInitSettings`, which sets initial parameters for the
/// Bela system.
pub struct InitSettings {
    settings: BelaInitSettings,
}

impl InitSettings {
    pub fn settings_ptr(&mut self) -> *mut BelaInitSettings {
        &mut self.settings
    }

    pub fn verbose(&self) -> bool {
        match self.settings.verbose {
            0 => false,
            _ => true
        }
    }

    pub fn set_verbose(&mut self, val: bool) {
        self.settings.verbose = match val {
            true => 1,
            false => 0
        };
    }

    pub fn high_performance_mode(&self) -> bool {
        match self.settings.highPerformanceMode {
            0 => false,
            _ => true
        }
    }

    pub fn set_high_performance_mode(&mut self, val: bool) {
        self.settings.highPerformanceMode = match val {
            true => 1,
            false => 0
        };
    }

    pub fn analog_outputs_persist(&self) -> bool {
        match self.settings.analogOutputsPersist {
            0 => false,
            _ => true
        }
    }

    pub fn set_analog_outputs_persist(&mut self, val: bool) {
        self.settings.analogOutputsPersist = match val {
            true => 1,
            false => 0
        };
    }
}

impl Default for InitSettings {
    fn default() -> InitSettings {
        let settings = unsafe {
            let mut settings: BelaInitSettings = mem::uninitialized();
            bela_sys::Bela_defaultSettings(&mut settings);
            settings
        };

        InitSettings {
            settings
        }
    }
}
