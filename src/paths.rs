// Copyright 2013-2015, The Gtk-rs Project Developers.
// See the COPYRIGHT file at the top-level directory of this distribution.
// Licensed under the MIT license, see the LICENSE file or <http://opensource.org/licenses/MIT>

use std::iter::Iterator;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use c_vec::CVec;
use glib::translate::*;
use ffi::enums::PathDataType;
use ffi::{
    cairo_path_t,
    cairo_path_data_header
};
use ffi;

pub struct Path(cairo_path_t);

impl Path {
    pub fn ensure_status(&mut self) {
        self.0.status.ensure_valid()
    }

    pub fn iter(&self) -> PathSegments {
        unsafe {
            let length = self.0.num_data as usize;
            PathSegments {
                data: CVec::new(self.0.data, length),
                i: 0,
                num_data: length,
            }
        }
    }
}

impl<'a> ToGlibPtr<'a, *const ffi::cairo_path_t> for &'a Path {
    type Storage = &'a Path;

    #[inline]
    fn to_glib_none(&self) -> Stash<'a, *const ffi::cairo_path_t, &'a Path> {
        Stash(&self.0, *self)
    }
}

pub struct BoxedPath(*mut Path);

impl Deref for BoxedPath {
    type Target = Path;

    fn deref(&self) -> &Path {
        unsafe { &*self.0 }
    }
}

impl DerefMut for BoxedPath {
    fn deref_mut(&mut self) -> &mut Path {
        unsafe { &mut *self.0 }
    }
}

impl Drop for BoxedPath {
    fn drop(&mut self) {
        unsafe{
            ffi::cairo_path_destroy(self.0 as *mut cairo_path_t);
        }
    }
}

impl FromGlibPtr<*mut ffi::cairo_path_t> for BoxedPath {
    #[inline]
    unsafe fn from_glib_none(_: *mut ffi::cairo_path_t) -> BoxedPath {
        panic!()
    }

    #[inline]
    unsafe fn from_glib_full(ptr: *mut ffi::cairo_path_t) -> BoxedPath {
        assert!(!ptr.is_null());
        BoxedPath(ptr as *mut Path)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PathSegment {
    MoveTo((f64,f64)),
    LineTo((f64,f64)),
    CurveTo((f64, f64),(f64, f64),(f64, f64)),
    ClosePath
}

pub struct PathSegments {
    data: CVec<[f64; 2]>,
    i: usize,
    num_data: usize
}

impl Iterator for PathSegments {
    type Item = PathSegment;

    fn next(&mut self) -> Option<PathSegment> {
        let i = self.i;

        if i >= self.num_data{
            return None;
        }

        let (data_type, length) = unsafe {
            let data_header: &cairo_path_data_header = transmute(self.data.get(i));
            (data_header.data_type, data_header.length)
        };

        self.i += length as usize;

        let ref data = self.data;

        Some(match data_type {
            PathDataType::MoveTo => PathSegment::MoveTo(to_tuple(data.get(i + 1).unwrap())),
            PathDataType::LineTo => PathSegment::LineTo(to_tuple(data.get(i + 1).unwrap())),
            PathDataType::CurveTo => {
                PathSegment::CurveTo(to_tuple(data.get(i + 1).unwrap()),
                    to_tuple(data.get(i + 2).unwrap()), to_tuple(data.get(i + 3).unwrap()))
            }
            PathDataType::ClosePath => PathSegment::ClosePath
        })
    }
}

fn to_tuple(pair: &[f64; 2]) -> (f64, f64) {
    (pair[0], pair[1])
}
