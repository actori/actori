use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use actori::prelude::*;
use tokio::time::{delay_for, Duration};

struct Die;

impl Message for Die {
    type Result = ();
}

struct MyActor(Arc<AtomicUsize>, Arc<AtomicUsize>, Arc<AtomicUsize>);

impl Actor for MyActor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Context<MyActor>) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}
impl actori::Supervised for MyActor {
    fn restarting(&mut self, _: &mut actori::Context<MyActor>) {
        self.1.fetch_add(1, Ordering::Relaxed);
    }
}

impl actori::Handler<Die> for MyActor {
    type Result = ();

    fn handle(&mut self, _: Die, ctx: &mut actori::Context<MyActor>) {
        self.2.fetch_add(1, Ordering::Relaxed);
        ctx.stop();
    }
}

#[test]
fn test_supervisor_restart() {
    let starts = Arc::new(AtomicUsize::new(0));
    let restarts = Arc::new(AtomicUsize::new(0));
    let messages = Arc::new(AtomicUsize::new(0));
    let starts2 = Arc::clone(&starts);
    let restarts2 = Arc::clone(&restarts);
    let messages2 = Arc::clone(&messages);

    let addr = Arc::new(Mutex::new(None));
    let addr2 = Arc::clone(&addr);

    System::run(move || {
        let addr =
            actori::Supervisor::start(move |_| MyActor(starts2, restarts2, messages2));
        addr.do_send(Die);
        addr.do_send(Die);
        *addr2.lock().unwrap() = Some(addr);

        actori::spawn(async move {
            delay_for(Duration::new(0, 100_000)).await;
            System::current().stop();
        });
    })
    .unwrap();

    assert_eq!(starts.load(Ordering::Relaxed), 3);
    assert_eq!(restarts.load(Ordering::Relaxed), 2);
    assert_eq!(messages.load(Ordering::Relaxed), 2);
}
