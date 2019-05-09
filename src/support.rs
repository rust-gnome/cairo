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

impl<W> CallbackEnv<W> {
    fn new(writer: W) -> Box<Self> {
        Box::new(CallbackEnv {
            writer,
            error: None,
        })
    }

    fn as_void(&mut self) -> *mut c_void {
        self as *mut Self as *mut c_void
    }

    unsafe fn from_void(ptr: &mut *mut c_void) -> &mut Self {
        &mut *(*ptr as *mut Self)
    }

    unsafe fn write(&mut self, data: *mut c_uchar, length: c_uint) -> cairo_status_t
        where W: io::Write
    {
        let data = slice::from_raw_parts(data, length as usize);

        let result = match self.writer.write_all(data) {
            Ok(_) => Status::Success,
            Err(e) => {
                self.error = Some(e);
                Status::WriteError
            }
        };

        result.into()
    }
}

#[derive(Debug)]
pub struct Writer<S: FromRawSurface + AsRef<Surface>, W: io::Write> {
    pub surface: S,
    callback_env: Box<CallbackEnv<W>>,
}

impl<S: FromRawSurface + AsRef<Surface>, W: io::Write> Writer<S, W> {
    extern fn write_cb(mut env: *mut c_void, data: *mut c_uchar, length: c_uint) -> cairo_status_t {
        unsafe {
            // Safety: the type of `env` would matches `&mut *self.callback_env` in a `Writer` method.
            let env: &mut CallbackEnv<W> = CallbackEnv::from_void(&mut env);
            env.write(data, length)
        }
    }

    pub fn new(constructor: Constructor, width: f64, height: f64, writer: W) -> Writer<S, W> {
        let mut callback_env = CallbackEnv::new(writer);
        let env_ptr = callback_env.as_void();
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

#[derive(Debug)]
pub struct RefWriter<'w, S: FromRawSurface, W: io::Write + 'w> {
    pub surface: S,
    callback_env: Box<CallbackEnv<&'w mut W>>,
}

impl<'w, S: FromRawSurface, W: io::Write + 'w> RefWriter<'w, S, W> {
    extern fn write_cb(mut env: *mut c_void, data: *mut c_uchar, length: c_uint) -> cairo_status_t {
        unsafe {
            // Safety: the type of `env` would matches `&mut *self.callback_env` in a `Writer` method.
            let env: &mut CallbackEnv<&'w mut W> = CallbackEnv::from_void(&mut env);
            env.write(data, length)
        }
    }

    pub fn new(constructor: Constructor, width: f64, height: f64, writer: &'w mut W) -> RefWriter<'w, S, W> {
        let mut callback_env = CallbackEnv::new(writer);
        let env_ptr = callback_env.as_void();
        let surface = unsafe {
            S::from_raw_surface(constructor(Some(Self::write_cb), env_ptr, width, height))
        };

        RefWriter {
            surface,
            callback_env,
        }
    }

    pub fn io_error(&self) -> Option<&io::Error> { self.callback_env.error.as_ref() }
    pub fn take_io_error(&mut self) -> Option<io::Error> { self.callback_env.error.take() }
}

impl<'w, S: FromRawSurface + AsRef<Surface>, W: io::Write> fmt::Display for RefWriter<'w, S, W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "support::RefWriter")
    }
}
