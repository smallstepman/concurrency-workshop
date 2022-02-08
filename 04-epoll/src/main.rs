// https://cfsamsonbooks.gitbook.io/epoll-kqueue-iocp-explained/part-1-an-express-explanation/the-basics
use std::io::{self, Write};
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

mod ffi {
    pub const EPOLL_CTL_ADD: i32 = 1;
    pub const EPOLLIN: i32 = 0x1;
    pub const EPOLLONESHOT: i32 = 0x40000000;

    #[link(name = "c")]
    extern "C" {
        pub fn epoll_create(size: i32) -> i32;
        pub fn close(fd: i32) -> i32;
        pub fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: *mut Event) -> i32;
        pub fn epoll_wait(epfd: i32, events: *mut Event, maxevents: i32, timeout: i32) -> i32;
    }

    #[derive(Debug)]
    #[repr(C, packed)]
    pub struct Event {
        pub(crate) events: u32,
        pub(crate) epoll_data: usize,
    }

    // #[repr(C)]
    // pub union Data {
    //     void: *const std::os::raw::c_void,
    //     fd: i32,
    //     uint32: u32,
    //     uint64: u64,
    // }
}

fn main() {
    let mut event_counter = 0;

    let queue = unsafe { ffi::epoll_create(1) };
    if queue < 0 {
        panic!("{}", io::Error::last_os_error());
    }

    let mut streams = vec![];

    let magic = 40;
    for i in 1..magic {
        let addr = "slowwly.robertomurray.co.uk:80";
        let mut stream = TcpStream::connect(addr).unwrap();
        let delay = (magic - 1 - i) * 1000;
        let request = format!(
            "GET /{delay}/{}/url/http://www.google.com HTTP/1.1\r\n\
             Host: slowwly.robertomurray.co.uk\r\n\
             Connection: close\r\n\
             \r\n",
            delay = delay
        );
        stream.write_all(request.as_bytes()).unwrap();
        stream.set_nonblocking(true).unwrap();

        let mut event = ffi::Event {
            events: (ffi::EPOLLIN | ffi::EPOLLONESHOT) as u32,
            epoll_data: i,
        };
        let res =
            unsafe { ffi::epoll_ctl(queue, ffi::EPOLL_CTL_ADD, stream.as_raw_fd(), &mut event) };
        if res < 0 {
            panic!("{}", io::Error::last_os_error());
        }

        streams.push(stream);
        event_counter += 1;
    }

    while event_counter > 0 {
        let mut events = Vec::with_capacity(10);
        let res = unsafe { ffi::epoll_wait(queue, events.as_mut_ptr(), 10, -1) };
        if res < 0 {
            panic!("{}", io::Error::last_os_error());
        }
        unsafe { events.set_len(res as usize) };
        for event in events {
            println!("RECEIVED: {:?}", event);
            event_counter -= 1;
        }
    }

    let res = unsafe { ffi::close(queue) };
    if res < 0 {
        panic!("{}", io::Error::last_os_error());
    }
    println!("finished");
}
