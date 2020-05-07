// here should be a great project header, created by L.N

// annotation: the terms "roots" and "startnodes" are describing the same (maybe L.N adapts to unique term if he wants a good grade)
mod dijkstra;
mod input_output;
mod user_output;
use crate::user_output::*;
use std::io::{stdin, stdout, Write};

use crate::input_output::RootList;
use input_output::*;

#[macro_use]
extern crate lazy_static;    // used for static Vectors and Hash Maps

use std::collections::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{io, thread, time};

use actix::prelude::*;
use actix_files as fs;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(25);

enum StatusServer {
    // enum for managing status of server during distributed calculation
    send_acknowledge,
    send_json,
    send_roots,
    receive_calculation_success,
    receive_result,
    error_status,
}

// Messages to communicate with clients
pub const MSG_ACKNOWLEDGE: &'static str = "y\r\n";
pub const MSG_JSON: &'static str = "json";
pub const MSG_ROOT: &'static str = "root";
pub const MSG_SUCCESS: &'static str = "success";
pub const MSG_ERROR: &'static str = "error";
pub const MSG_CONFIRM: &'static str = "confirm";

static mut AmountClients: i32 = 0; // variabel to save amount of clients
static mut AmountClientsReady: i32 = 0; // variabel to save amount of clients ready for calculating
static mut JsonCounter: i32 = 0;
static mut RootCounter: i32 = 0;
static mut status_server: StatusServer = StatusServer::send_acknowledge; // variable for the current server status
static mut node_counter: i32 = -1;

lazy_static! {
    static ref tables: Mutex<HashMap<i32, Table>> = {
        let mut m = HashMap::new();
        Mutex::new(m)
    };
}

lazy_static! {
    static ref RootList_all: Mutex<Vec<String>> = Mutex::new(Vec::new());  
    // List for saving all start nodes before sending them to clients     
}

