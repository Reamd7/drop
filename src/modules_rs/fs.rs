use wasi::Fd;

use crate::event_loop::PollResult;
use crate::quickjs_sys::*;
use std::convert::TryInto;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs;
use std::fs::Permissions;
use std::io;
use std::os::wasi::prelude::FromRawFd;
use std::ptr;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// Attempts to open a bare path `p`.
///
/// WASI has no fundamental capability to do this. All syscalls and operations
/// are relative to already-open file descriptors. The C library, however,
/// manages a map of pre-opened file descriptors to their path, and then the C
/// library provides an API to look at this. In other words, when you want to
/// open a path `p`, you have to find a previously opened file descriptor in a
/// global table and then see if `p` is relative to that file descriptor.
///
/// This function, if successful, will return two items:
///
/// * The first is a `ManuallyDrop<WasiFd>`. This represents a pre-opened file
///   descriptor which we don't have ownership of, but we can use. You shouldn't
///   actually drop the `fd`.
///
/// * The second is a path that should be a part of `p` and represents a
///   relative traversal from the file descriptor specified to the desired
///   location `p`.
///
/// If successful you can use the returned file descriptor to perform
/// file-descriptor-relative operations on the path returned as well. The
/// `rights` argument indicates what operations are desired on the returned file
/// descriptor, and if successful the returned file descriptor should have the
/// appropriate rights for performing `rights` actions.
///
/// Note that this can fail if `p` doesn't look like it can be opened relative
/// to any pre-opened file descriptor.
pub fn open_parent(p: &str) -> io::Result<(Fd, String)> {
    let p = CString::new(p.as_bytes())?;
    let mut buf = Vec::<u8>::with_capacity(512);
    loop {
        unsafe {
            let mut relative_path = buf.as_ptr().cast();
            let mut abs_prefix = ptr::null();
            let fd = __wasilibc_find_relpath(
                p.as_ptr(),
                &mut abs_prefix,
                &mut relative_path,
                buf.capacity(),
            );
            if fd == -1 {
                if io::Error::last_os_error().raw_os_error() == Some(libc::ENOMEM) {
                    // Trigger the internal buffer resizing logic of `Vec` by requiring
                    // more space than the current capacity.
                    let cap = buf.capacity();
                    buf.set_len(cap);
                    buf.reserve(1);
                    continue;
                }
                let msg = format!(
                    "failed to find a pre-opened file descriptor \
                     through which {:?} could be opened",
                    p
                );
                // io::ErrorKind::Uncategorized is unstable
                // return Err(io::Error::new(io::ErrorKind::Uncategorized, msg));
                return Err(io::Error::new(io::ErrorKind::Other, msg));
            }
            let relative = CStr::from_ptr(relative_path).to_bytes().to_vec();

            return Ok((fd as Fd, String::from_utf8(relative).unwrap()));
        }
    }

    extern "C" {
        pub fn __wasilibc_find_relpath(
            path: *const libc::c_char,
            abs_prefix: *mut *const libc::c_char,
            relative_path: *mut *const libc::c_char,
            relative_path_len: libc::size_t,
        ) -> libc::c_int;
    }
}

impl From<u64> for JsValue {
    fn from(val: u64) -> Self {
        JsValue::Float(val as f64)
    }
}

fn permissions_to_mode(permit: Permissions) -> i32 {
    const F_OK: i32 = 0;
    const R_OK: i32 = 4;
    const W_OK: i32 = 2;
    const X_OK: i32 = 1;
    let p = if permit.readonly() {
        F_OK | R_OK | X_OK
    } else {
        F_OK | R_OK | W_OK | X_OK
    };
    p | p << 3 | p << 6
}

