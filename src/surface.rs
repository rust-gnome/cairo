// Copyright 2015-2016, The Gtk-rs Project Developers.
// See the COPYRIGHT file at the top-level directory of this distribution.
// Licensed under the MIT license, see the LICENSE file or <http://opensource.org/licenses/MIT>

use std::mem;
use libc::c_void;

#[cfg(feature = "glib")]
use glib::translate::*;
use ffi;
use ffi::enums::{
    Content,
    Status,
    SurfaceType,
};

#[derive(Debug)]
pub struct Surface(pub *mut ffi::cairo_surface_t);

impl Surface {
    #[doc(hidden)]
    pub unsafe fn from_raw_none(ptr: *mut ffi::cairo_surface_t) -> Surface {
        assert!(!ptr.is_null());
        ffi::cairo_surface_reference(ptr);
        Surface(ptr)
    }

    #[doc(hidden)]
    pub unsafe fn from_raw_full(ptr: *mut ffi::cairo_surface_t) -> Surface {
        assert!(!ptr.is_null());
        Surface(ptr)
    }

    #[doc(hidden)]
    pub fn to_raw_none(&self) -> *mut ffi::cairo_surface_t {
        self.0
    }

    pub fn status(&self) -> Status {
        unsafe { ffi::cairo_surface_status(self.0) }
    }

    pub fn create_similar(&self, content: Content, width: i32, height: i32) -> Surface {
        unsafe { Self::from_raw_full(ffi::cairo_surface_create_similar(self.0, content, width, height)) }
    }

    #[cfg(macos)]
    pub fn quartz_create(format: Format, width: u32, height: u32) -> Surface {
        unsafe { from_glib_full(ffi::cairo_surface_create_similar(format, width, height)) }
    }

    #[cfg(macos)]
    pub fn quartz_create_for_cg_context(cg_context: CGContextRef, width: c_uint, height: c_uint) -> Surface {
        unsafe { from_glib_full(ffi::cairo_surface_create_similar(cgContext, width, height)) }
    }

    #[cfg(macos)]
    pub fn quartz_get_cg_context(&self) -> CGContextRef {
        unsafe { ffi::cairo_surface_create_similar(self.to_glib_none().0) }
    }
}

#[cfg(feature = "glib")]
impl<'a> ToGlibPtr<'a, *mut ffi::cairo_surface_t> for Surface {
    type Storage = &'a Surface;

    #[inline]
    fn to_glib_none(&'a self) -> Stash<'a, *mut ffi::cairo_surface_t, Self> {
        Stash(self.to_raw_none(), self)
    }
}

#[cfg(feature = "glib")]
impl FromGlibPtrNone<*mut ffi::cairo_surface_t> for Surface {
    #[inline]
    unsafe fn from_glib_none(ptr: *mut ffi::cairo_surface_t) -> Surface {
        Self::from_raw_none(ptr)
    }
}

#[cfg(feature = "glib")]
impl FromGlibPtrFull<*mut ffi::cairo_surface_t> for Surface {
    #[inline]
    unsafe fn from_glib_full(ptr: *mut ffi::cairo_surface_t) -> Surface {
        Self::from_raw_full(ptr)
    }
}

impl AsRef<Surface> for Surface {
    fn as_ref(&self) -> &Surface {
        self
    }
}

impl Clone for Surface {
    fn clone(&self) -> Surface {
        unsafe { Self::from_raw_none(self.0) }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { ffi::cairo_surface_destroy(self.0); }
    }
}

pub trait SurfaceExt {
    fn flush(&self);
    fn finish(&self);
    fn get_type(&self) -> SurfaceType;
}

impl<O: AsRef<Surface>> SurfaceExt for O {
    fn flush(&self) {
        unsafe { ffi::cairo_surface_flush(self.as_ref().0); }
    }

    fn finish(&self) {
        unsafe { ffi::cairo_surface_finish(self.as_ref().0); }
    }

    fn get_type(&self) -> SurfaceType {
        unsafe { ffi::cairo_surface_get_type(self.as_ref().0) }
    }
}

pub trait SurfacePriv {
    unsafe fn set_user_data<K, T>(&self, key: &K, data: Box<T>) -> Result<(), Status>;
}

impl<O: AsRef<Surface>> SurfacePriv for O {
    unsafe fn set_user_data<K, T>(&self, key: &K, data: Box<T>) -> Result<(), Status> {
        let status = ffi::cairo_surface_set_user_data(self.as_ref().0, mem::transmute(key),
            mem::transmute(data), Some(unbox::<T>));
        match status {
            Status::Success => Ok(()),
            x => Err(x),
        }
    }
}

unsafe extern fn unbox<T>(data: *mut c_void) {
    let data: Box<T> = mem::transmute(data);
    drop(data);
}
