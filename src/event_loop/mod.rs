use wasi::Subscription;

use crate::{quickjs_sys as qjs, Context, JsValue};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::io;
use std::mem::{zeroed, ManuallyDrop};
use std::ops::{Add, Div};

pub enum PollResult {
    Timeout,
    Read(Vec<u8>),
    Error(io::Error),
    Write(usize),
}

struct TimeoutTask {
    timeout: u128,
    callback: Box<dyn FnOnce(&mut qjs::Context, PollResult)>,
}

impl TimeoutTask {
    fn as_subscription(&self, index: usize) -> Subscription {
        wasi::Subscription {
            userdata: index as u64,
            u: wasi::SubscriptionU {
                tag: wasi::EVENTTYPE_CLOCK.raw(),
                u: wasi::SubscriptionUU {
                    clock: wasi::SubscriptionClock {
                        id: wasi::CLOCKID_REALTIME,
                        timeout: self.timeout as u64,
                        precision: 0,
                        flags: 0, // passing flags here causes wasmtime to crash
                    },
                },
            },
        }
    }
}

struct FdReadTask {
    fd: std::os::wasi::io::RawFd,
    pos: i64,
    len: u64,
    callback: Box<dyn FnOnce(&mut qjs::Context, PollResult)>,
}

impl FdReadTask {
    fn as_subscription(&self, index: usize) -> Subscription {
        wasi::Subscription {
            userdata: index as u64,
            u: wasi::SubscriptionU {
                tag: wasi::EVENTTYPE_FD_READ.raw(),
                u: wasi::SubscriptionUU {
                    fd_read: wasi::SubscriptionFdReadwrite {
                        file_descriptor: self.fd as u32,
                    },
                },
            },
        }
    }
}

struct FdWriteTask {
    fd: std::os::wasi::io::RawFd,
    pos: i64,
    buf: Vec<u8>,
    callback: Box<dyn FnOnce(&mut qjs::Context, PollResult)>,
}

impl FdWriteTask {
    fn as_subscription(&self, index: usize) -> Subscription {
        wasi::Subscription {
            userdata: index as u64,
            u: wasi::SubscriptionU {
                tag: wasi::EVENTTYPE_FD_WRITE.raw(),
                u: wasi::SubscriptionUU {
                    fd_write: wasi::SubscriptionFdReadwrite {
                        file_descriptor: self.fd as u32,
                    },
                },
            },
        }
    }
}

enum PollTask {
    Timeout(TimeoutTask),
    FdRead(FdReadTask),
    FdWrite(FdWriteTask),
}

#[derive(Default)]
struct IoSelector {
    tasks: Vec<Option<PollTask>>,
}

impl IoSelector {
    pub fn add_task(&mut self, task: PollTask) -> usize {
        let mut n = 0;
        for t in &mut self.tasks {
            if t.is_none() {
                t.insert(task);
                return n;
            }
            n += 1;
        }
        self.tasks.push(Some(task));
        n
    }

    pub fn delete_task(&mut self, id: usize) -> Option<PollTask> {
        self.tasks.get_mut(id)?.take()
    }

