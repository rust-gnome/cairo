// Copyright 2018-2019, The Gtk-rs Project Developers.
// See the COPYRIGHT file at the top-level directory of this distribution.
// Licensed under the MIT license, see the LICENSE file or <http://opensource.org/licenses/MIT>

use std::io;
use std::slice;
use std::fmt;

use ffi::{self, cairo_status_t};
use libc::{c_void, c_uchar, c_uint, c_double};
use ::enums::Status;
use surface::{Surface, SurfaceExt};


pub type Constructor = unsafe extern fn (ffi::cairo_write_func_t, *mut c_void, c_double, c_double) -> *mut ffi::cairo_surface_t;

pub trait FromRawSurface {
    unsafe fn from_raw_surface(surface: *mut ffi::cairo_surface_t) -> Self;
}

#[derive(Debug)]
struct CallbackEnv<W> {
    writer: W,
    error: Option<io::Error>,
}

#[derive(Debug)]
pub struct Writer<S: FromRawSurface + AsRef<Surface>, W: io::Write> {
    pub surface: S,
    callback_env: Box<CallbackEnv<W>>,
}

impl<S: FromRawSurface + AsRef<Surface>, W: io::Write> Writer<S, W> {
    extern fn write_cb(env: *mut c_void, data: *mut c_uchar, length: c_uint) -> cairo_status_t {
        let env: &mut CallbackEnv<W> = unsafe { &mut *(env as *mut CallbackEnv<W>) };
        let data = unsafe { slice::from_raw_parts(data, length as usize) };

        let result = match env.writer.write_all(data) {
            Ok(_) => Status::Success,
            Err(e) => {
                env.error = Some(e);
                Status::WriteError
            }
        };

        result.into()
    }

    pub fn new(constructor: Constructor, width: f64, height: f64, writer: W) -> Writer<S, W> {
        let mut callback_env = Box::new(CallbackEnv {
            writer,
            error: None,
        });
        let env_ptr = &mut *callback_env as *mut CallbackEnv<W> as *mut c_void;
        let surface = unsafe {
            S::from_raw_surface(constructor(Some(Self::write_cb), env_ptr, width, height))
        };

        Writer {
            surface,
            callback_env,
        }
    }

    pub fn writer(&self) -> &W { &self.callback_env.writer }
    pub fn writer_mut(&mut self) -> &mut W { &mut self.callback_env.writer }

    pub fn io_error(&self) -> Option<&io::Error> { self.callback_env.error.as_ref() }
    pub fn take_io_error(&mut self) -> Option<io::Error> { self.callback_env.error.take() }

    pub fn finish(self) -> W {
        let surface = self.surface;
        surface.as_ref().finish();
        drop(surface);

        self.callback_env.writer
    }
}

impl<S: FromRawSurface + AsRef<Surface>, W: io::Write> fmt::Display for Writer<S, W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "support::Writer")
    }
}