/// do websocket handshake and start `MyWebSocket` actor
async fn ws_index(r: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    unsafe {
        AmountClients += 1;
    }
    println!("");
    println!("new client connected to the server:");
    println!("{:?}", r);
    let res = ws::start(MyWebSocket::new(), &r, stream);
    println!("{:?}", res);
    res
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
struct MyWebSocket {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages

        match msg {
            Ok(ws::Message::Ping(msg)) => {
                // when receiving ping -> comm with client
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                // when receiving pong -> com with Web IF
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                unsafe {
                    // go to corresponding modus according to client message
                    match text.as_str() {
                        MSG_ACKNOWLEDGE => status_server = StatusServer::send_acknowledge,
                        MSG_ROOT => status_server = StatusServer::send_roots,
                        MSG_SUCCESS => status_server = StatusServer::receive_calculation_success,
                        MSG_ERROR=> status_server=StatusServer::error_status,
                        _=> status_server = StatusServer::receive_result,
                    }

                    match status_server {
                        StatusServer::send_acknowledge => {
                            acknowledgeclient();
                            // new client acknowledged

                            struct Threading(String);
                            let thread = Threading("".to_string());
                            let mutex = std::sync::Mutex::new(thread);
                            let arc = std::sync::Arc::new(mutex);
                            let child;
                            {
                                let arc = arc.clone();
                                child = std::thread::spawn(move || {
                                    let mut guard = arc.lock().unwrap();
                                    if io::stdin().read_line(&mut guard.0).is_err() {
                                        println!("{}", MSG_ERROR);
                                    }
                                });
                            }

                            child.join().unwrap();
                            println!("sending json file to client {} ...", JsonCounter);
                            ctx.text(MSG_JSON);                   // sending JSON trigger to client
                            let json_input: String = read_input(); // reading json file
                            ctx.text(json_input);                 // sending JSON file to client
                            if AmountClients == JsonCounter {
                                let mut rootlist = RootList { roots: Vec::new() };
                                input_parse(&mut rootlist, read_input()).unwrap();
                                for i in 0..rootlist.roots.len() {
                                    RootList_all.lock().unwrap().push(rootlist.roots[i].clone());       // make a List with all nodes out of JSON input
                                }
                            }
                
                            JsonCounter -= 1;
                            status_server = StatusServer::send_json;
                        }

                        StatusServer::send_json => (),
                        // sending start nodes to client
                        StatusServer::send_roots => {
                            println!("");
                            println!(
                                "Sending respective startnodes to client {} ...",
                                RootCounter
                            );
                            let AmountRoots = RootList_all.lock().unwrap().len() as i32;
                            let RootsPerClient = AmountRoots / AmountClients;
                           
                            if node_counter == -1 {
                                node_counter = AmountRoots;
                            }

                            let start = AmountRoots - node_counter;
                            node_counter = node_counter - RootsPerClient;
                            let mut end = AmountRoots - node_counter;

                            let tmp = AmountRoots % AmountClients;
                            if (tmp - (AmountClients - RootCounter)) > 0 {
                                node_counter -= 1;
                                end += 1;
                            }

                            if node_counter == 0 {
                                node_counter = -1;
                            }

                            let vartmp = RootList_all.lock().unwrap().to_vec();
                            let vartmp2 = splitVec(start as usize, (end - 1) as usize, vartmp);

                            ctx.text(MSG_ROOT);     // sending root trigger for client
                            ctx.text(vartmp2.as_str()); // sending startnodes
                            RootCounter -= 1;
                            status_server = StatusServer::receive_calculation_success;
                        }
                        StatusServer::receive_calculation_success => {
                            println!(
                                "All data were sent to client {} calculation in progress!",
                                RootCounter + 1
                            );
                            status_server = StatusServer::receive_result;
                        }
                        StatusServer::receive_result => {
                            println!(
                                "Client {} has sent his results to this server!",
                                RootCounter+1
                            );
                            println!("");

                            let tmp_table_strings: Vec<&str> = text.split(":").collect(); // decode the table string received from client

                            for m in 0..tmp_table_strings.len() {
                                let (node_id, table) =
                                    create_table_from_string(tmp_table_strings[m]);

                                let tmp_mutexguard = tables.lock().unwrap().insert(node_id, table);      
                                drop(tmp_mutexguard);
                            }

                            JsonCounter += 1;

                            ctx.text(MSG_CONFIRM);

                            if JsonCounter == AmountClients {
                                // all results combined; user can make a request now
                                user_input(); 
                            }
                        }
                        StatusServer::error_status => {
                            println!("{}", MSG_ERROR);
                            ctx.text(MSG_ERROR);
                        }
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(_)) => {
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl MyWebSocket {
    fn new() -> Self {
        Self { hb: Instant::now() }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let ConnectionString= set_connection();
    let AmountWorkers = set_workers();         // getting amount of workers

    
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // websocket route
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            // static files
            .service(fs::Files::new("/", "static/").index_file("index.html"))
    })
    // start http server on 127.0.0.1:8080
    .workers(AmountWorkers as usize)
   // .bind(ConnectionString)?   does not work, maybe Chris Mader will fix this
    .bind("127.0.0.1:8080")? 
    .run()
    .await
}

fn acknowledgeclient() {
    unsafe {
        JsonCounter = AmountClients;
        RootCounter = AmountClients;
        AmountClientsReady += 1;
        println!("");
        println!("total amount of clients is : {}", AmountClients);
        println!(
            "amount of clients ready for calculation is : {}",
            AmountClientsReady
        );
        if AmountClientsReady == AmountClients {
            println!("-------------------------------------------------------------------------");
            println!("All clients have acknowledged and would like to join; you can start the calculation right now!");
            println!("Press enter or any other input as often as many clients you hav in the distributed system..");
            println!("");
        } else {
            println!("");
            println!(
                "Clients still missing : {}",
                AmountClients - AmountClientsReady
            );
            println!("Request your clients to confirm!");
            println!("Waiting for all clients to acknowledge calculation partizipation...");
            println!("");
            println!("");
        }
    }
}

//function to build a string out of elements of a vec<string> with given start and end index
fn splitVec(start: usize, end: usize, vec: Vec<String>) -> String {
    let mut return_string: String = String::from("");
    for i in start..(end + 1) {
        if i == start {
            return_string = vec[i].clone();
        } else {
            return_string = format!("{};{}", return_string, vec[i].clone());
        }
    }
    return return_string;
}

fn set_workers() -> i32 {
   
    println!("");
    println!(
        "Please give the max amounts of workers you want to use:"
    );

    let mut input = String::new();

    io::stdin().read_line(&mut input).unwrap();
    let n: i32 = input.trim().parse().unwrap();

    println!("");
    println!("Setting up the server for you...");

    return n;
}

fn set_connection() -> String
{
    println!("Welcome to Big Data Djikstra! This is a distributed system for calculating the shortest path using actix and websockets for communication");
    println!("Please enter the IP adress and port to host your webserver.");
    println!("Standard is 127.0.0.1:8080 (localhost), optionally you can enter your IPv4 to host this service for multiple devices. Therefore use ipconfig on your cmd to look up your address");

    let mut input = String::new();

    io::stdin().read_line(&mut input).unwrap();
    return input;

}
fn user_input() {
    thread::spawn(move || {
        let mut exit = String::from("n");

        while !(exit.trim() == "y") {
            let mut start = String::new();
            let mut end = String::new();

            print!("Startknoten eingeben:\n");
            let _ = stdout().flush();
            stdin()
                .read_line(&mut start)
                .expect("Did not enter a correct string\n");
            if let Some('\n') = start.chars().next_back() {
                start.pop();
            }
            if let Some('\r') = start.chars().next_back() {
                start.pop();
            }
            let start = start.parse::<i32>().unwrap();

            print!("Zielknoten eingeben:\n");
            let _ = stdout().flush();
            stdin()
                .read_line(&mut end)
                .expect("Did not enter a correct string\n");
            if let Some('\n') = end.chars().next_back() {
                end.pop();
            }
            if let Some('\r') = end.chars().next_back() {
                end.pop();
            }
            let end = end.parse::<u16>().unwrap();

            let path = tables.lock().unwrap()[&start].get_path(end);

            println!("{}", path);

            print!("Beenden?(any key for no or y for yes):\n");
            let _ = stdout().flush();
            stdin()
                .read_line(&mut exit)
                .expect("Did not enter a correct string\n");

            println!("{}", exit);
        }
    });
}