fn stat_to_js_object(ctx: &mut Context, stat: wasi::Filestat) -> JsValue {
    let mut res = ctx.new_object();
    res.set(
        "is_file",
        (stat.filetype == wasi::FILETYPE_REGULAR_FILE).into(),
    );
    res.set(
        "is_directory",
        (stat.filetype == wasi::FILETYPE_DIRECTORY).into(),
    );
    res.set(
        "is_symlink",
        (stat.filetype == wasi::FILETYPE_SYMBOLIC_LINK).into(),
    );
    res.set(
        "is_block_device",
        (stat.filetype == wasi::FILETYPE_BLOCK_DEVICE).into(),
    );
    res.set(
        "is_char_device",
        (stat.filetype == wasi::FILETYPE_CHARACTER_DEVICE).into(),
    );
    res.set(
        "is_socket",
        (stat.filetype == wasi::FILETYPE_SOCKET_DGRAM
            || stat.filetype == wasi::FILETYPE_SOCKET_STREAM)
            .into(),
    );
    res.set("size", stat.size.into());
    res.set("mtime", (stat.mtim / 1000000).into());
    res.set("atime", (stat.atim / 1000000).into());
    res.set("birthtime", (stat.ctim / 1000000).into());
    res.set("dev", stat.dev.into());
    res.set("ino", stat.ino.into());
    res.set("mode", 0o666.into());
    res.set("nlink", stat.nlink.into());
    res.set("uid", 0.into());
    res.set("gid", 0.into());
    res.set("rdev", 0.into());
    res.set("blksize", 0.into());
    res.set("blocks", 0.into());
    JsValue::Object(res)
}

fn err_to_js_object(_ctx: &mut Context, e: io::Error) -> JsValue {
    panic!("err_to_js_object: {:?}", e);
    // errno_to_js_object(ctx, wasi::Errno(e.raw_os_error().unwrap() as u16))
}

fn errno_to_js_object(ctx: &mut Context, e: wasi::Errno) -> JsValue {
    let mut res = ctx.new_object();
    res.set("message", JsValue::String(ctx.new_string(e.message())));
    res.set("code", JsValue::String(ctx.new_string(e.name())));
    res.set("errno", JsValue::Int(e.raw() as i32));
    JsValue::Object(res)
}

fn stat_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let path = arg.get(0);
    if path.is_none() {
        return JsValue::UnDefined;
    }
    if let JsValue::String(s) = path.unwrap() {
        let (dir, file) = match open_parent(s.as_str()) {
            Ok(ok) => ok,
            Err(e) => {
                return {
                    let err = err_to_js_object(ctx, e);
                    JsValue::Exception(ctx.throw_error(err))
                }
            }
        };
        return match unsafe {
            wasi::path_filestat_get(dir, wasi::LOOKUPFLAGS_SYMLINK_FOLLOW, file.as_str())
        } {
            Ok(stat) => stat_to_js_object(ctx, stat),
            Err(e) => {
                let err = errno_to_js_object(ctx, e);
                JsValue::Exception(ctx.throw_error(err))
            }
        };
    } else {
        return JsValue::UnDefined;
    }
}

fn fstat_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let fd = arg.get(0);
    if fd.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(f) = get_js_number(fd) {
        return match unsafe { wasi::fd_filestat_get(f as u32) } {
            Ok(stat) => stat_to_js_object(ctx, stat),
            Err(e) => {
                let err = errno_to_js_object(ctx, e);
                JsValue::Exception(ctx.throw_error(err))
            }
        };
    } else {
        return JsValue::UnDefined;
    }
}

fn lstat_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let path = arg.get(0);
    if path.is_none() {
        return JsValue::UnDefined;
    }
    if let JsValue::String(s) = path.unwrap() {
        let (dir, file) = match open_parent(s.as_str()) {
            Ok(ok) => ok,
            Err(e) => {
                return {
                    let err = err_to_js_object(ctx, e);
                    JsValue::Exception(ctx.throw_error(err))
                }
            }
        };
        return match unsafe { wasi::path_filestat_get(dir, 0, file.as_str()) } {
            Ok(stat) => stat_to_js_object(ctx, stat),
            Err(e) => {
                let err = errno_to_js_object(ctx, e);
                JsValue::Exception(ctx.throw_error(err))
            }
        };
    } else {
        return JsValue::UnDefined;
    }
}

fn mkdir_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let path = arg.get(0);
    let recursive = arg.get(1);
    let mode = arg.get(2);
    if path.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::String(s)) = path {
        if let Some(JsValue::Bool(r)) = recursive {
            if let Some(JsValue::Int(_m)) = mode {
                let res = if *r {
                    fs::create_dir_all(s.as_str())
                } else {
                    fs::create_dir(s.as_str())
                };
                return match res {
                    Ok(()) => JsValue::UnDefined,
                    Err(e) => {
                        let err = err_to_js_object(ctx, e);
                        JsValue::Exception(ctx.throw_error(err))
                    }
                };
            }
        }
    }
    return JsValue::UnDefined;
}

