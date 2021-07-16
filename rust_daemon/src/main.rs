use actix_web::{App, HttpRequest, HttpResponse, HttpServer};
use riker::actors::*;
use std::sync::mpsc;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone, Debug)]
struct CommMsg {
    index: u32,
    name: String,
    data: MsgData,
}
#[derive(Clone, Debug)]
enum MsgData {
    Sta, //(mpsc::Sender),
    Req { code: u32, index: u32 },
    TmO,
    Rsp { ret: u8 },
}

// 模块1
struct MyActor {
    inst: [MyInstance; 10],
}

#[derive(Copy, Clone)]
struct MyInstance {
    index: u32,
    status: u8,
    ti: Uuid,
    // tx: mpsc::Sender,
}

// implement the Actor trait
impl Actor for MyActor {
    type Msg = CommMsg;

    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, _sender: Sender) {
        println!("Received: {:?}", msg);
        /*if "testreq" == msg.name {
            if let MsgData::Req { code: _, ti } = msg.data {
                // println!("time id {:?} {:?}", code, ti);
                ctx.cancel_schedule(ti);
                self.index = 1;
                println!("index {}", self.index);
            }
        }*/
        match msg.data {
            MsgData::Sta => {
                //(tx)
                let inst = &mut self.inst[0];

                // inst.tx = tx;

                let req = CommMsg {
                    index: 99,
                    name: String::from("testreq"),
                    data: MsgData::Req {
                        code: 4,
                        index: inst.index,
                    },
                };

                // 超时消息
                let timeout_req = CommMsg {
                    index: inst.index,
                    name: String::from("timeoutMsg"),
                    data: MsgData::TmO,
                };
                let ti =
                    ctx.schedule_once(Duration::from_millis(100), ctx.myself(), None, timeout_req);
                inst.ti = ti;

                let ma2 = ctx.select("/user/my-actor2").unwrap();
                ma2.try_tell(req, None);
                inst.status = 1;
            }
            MsgData::TmO => {
                println!("actor1 timeout");
                let inst = &mut self.inst[msg.index as usize];

                inst.status = 0;
            }
            MsgData::Rsp { ret } => {
                println!("actor1 recv ret {}", ret);
                let inst = &mut self.inst[msg.index as usize];

                // inst.tx.send(ret).unwrap();

                ctx.cancel_schedule(inst.ti);
                inst.status = 0;
            }
            _ => {}
        }
    }
}

// provide factory and props methods
impl MyActor {
    fn actor() -> Self {
        MyActor {
            inst: [MyInstance {
                index: 0,
                status: 0,
                ti: Uuid::nil(),
            }; 10],
        }
    }

    fn props() -> BoxActorProd<MyActor> {
        Props::new(MyActor::actor)
    }
}

// 模块2
struct MyActor2;
impl Actor for MyActor2 {
    type Msg = CommMsg;

    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, _sender: Sender) {
        if "testreq" == msg.name {
            if let MsgData::Req { code, index } = msg.data {
                println!("actor2 recv code {}", code);

                // std::thread::sleep(Duration::from_millis(200));

                let ma = ctx.select("/user/my-actor").unwrap();
                let rsp = CommMsg {
                    index: index,
                    name: String::from("testrsp"),
                    data: MsgData::Rsp { ret: 1 },
                };
                ma.try_tell(rsp, None);
            }
        }
    }
}
impl MyActor2 {
    fn actor() -> Self {
        MyActor2
    }

    fn props() -> BoxActorProd<MyActor2> {
        Props::new(MyActor2::actor)
    }
}

// start the system and create an actor
fn main() {
    let sys = ActorSystem::new().unwrap();

    let props = MyActor::props();
    let my_actor = sys.actor_of(props, "my-actor").unwrap();

    let props2 = MyActor2::props();
    sys.actor_of(props2, "my-actor2").unwrap();

    let req = CommMsg {
        index: 7,
        name: String::from("test_start"),
        data: MsgData::Sta,
    };
    my_actor.tell(req, None);

    /*HttpServer::new(|| App::new().route("/", move |r| r.h(IfHandler { ma: my_actor })))
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .unwrap();*/

    std::thread::sleep(Duration::from_millis(2000));
}

/*struct IfHandler {
    ma: actor_ref::ActorRef,
}
impl<S> Handler<S> for IfHandler {
    type Result = HttpResponse;

    /// Handle request
    fn handle(&mut self, req: HttpRequest<S>) -> Self::Result {
        let (tx, rx) = mpsc::channel();

        let req = CommMsg {
            index: 7,
            name: String::from("test_start"),
            data: MsgData::Sta(tx),
        };
        self.ma.tell(req, None);

        let ret = rx.recv().unwrap();
        HttpResponse::Ok().body(format!("ret code is {}\n", ret))
    }
}*/

/*struct RspRet {
    ret: u32,
}
impl Future for RspRet {
    type Item = u32;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.ret {
            0 => Ok(Async::NotReady),
            _ => Ok(Async::Ready(self.ret)),
        }
    }
}*/

/*
use std::any::Any;
use std::collections::HashMap;
use std::sync::mpsc;
use tokio::prelude::*;
use tokio::timer::Interval;

// let m = 3;

fn main() {
    let i = 1;
    let j = 2;
    println!("Hello, world! {}", i + j);

    let mut chan_map: HashMap<&str, MsgChan<Box<dyn CommMsg>>> = HashMap::new();
    // chan_map.insert("test", 3);
    // let t1 = chan_map.get("test").unwrap();
    // println!("map test {}", t1);

    let (tx, rx) = mpsc::channel();
    let test_chan = MsgChan { tx: tx, rx: rx };
    chan_map.insert("test", test_chan);

    let req = Reqmsg {
        name: String::from("testreq"),
        code: 1,
    };
    chan_map
        .get("test")
        .unwrap()
        .tx
        .send(Box::new(req))
        .unwrap();
    let ret = chan_map.get("test").unwrap().rx.recv().unwrap();
    println!("recv {}", ret.get_name());

    let m1 = String::from("test in");
    println!("test string {}", string_test1(m1));

    // println!("globel {}", m);
    let testreq: Box<dyn Any> = Box::new(Reqmsg {
        name: String::from("testreq"),
        code: 1,
    });
    let testany = testreq.downcast::<Reqmsg>().unwrap();
    println!("test req code {}", testany.code);
    // let commany = testreq.downcast_ref::<dyn CommMsg>().unwrap();
    // println!("test any msg {}", commany.get_name());
    // let testany = testreq.downcast_ref::<Reqmsg>().unwrap();
    // println!("test req code {}", testany.code);

    // 测试定时器
    /*let task = Interval::new(Instant::now(), Duration::from_millis(500))
        .take(2)
        .for_each(|instant| {
            println!("fire; instant={:?}", instant);
            Ok(())
        })
        .map_err(|e| panic!("interval errored; err={:?}", e));

    tokio::run(task);*/
}

struct MsgChan<T> {
    tx: mpsc::Sender<T>,
    rx: mpsc::Receiver<T>,
}

trait CommMsg {
    fn get_name(&self) -> &String;
}

struct Reqmsg {
    name: String,
    code: u8,
}
impl CommMsg for Reqmsg {
    fn get_name(&self) -> &String {
        &self.name
    }
}

fn string_test1(testin: String) -> String {
    testin
}
*/