    pub fn poll(&mut self, ctx: &mut qjs::Context) -> io::Result<usize> {
        let mut subscription_vec = Vec::with_capacity(self.tasks.len());
        for (i, timeout) in self.tasks.iter().enumerate() {
            if let Some(task) = timeout {
                match task {
                    PollTask::Timeout(task) => {
                        subscription_vec.push(task.as_subscription(i));
                    }
                    PollTask::FdRead(task) => {
                        subscription_vec.push(task.as_subscription(i));
                    }
                    PollTask::FdWrite(task) => {
                        subscription_vec.push(task.as_subscription(i));
                    }
                }
            }
        }

        if subscription_vec.is_empty() {
            return Ok(0);
        }

        let mut revent: [wasi::Event; 1] = unsafe { zeroed() };

        let n = unsafe {
            wasi::poll_oneoff(
                subscription_vec.as_ptr(),
                revent.as_mut_ptr(),
                subscription_vec.len(),
            )
            .unwrap_or_else(|e| {
                panic!("failed to poll: {:?}", e);
            })
        };

        for i in 0..n {
            let event = revent[i];
            let index = event.userdata as usize;
            if let Some(task) = self.delete_task(index) {
                match (task, event.type_) {
                    (PollTask::Timeout(TimeoutTask { callback, .. }), wasi::EVENTTYPE_CLOCK) => {
                        callback(ctx, PollResult::Timeout);
                    }
                    (
                        PollTask::FdRead(FdReadTask {
                            fd,
                            pos,
                            len,
                            callback,
                        }),
                        wasi::EVENTTYPE_FD_READ,
                    ) => {
                        if event.error.raw() > 0 {
                            let e = io::Error::from_raw_os_error(event.error.raw().into());
                            callback(ctx, PollResult::Error(e));
                            continue;
                        }
                        let len = len as usize; // len.min(event.fd_readwrite.nbytes) as usize;
                        let mut buf = vec![0u8; len];
                        let res = if pos >= 0 {
                            unsafe {
                                wasi::fd_pread(
                                    fd as u32,
                                    &[wasi::Iovec {
                                        buf: buf.as_mut_ptr(),
                                        buf_len: len,
                                    }],
                                    pos as u64,
                                )
                            }
                        } else {
                            unsafe {
                                wasi::fd_read(
                                    fd as u32,
                                    &[wasi::Iovec {
                                        buf: buf.as_mut_ptr(),
                                        buf_len: len,
                                    }],
                                )
                            }
                        };
                        callback(
                            ctx,
                            match res {
                                Ok(rlen) => {
                                    buf.resize(rlen, 0);
                                    PollResult::Read(buf)
                                }
                                Err(e) => {
                                    PollResult::Error(io::Error::from_raw_os_error(e.raw() as i32))
                                }
                            },
                        );
                    }
                    (
                        PollTask::FdWrite(FdWriteTask {
                            fd,
                            pos,
                            buf,
                            callback,
                        }),
                        wasi::EVENTTYPE_FD_WRITE,
                    ) => {
                        if event.error.raw() > 0 {
                            let e = io::Error::from_raw_os_error(event.error.raw().into());
                            callback(ctx, PollResult::Error(e));
                            continue;
                        }
                        if pos != -1 {
                            let res = unsafe { wasi::fd_seek(fd as u32, pos, wasi::WHENCE_SET) };
                            if let Err(e) = res {
                                callback(
                                    ctx,
                                    PollResult::Error(io::Error::from_raw_os_error(e.raw() as i32)),
                                );
                                continue;
                            }
                        }
                        let res = unsafe {
                            wasi::fd_write(
                                fd as u32,
                                &[wasi::Ciovec {
                                    buf: buf.as_ptr(),
                                    buf_len: buf.len(),
                                }],
                            )
                        };
                        callback(
                            ctx,
                            match res {
                                Ok(len) => PollResult::Write(len),
                                Err(e) => {
                                    PollResult::Error(io::Error::from_raw_os_error(e.raw() as i32))
                                }
                            },
                        );
                    }
                    (_, _) => {}
                }
            }
        }
        Ok(n)
    }
}

#[derive(Default)]
pub struct EventLoop {
    next_tick_queue: LinkedList<Box<dyn FnOnce(&mut qjs::Context)>>,
    io_selector: IoSelector,
}

impl EventLoop {
    pub fn run_once(&mut self, ctx: &mut qjs::Context) -> io::Result<usize> {
        let n = self.run_tick_task(ctx);
        if n > 0 {
            Ok(n)
        } else {
            self.io_selector.poll(ctx)
        }
    }

    fn run_tick_task(&mut self, ctx: &mut qjs::Context) -> usize {
        let mut i = 0;
        while let Some(f) = self.next_tick_queue.pop_front() {
            f(ctx);
            i += 1;
        }
        i
    }

    pub fn set_timeout(
        &mut self,
        callback: qjs::JsFunction,
        timeout: std::time::Duration,
        args: Option<Vec<JsValue>>,
    ) -> usize {
        let timeout_task = PollTask::Timeout(TimeoutTask {
            timeout: timeout.as_nanos().into(),
            callback: Box::new(move |_ctx, _res| {
                match args {
                    Some(argv) => callback.call(&argv),
                    None => callback.call(&[]),
                };
            }),
        });
        self.io_selector.add_task(timeout_task)
    }

    pub fn clear_timeout(&mut self, timeout_id: usize) {
        if let Some(t) = self.io_selector.tasks.get_mut(timeout_id) {
            if let Some(PollTask::Timeout(_)) = t {
                t.take();
            };
        };
    }

    pub fn set_next_tick(&mut self, callback: Box<dyn FnOnce(&mut qjs::Context)>) {
        self.next_tick_queue.push_back(callback);
    }

    pub fn fd_read(
        &mut self,
        fd: std::os::wasi::io::RawFd,
        pos: i64,
        len: u64,
        callback: Box<dyn FnOnce(&mut qjs::Context, PollResult)>,
    ) {
        self.io_selector.add_task(PollTask::FdRead(FdReadTask {
            fd,
            pos,
            len,
            callback,
        }));
    }

    pub fn fd_write(
        &mut self,
        fd: std::os::wasi::io::RawFd,
        pos: i64,
        buf: Vec<u8>,
        callback: Box<dyn FnOnce(&mut qjs::Context, PollResult)>,
    ) {
        self.io_selector.add_task(PollTask::FdWrite(FdWriteTask {
            fd,
            pos,
            buf,
            callback,
        }));
    }
}