fn rmdir_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let path = arg.get(0);
    let recursive = arg.get(1);
    if path.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::String(s)) = path {
        if let Some(JsValue::Bool(r)) = recursive {
            let res = if *r {
                fs::remove_dir_all(s.as_str())
            } else {
                fs::remove_dir(s.as_str())
            };
            return match res {
                Ok(()) => JsValue::UnDefined,
                Err(e) => {
                    let err = err_to_js_object(ctx, e);
                    JsValue::Exception(ctx.throw_error(err))
                }
            };
        }
    }
    return JsValue::UnDefined;
}

fn rm_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let path = arg.get(0);
    let recursive = arg.get(1);
    let force = arg.get(2);
    if path.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::String(s)) = path {
        if let Some(JsValue::Bool(r)) = recursive {
            if let Some(JsValue::Bool(f)) = force {
                let res = fs::metadata(s.as_str()).and_then(|stat| {
                    if stat.is_file() {
                        fs::remove_file(s.as_str())
                    } else {
                        if *r {
                            fs::remove_dir_all(s.as_str())
                        } else {
                            fs::remove_dir(s.as_str())
                        }
                    }
                });
                return match res {
                    Ok(()) => JsValue::UnDefined,
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::NotFound && *f {
                            JsValue::UnDefined
                        } else {
                            let err = err_to_js_object(ctx, e);
                            JsValue::Exception(ctx.throw_error(err))
                        }
                    }
                };
            }
        }
    }
    return JsValue::UnDefined;
}

fn rename_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let old_path = arg.get(0);
    let new_path = arg.get(1);
    if old_path.is_none() || new_path.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::String(from)) = old_path {
        if let Some(JsValue::String(to)) = new_path {
            return match fs::rename(from.as_str(), to.as_str()) {
                Ok(()) => JsValue::UnDefined,
                Err(e) => {
                    let err = err_to_js_object(ctx, e);
                    JsValue::Exception(ctx.throw_error(err))
                }
            };
        }
    }
    return JsValue::UnDefined;
}

fn truncate_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let path = arg.get(0);
    let len = arg.get(1);
    if path.is_none() || len.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::String(p)) = path {
        if let Some(l) = get_js_number(len) {
            let res = fs::OpenOptions::new()
                .write(true)
                .open(p.as_str())
                .and_then(|file| file.set_len(l as u64));
            return match res {
                Ok(()) => JsValue::UnDefined,
                Err(e) => {
                    let err = err_to_js_object(ctx, e);
                    JsValue::Exception(ctx.throw_error(err))
                }
            };
        }
    }
    return JsValue::UnDefined;
}

fn ftruncate_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let fd = arg.get(0);
    let len = arg.get(1);
    if fd.is_none() || len.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::Int(f)) = fd {
        if let Some(l) = get_js_number(len) {
            let res = unsafe { wasi::fd_filestat_set_size(*f as u32, l as u64) };
            return match res {
                Ok(()) => JsValue::UnDefined,
                Err(e) => {
                    let err = errno_to_js_object(ctx, e);
                    JsValue::Exception(ctx.throw_error(err))
                }
            };
        }
    }
    return JsValue::UnDefined;
}

fn realpath_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let path = arg.get(0);
    if path.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::String(p)) = path {
        let (dir, file) = match open_parent(p.as_str()) {
            Ok(ok) => ok,
            Err(e) => {
                return {
                    let err = err_to_js_object(ctx, e);
                    JsValue::Exception(ctx.throw_error(err))
                }
            }
        };
        let mut buf = vec![0; 1024];
        let res = unsafe { wasi::path_readlink(dir, file.as_str(), buf.as_mut_ptr(), buf.len()) };
        return match res {
            Ok(size) => ctx
                .new_string(std::str::from_utf8(&buf[0..size]).unwrap())
                .into(),
            Err(e) => {
                let err = errno_to_js_object(ctx, e);
                JsValue::Exception(ctx.throw_error(err))
            }
        };
    }
    return JsValue::UnDefined;
}

