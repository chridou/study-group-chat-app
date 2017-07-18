#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate ws;

use std::sync::Mutex;
use std::thread;
use std::path::{PathBuf, Path};
use rocket::*;
use rocket::response::NamedFile;
use ws::util::Token;

mod broker;
use broker::{Broker,BrokerHandle};

struct Handler {
    id: Token,
    broker: BrokerHandle,
}

impl ws::Handler for Handler {
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.broker.broadcast(self.id, msg.to_string());
        Ok(())
    }

    fn on_close(&mut self, _code: ws::CloseCode, _reason: &str) {
        let _ = self.broker.disconnect(self.id);
    }
}

struct ChatServer {
    broker: Mutex<BrokerHandle>,
}

impl ChatServer {
    pub fn launch() -> Self {
        let broker = Broker::spawn();

        Self::spawn_websocket(broker.clone());

        ChatServer {
            broker: Mutex::new(broker),
        }
    }

    fn spawn_websocket(broker: BrokerHandle) {
        thread::spawn(move || {
            ws::listen("localhost:8001", |sender| {
                let id = sender.token();

                let _ = broker.connect(sender);

                Handler {
                    id: id,
                    broker: broker.clone(),
                }
            }).unwrap()
        });
    }
}


#[get("/")]
fn home(state: State<ChatServer>) -> NamedFile {
    NamedFile::open("assets/index.html").unwrap()
}

#[get("/test")]
fn test(state: State<ChatServer>) -> String {
    unimplemented!()
}


#[get("/assets/<file..>")]
fn assets(file: PathBuf) -> Option<NamedFile> {
    let path = Path::new("assets").join(file);
    NamedFile::open(path).ok()
}

fn main() {
    println!("Hello, world!");

    let server = ChatServer::launch();


    let _ = rocket::ignite()
        .manage(server)
        .mount("/", routes![home, assets, test])
        .launch();
}
