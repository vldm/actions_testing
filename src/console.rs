//
// Copyright (c) 2018 Stegos
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use futures::sync::mpsc::UnboundedReceiver;
use futures::{Async, Future, Poll, Stream};
use libp2p::Multiaddr;
use std::mem;
use std::thread;
use std::thread::ThreadId;
use stegos_network::Node;
use tokio_stdin;

/// Console (stdin) service.
pub struct ConsoleService {
    /// Network node.
    node: Node,
    /// A channel to receive message from stdin thread.
    stdin: UnboundedReceiver<u8>,
    /// Input buffer.
    buf: Vec<u8>,
    /// Thread Id (just for debug).
    thread_id: ThreadId,
}

impl ConsoleService {
    fn on_input(&mut self, ch: u8) {
        if ch != b'\r' && ch != b'\n' {
            self.buf.push(ch);
            return;
        } else if self.buf.is_empty() {
            return;
        }

        let msg = String::from_utf8(mem::replace(&mut self.buf, Vec::new())).unwrap();
        if msg.starts_with("/dial ") {
            let target: Multiaddr = msg[6..].parse().unwrap();
            println!("main: *Dialing {}*", target);
            self.node.dial(target).unwrap();
        } else if msg.starts_with("/publish ") {
            let sep_pos = msg[9..].find(' ').unwrap_or(0);
            let topic: String = msg[9..9 + sep_pos].to_string();
            let msg: String = msg[9 + sep_pos + 1..].to_string();
            println!("main: *Publishing to topic '{}': {} *", topic, msg);
            self.node.publish(&topic, msg.as_bytes().to_vec()).unwrap();
        } else {
            eprintln!("Usage:");
            eprintln!("/dial multiaddr");
            eprintln!("/publish topic message");
        }
    }
}

impl ConsoleService {
    /// Constructor.
    pub fn new(net: Node) -> Self {
        let stdin = tokio_stdin::spawn_stdin_stream_unbounded();
        let buf = Vec::<u8>::new();
        let thread_id = thread::current().id();
        ConsoleService {
            node: net,
            stdin,
            buf,
            thread_id,
        }
    }
}

/// Tokio boilerplate.
impl Future for ConsoleService {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        assert_eq!(self.thread_id, thread::current().id());
        loop {
            match self.stdin.poll() {
                Ok(Async::Ready(Some(ch))) => self.on_input(ch),
                Ok(Async::Ready(None)) => unreachable!(),
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(()) => panic!(),
            }
        }
    }
}