fn copy_file_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let from_path = arg.get(0);
    let to_path = arg.get(1);
    if from_path.is_none() || to_path.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::String(from)) = from_path {
        if let Some(JsValue::String(to)) = to_path {
            let res = fs::copy(from.as_str(), to.as_str());
            return match res {
                Ok(_) => JsValue::UnDefined,
                Err(e) => {
                    let err = err_to_js_object(ctx, e);
                    JsValue::Exception(ctx.throw_error(err))
                }
            };
        }
    }
    return JsValue::UnDefined;
}

fn link_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let from_path = arg.get(0);
    let to_path = arg.get(1);
    if from_path.is_none() || to_path.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::String(from)) = from_path {
        if let Some(JsValue::String(to)) = to_path {
            let res = fs::hard_link(from.as_str(), to.as_str());
            return match res {
                Ok(_) => JsValue::UnDefined,
                Err(e) => {
                    let err = err_to_js_object(ctx, e);
                    JsValue::Exception(ctx.throw_error(err))
                }
            };
        }
    }
    return JsValue::UnDefined;
}

fn symlink_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let from_path = arg.get(0);
    let to_path = arg.get(1);
    if from_path.is_none() || to_path.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::String(from)) = from_path {
        if let Some(JsValue::String(to)) = to_path {
            let (dir, file) = match open_parent(to.as_str()) {
                Ok(ok) => ok,
                Err(e) => {
                    return {
                        let err = err_to_js_object(ctx, e);
                        JsValue::Exception(ctx.throw_error(err))
                    }
                }
            };
            let res = unsafe { wasi::path_symlink(from.as_str(), dir, file.as_str()) };
            return match res {
                Ok(_) => JsValue::UnDefined,
                Err(e) => {
                    let err = errno_to_js_object(ctx, e);
                    JsValue::Exception(ctx.throw_error(err))
                }
            };
        }
    }
    return JsValue::UnDefined;
}

fn utime_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let path = arg.get(0);
    let atime = arg.get(1);
    let mtime = arg.get(2);
    if path.is_none() || atime.is_none() || mtime.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::String(p)) = path {
        if let Some(a) = get_js_number(atime) {
            if let Some(m) = get_js_number(mtime) {
                let (dir, file) = match open_parent(p.as_str()) {
                    Ok(ok) => ok,
                    Err(e) => {
                        return {
                            let err = err_to_js_object(ctx, e);
                            JsValue::Exception(ctx.throw_error(err))
                        }
                    }
                };
                let res = unsafe {
                    wasi::path_filestat_set_times(
                        dir,
                        wasi::LOOKUPFLAGS_SYMLINK_FOLLOW,
                        file.as_str(),
                        (a as u64) * 1000000,
                        (m as u64) * 1000000,
                        wasi::FSTFLAGS_ATIM | wasi::FSTFLAGS_MTIM,
                    )
                };
                return match res {
                    Ok(_) => JsValue::UnDefined,
                    Err(e) => {
                        let err = errno_to_js_object(ctx, e);
                        JsValue::Exception(ctx.throw_error(err))
                    }
                };
            }
        }
    }
    return JsValue::UnDefined;
}

fn lutime_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let path = arg.get(0);
    let atime = arg.get(1);
    let mtime = arg.get(2);
    if path.is_none() || atime.is_none() || mtime.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::String(p)) = path {
        if let Some(a) = get_js_number(atime) {
            if let Some(m) = get_js_number(mtime) {
                let (dir, file) = match open_parent(p.as_str()) {
                    Ok(ok) => ok,
                    Err(e) => {
                        return {
                            let err = err_to_js_object(ctx, e);
                            JsValue::Exception(ctx.throw_error(err))
                        }
                    }
                };
                let res = unsafe {
                    wasi::path_filestat_set_times(
                        dir,
                        0,
                        file.as_str(),
                        (a as u64) * 1000000,
                        (m as u64) * 1000000,
                        wasi::FSTFLAGS_ATIM | wasi::FSTFLAGS_MTIM,
                    )
                };
                return match res {
                    Ok(_) => JsValue::UnDefined,
                    Err(e) => {
                        let err = errno_to_js_object(ctx, e);
                        JsValue::Exception(ctx.throw_error(err))
                    }
                };
            }
        }
    }
    return JsValue::UnDefined;
}

