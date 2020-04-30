use async_std::task;
use std::future::Future;
use std::task::{ Poll, Context, Waker };
use std::pin::Pin;
use std::sync::{
    Mutex,
    Arc,
};
use std::time::Duration;
use std::thread::sleep;

static mut COUNTER: usize = 0;
static LEVEL: usize = 25;

struct SharedState {
    completed: bool,
    cnt: usize,
    waker: Option<Waker>,
}

enum Node {
    Never,
    Now(Arc<Mutex<SharedState>>),
    Pair(Box<Node>, Box<Node>),
}

impl Node {
    fn new() -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState{
            completed: false,
            cnt: 0,
            waker: None
        }));
        let cloned = shared_state.clone();
        task::spawn(async move {
            let duration = Duration::from_millis(10);
            let mut prev_cnt = 0;
            loop {
                sleep(duration);
                let mut state = cloned.lock().unwrap();
                if state.cnt >= 10 {
                    state.completed = true;
                }
                if state.cnt != prev_cnt {
                    prev_cnt = state.cnt;
                    if let Some(ref waker) = state.waker {
                        waker.clone().wake();
                    }
                }
            }
        });
        Self::Now(shared_state.clone())
    }
}

impl Future for Node {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match *self {
            Node::Now(ref shared_state) => {
                let mut shared_state = shared_state.lock().unwrap();
                shared_state.cnt += 1;
                println!("Polled {} time, completed {}", shared_state.cnt, shared_state.completed);
                if shared_state.completed {
                    Poll::Ready(())
                }
                else {
                    shared_state.waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
            Node::Never => Poll::Pending,
            Node::Pair(ref mut a, ref mut b) => {
                match Pin::new(a.as_mut()).poll(cx) {
                    Poll::Ready(()) => Poll::Ready(()),
                    Poll::Pending => Pin::new(b.as_mut()).poll(cx),
                }
            }
        }
    }
}


fn build(level: usize) -> Node {
    if level == 0 {
        unsafe {
            COUNTER += 1;
            if COUNTER == (1 << LEVEL) - 1{
                Node::new()
            }
            else {
                Node::Never
            }
        }
    }
    else {
        Node::Pair(Box::new(build(level - 1)), Box::new(build(level - 1)))
    }
}


fn main() {
    let fut = build(LEVEL);
    println!("build done.");
    task::block_on(fut);
}