fn futime_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let fd = arg.get(0);
    let atime = arg.get(1);
    let mtime = arg.get(2);
    if fd.is_none() || atime.is_none() || mtime.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::Int(f)) = fd {
        if let Some(JsValue::Float(a)) = atime {
            if let Some(JsValue::Float(m)) = mtime {
                let res = unsafe {
                    wasi::fd_filestat_set_times(
                        *f as u32,
                        *a as u64,
                        *m as u64,
                        wasi::FSTFLAGS_ATIM | wasi::FSTFLAGS_MTIM,
                    )
                };
                return match res {
                    Ok(_) => JsValue::UnDefined,
                    Err(e) => {
                        let err = errno_to_js_object(ctx, e);
                        JsValue::Exception(ctx.throw_error(err))
                    }
                };
            }
        }
    }
    return JsValue::UnDefined;
}

fn fclose_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let fd = arg.get(0);
    if fd.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::Int(f)) = fd {
        let res = unsafe { wasi::fd_close(*f as u32) };
        return match res {
            Ok(_) => JsValue::UnDefined,
            Err(e) => {
                let err = errno_to_js_object(ctx, e);
                JsValue::Exception(ctx.throw_error(err))
            }
        };
    }
    return JsValue::UnDefined;
}

fn fdatasync_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let fd = arg.get(0);
    if fd.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::Int(f)) = fd {
        let res = unsafe { wasi::fd_datasync(*f as u32) };
        return match res {
            Ok(_) => JsValue::UnDefined,
            Err(e) => {
                let err = errno_to_js_object(ctx, e);
                JsValue::Exception(ctx.throw_error(err))
            }
        };
    }
    return JsValue::UnDefined;
}

fn fsync_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    let fd = arg.get(0);
    if fd.is_none() {
        return JsValue::UnDefined;
    }
    if let Some(JsValue::Int(f)) = fd {
        let res = unsafe { wasi::fd_sync(*f as u32) };
        return match res {
            Ok(_) => JsValue::UnDefined,
            Err(e) => {
                let err = errno_to_js_object(ctx, e);
                JsValue::Exception(ctx.throw_error(err))
            }
        };
    }
    return JsValue::UnDefined;
}

fn get_js_number(val: Option<&JsValue>) -> Option<i64> {
    match val {
        Some(JsValue::Int(i)) => Some(*i as i64),
        Some(JsValue::Float(f)) => Some(*f as i64),
        _ => None,
    }
}

fn fread(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    if let Some(JsValue::Int(fd)) = arg.get(0) {
        if let Some(position) = get_js_number(arg.get(1)) {
            if let Some(JsValue::Int(length)) = arg.get(2) {
                let (promise, ok, error) = ctx.new_promise();
                if let Some(event_loop) = ctx.event_loop() {
                    event_loop.fd_read(
                        *fd,
                        position,
                        *length as u64,
                        Box::new(move |ctx, res| match res {
                            PollResult::Read(data) => {
                                let buf = ctx.new_array_buffer(&data);
                                if let JsValue::Function(resolve) = ok {
                                    resolve.call(&[JsValue::ArrayBuffer(buf)]);
                                }
                            }
                            PollResult::Error(e) => {
                                if let JsValue::Function(reject) = error {
                                    reject.call(&[err_to_js_object(ctx, e)]);
                                }
                            }
                            _ => {}
                        }),
                    );
                    return promise;
                }
            }
        }
    }
    return JsValue::UnDefined;
}

fn fread_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    if let Some(JsValue::Int(fd)) = arg.get(0) {
        if let Some(position) = get_js_number(arg.get(1)) {
            if let Some(JsValue::Int(length)) = arg.get(2) {
                let len = *length as usize;
                let mut buf = vec![0; len];
                let res = if position >= 0 {
                    unsafe {
                        wasi::fd_pread(
                            *fd as u32,
                            &[wasi::Iovec {
                                buf: buf.as_mut_ptr(),
                                buf_len: len,
                            }],
                            position as u64,
                        )
                    }
                } else {
                    unsafe {
                        wasi::fd_read(
                            *fd as u32,
                            &[wasi::Iovec {
                                buf: buf.as_mut_ptr(),
                                buf_len: len,
                            }],
                        )
                    }
                };
                return match res {
                    Ok(rlen) => {
                        let data = ctx.new_array_buffer(&buf[0..rlen]);
                        JsValue::ArrayBuffer(data)
                    }
                    Err(e) => {
                        let err = errno_to_js_object(ctx, e);
                        JsValue::Exception(ctx.throw_error(err))
                    }
                };
            }
        }
    }
    return JsValue::UnDefined;
}

fn open_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    if let Some(JsValue::String(path)) = arg.get(0) {
        if let Some(JsValue::Int(flag)) = arg.get(1) {
            if let Some(JsValue::Int(_mode)) = arg.get(2) {
                let fdflag = if flag & 128 == 128 {
                    wasi::FDFLAGS_NONBLOCK
                } else {
                    // passing this here seems unnecessary and actually crashes wasmtime?
                    // wasi::FDFLAGS_SYNC
                    0
                } | if flag & 8 == 8 {
                    wasi::FDFLAGS_APPEND
                } else {
                    0
                };
                let oflag = if flag & 512 == 512 {
                    wasi::OFLAGS_CREAT
                } else {
                    0
                } | if flag & 2048 == 2048 {
                    wasi::OFLAGS_EXCL
                } else {
                    0
                } | if flag & 1024 == 1024 {
                    wasi::OFLAGS_TRUNC
                } else {
                    0
                };
                let right = if flag & 1 == 1 || flag & 2 == 2 {
                    wasi::RIGHTS_FD_WRITE
                        | wasi::RIGHTS_FD_ADVISE
                        | wasi::RIGHTS_FD_ALLOCATE
                        | wasi::RIGHTS_FD_DATASYNC
                        | wasi::RIGHTS_FD_FDSTAT_SET_FLAGS
                        | wasi::RIGHTS_FD_FILESTAT_SET_SIZE
                        | wasi::RIGHTS_FD_FILESTAT_SET_TIMES
                        | wasi::RIGHTS_FD_SYNC
                        | wasi::RIGHTS_FD_WRITE
                } else {
                    0
                } | wasi::RIGHTS_FD_FILESTAT_GET
                    | wasi::RIGHTS_FD_SEEK
                    | wasi::RIGHTS_POLL_FD_READWRITE
                    | wasi::RIGHTS_FD_READ
                    | wasi::RIGHTS_FD_READDIR;
                let (dir, file) = match open_parent(path.as_str()) {
                    Ok(ok) => ok,
                    Err(e) => {
                        return {
                            let err = err_to_js_object(ctx, e);
                            JsValue::Exception(ctx.throw_error(err))
                        }
                    }
                };
                let res =
                    unsafe { wasi::path_open(dir, 0, file.as_str(), oflag, right, 0, fdflag) };
                return match res {
                    Ok(fd) => JsValue::Int(fd as i32),
                    Err(e) => {
                        let err = errno_to_js_object(ctx, e);
                        JsValue::Exception(ctx.throw_error(err))
                    }
                };
            }
        }
    }
    return JsValue::UnDefined;
}

fn readlink_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    if let Some(JsValue::String(path)) = arg.get(0) {
        let mut buf = vec![0; 1024];
        let (dir, file) = match open_parent(path.as_str().into()) {
            Ok(ok) => ok,
            Err(e) => {
                return {
                    let err = err_to_js_object(ctx, e);
                    JsValue::Exception(ctx.throw_error(err))
                }
            }
        };
        let res = unsafe { wasi::path_readlink(dir, file.as_str(), buf.as_mut_ptr(), buf.len()) };
        return match res {
            Ok(_len) => match String::from_utf8(buf) {
                Ok(s) => ctx.new_string(s.as_str()).into(),
                Err(e) => ctx.new_error(e.to_string().as_str()),
            },
            Err(e) => {
                let err = errno_to_js_object(ctx, e);
                JsValue::Exception(ctx.throw_error(err))
            }
        };
    }
    return JsValue::UnDefined;
}

fn fwrite(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    if let Some(JsValue::Int(fd)) = arg.get(0) {
        if let Some(position) = get_js_number(arg.get(1)) {
            if let Some(JsValue::ArrayBuffer(buf)) = arg.get(2) {
                let (promise, ok, error) = ctx.new_promise();
                if let Some(event_loop) = ctx.event_loop() {
                    event_loop.fd_write(
                        *fd,
                        position,
                        buf.to_vec(),
                        Box::new(move |ctx, res| match res {
                            PollResult::Write(len) => {
                                if let JsValue::Function(resolve) = ok {
                                    resolve.call(&[JsValue::Int(len as i32)]);
                                }
                            }
                            PollResult::Error(e) => {
                                if let JsValue::Function(reject) = error {
                                    reject.call(&[err_to_js_object(ctx, e)]);
                                }
                            }
                            _ => {}
                        }),
                    );
                    return promise;
                }
            }
        }
    }
    return JsValue::UnDefined;
}

fn fwrite_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    if let Some(JsValue::Int(fd)) = arg.get(0) {
        if let Some(JsValue::Int(position)) = arg.get(1) {
            if let Some(JsValue::Object(obj)) = arg.get(2) {
                let buf = JsArrayBuffer(obj.js_ref().to_owned());
                if *position >= 0 {
                    let res =
                        unsafe { wasi::fd_seek(*fd as u32, *position as i64, wasi::WHENCE_SET) };
                    if let Err(e) = res {
                        let err = errno_to_js_object(ctx, e);
                        return JsValue::Exception(ctx.throw_error(err));
                    }
                }
                let data = buf.to_vec();
                let res = unsafe {
                    wasi::fd_write(
                        *fd as u32,
                        &[wasi::Ciovec {
                            buf: data.as_ptr(),
                            buf_len: data.len(),
                        }],
                    )
                };
                return match res {
                    Ok(len) => JsValue::Int(len as i32),
                    Err(e) => {
                        let err = errno_to_js_object(ctx, e);
                        JsValue::Exception(ctx.throw_error(err))
                    }
                };
            }
        }
    }
    return JsValue::UnDefined;
}

fn freaddir_sync(ctx: &mut Context, _this_val: JsValue, arg: &[JsValue]) -> JsValue {
    if let Some(JsValue::Int(fd)) = arg.get(0) {
        if let Some(JsValue::Int(cookie)) = arg.get(1) {
            let mut buf = vec![0; 4096];
            let res = unsafe {
                wasi::fd_readdir(*fd as u32, buf.as_mut_ptr(), buf.len(), *cookie as u64)
            };
            return match res {
                Ok(len) => {
                    let s = std::mem::size_of::<wasi::Dirent>();
                    let mut idx = 0;
                    let mut data_pack = ctx.new_array();
                    let mut aidx = 0;
                    let mut dir_next = 0;
                    while (idx + s) < len.min(4096) {
                        let dir = unsafe {
                            *(&buf[idx..(idx + s)] as *const [u8] as *const wasi::Dirent)
                        };
                        idx += s;
                        if (idx + dir.d_namlen as usize) >= len.min(4096) {
                            break;
                        }
                        let name =
                            String::from_utf8_lossy(&buf[idx..(idx + dir.d_namlen as usize)])
                                .to_string();
                        idx += dir.d_namlen as usize;
                        let mut dirent = ctx.new_object();

                        dirent.set("filetype", JsValue::Int(dir.d_type.raw() as i32));
                        dirent.set("name", ctx.new_string(name.as_str()).into());
                        data_pack.put(aidx, dirent.into());
                        dir_next = dir.d_next;
                        aidx += 1;
                    }
                    let mut data = ctx.new_object();
                    data.set("res", data_pack.into());
                    data.set("fin", (len < buf.len()).into());
                    data.set("cookie", JsValue::Int(dir_next as i32));
                    data.into()
                }
                Err(e) => {
                    let err = errno_to_js_object(ctx, e);
                    JsValue::Exception(ctx.throw_error(err))
                }
            };
        }
    }
    return JsValue::UnDefined;
}

struct FS;

impl ModuleInit for FS {
    fn init_module(ctx: &mut Context, m: &mut JsModuleDef) {
        let stat_s = ctx.wrap_function("statSync", stat_sync);
        let lstat_s = ctx.wrap_function("lstatSync", lstat_sync);
        let fstat_s = ctx.wrap_function("fstatSync", fstat_sync);
        let mkdir_s = ctx.wrap_function("mkdirSync", mkdir_sync);
        let rmdir_s = ctx.wrap_function("rmdirSync", rmdir_sync);
        let rm_s = ctx.wrap_function("rmSync", rm_sync);
        let rename_s = ctx.wrap_function("renameSync", rename_sync);
        let truncate_s = ctx.wrap_function("truncateSync", truncate_sync);
        let ftruncate_s = ctx.wrap_function("ftruncateSync", ftruncate_sync);
        let realpath_s = ctx.wrap_function("realpathSync", realpath_sync);
        let copy_file_s = ctx.wrap_function("copyFileSync", copy_file_sync);
        let link_s = ctx.wrap_function("linkSync", link_sync);
        let symlink_s = ctx.wrap_function("symlinkSync", symlink_sync);
        let utime_s = ctx.wrap_function("utimeSync", utime_sync);
        let lutime_s = ctx.wrap_function("lutimeSync", lutime_sync);
        let futime_s = ctx.wrap_function("futimeSync", futime_sync);
        let fclose_s = ctx.wrap_function("fcloseSync", fclose_sync);
        let fsync_s = ctx.wrap_function("fsyncSync", fsync_sync);
        let fdatasync_s = ctx.wrap_function("fdatasyncSync", fdatasync_sync);
        let fread_s = ctx.wrap_function("freadSync", fread_sync);
        let fread_a = ctx.wrap_function("fread", fread);
        let open_s = ctx.wrap_function("openSync", open_sync);
        let readlink_s = ctx.wrap_function("readlinkSync", readlink_sync);
        let fwrite_s = ctx.wrap_function("fwriteSync", fwrite_sync);
        let fwrite_a = ctx.wrap_function("fwrite", fwrite);
        let freaddir_s = ctx.wrap_function("freaddirSync", freaddir_sync);
        m.add_export("statSync", stat_s.into());
        m.add_export("lstatSync", lstat_s.into());
        m.add_export("fstatSync", fstat_s.into());
        m.add_export("mkdirSync", mkdir_s.into());
        m.add_export("rmdirSync", rmdir_s.into());
        m.add_export("rmSync", rm_s.into());
        m.add_export("renameSync", rename_s.into());
        m.add_export("truncateSync", truncate_s.into());
        m.add_export("ftruncateSync", ftruncate_s.into());
        m.add_export("realpathSync", realpath_s.into());
        m.add_export("copyFileSync", copy_file_s.into());
        m.add_export("linkSync", link_s.into());
        m.add_export("symlinkSync", symlink_s.into());
        m.add_export("utimeSync", utime_s.into());
        m.add_export("lutimeSync", lutime_s.into());
        m.add_export("futimeSync", futime_s.into());
        m.add_export("fcloseSync", fclose_s.into());
        m.add_export("fsyncSync", fsync_s.into());
        m.add_export("fdatasyncSync", fdatasync_s.into());
        m.add_export("freadSync", fread_s.into());
        m.add_export("fread", fread_a.into());
        m.add_export("openSync", open_s.into());
        m.add_export("readlinkSync", readlink_s.into());
        m.add_export("fwriteSync", fwrite_s.into());
        m.add_export("fwrite", fwrite_a.into());
        m.add_export("freaddirSync", freaddir_s.into());
    }
}

pub fn init_module(ctx: &mut Context) {
    ctx.register_module(
        "_node:fs\0",
        FS,
        &[
            "statSync\0",
            "lstatSync\0",
            "fstatSync\0",
            "mkdirSync\0",
            "rmdirSync\0",
            "rmSync\0",
            "renameSync\0",
            "truncateSync\0",
            "ftruncateSync\0",
            "realpathSync\0",
            "copyFileSync\0",
            "linkSync\0",
            "symlinkSync\0",
            "utimeSync\0",
            "lutimeSync\0",
            "futimeSync\0",
            "fcloseSync\0",
            "fsyncSync\0",
            "fdatasyncSync\0",
            "freadSync\0",
            "fread\0",
            "openSync\0",
            "readlinkSync\0",
            "fwriteSync\0",
            "fwrite\0",
            "freaddirSync\0",
        ],
    )
}
