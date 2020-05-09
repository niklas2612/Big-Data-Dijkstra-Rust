// here should be a great project header, created by L.N

// annotation: the terms "roots" and "startnodes" are describing the same (maybe L.N adapts to unique term if he wants a good grade)
mod dijkstra;
mod input_output;
mod user_output;

use user_output::*;
use input_output::*;

use std::io::{stdin, stdout, Write};
use std::collections::*;
use std::sync::{Mutex};
use std::time::{Duration, Instant};
use std::{io, thread};
use actix::prelude::*;
use actix_files as fs;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;


#[macro_use]
extern crate lazy_static;    // used for static Vectors and Hash Maps




/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(25);

enum StatusServer {
    // enum for managing status of server during distributed calculation
    SendAcknowledge,
    SendJson,
    SendRoots,
    RecieveCalculationSuccess,
    RecieveResult,
    ErrorStatus,
}

// Messages to communicate with clients
pub const MSG_ACKNOWLEDGE: &'static str = "y\r\n";
pub const MSG_JSON: &'static str = "json";
pub const MSG_ROOT: &'static str = "root";
pub const MSG_SUCCESS: &'static str = "success";
pub const MSG_ERROR: &'static str = "error";
pub const MSG_CONFIRM: &'static str = "confirm";

static mut AMOUNT_CLIENTS: i32 = 0; // variabel to save amount of clients
static mut AMOUNT_CLIENTSREADY: i32 = 0; // variabel to save amount of clients ready for calculating
static mut JSON_COUNTER: i32 = 0;
static mut ROOT_COUNTER: i32 = 0;
static mut STATUS_SERVER: StatusServer = StatusServer::SendAcknowledge; // variable for the current server status
static mut NODE_COUNTER: i32 = -1;

lazy_static! {
    static ref TABLES: Mutex<HashMap<i32, Table>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

lazy_static! {
    static ref ROOTLIST_ALL: Mutex<Vec<String>> = Mutex::new(Vec::new());  
    // List for saving all start nodes before sending them to clients     
}

/// do websocket handshake and start `MyWebSocket` actor
async fn ws_index(r: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    unsafe {
        AMOUNT_CLIENTS += 1;
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
                        MSG_ACKNOWLEDGE => STATUS_SERVER = StatusServer::SendAcknowledge,
                        MSG_ROOT => STATUS_SERVER = StatusServer::SendRoots,
                        MSG_SUCCESS => STATUS_SERVER = StatusServer::RecieveCalculationSuccess,
                        MSG_ERROR=> STATUS_SERVER=StatusServer::ErrorStatus,
                        _=> STATUS_SERVER = StatusServer::RecieveResult,
                    }

                    match STATUS_SERVER {
                        StatusServer::SendAcknowledge => {
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
                            println!("sending json file to client {} ...", JSON_COUNTER+1);
                            ctx.text(MSG_JSON);                   // sending JSON trigger to client
                            let json_input: String = read_input(); // reading json file
                            ctx.text(json_input);                 // sending JSON file to client
                            if AMOUNT_CLIENTS == JSON_COUNTER {
                                let mut rootlist = RootList { roots: Vec::new() };
                                input_parse(&mut rootlist, read_input()).unwrap();
                                for i in 0..rootlist.roots.len() {
                                    ROOTLIST_ALL.lock().unwrap().push(rootlist.roots[i].clone());       // make a List with all nodes out of JSON input
                                }
                            }
                
                            JSON_COUNTER -= 1;
                            STATUS_SERVER = StatusServer::SendJson;
                        }

                        StatusServer::SendJson => (),
                        // sending start nodes to client
                        StatusServer::SendRoots => {
                            println!("");
                            println!(
                                "Sending respective startnodes to client {} ...",
                                ROOT_COUNTER
                            );
                            let amount_roots = ROOTLIST_ALL.lock().unwrap().len() as i32;
                            let roots_per_client = amount_roots / AMOUNT_CLIENTS;
                           
                            if NODE_COUNTER == -1 {
                                NODE_COUNTER = amount_roots;
                            }

                            let start = amount_roots - NODE_COUNTER;
                            NODE_COUNTER = NODE_COUNTER - roots_per_client;
                            let mut end = amount_roots - NODE_COUNTER;

                            let tmp = amount_roots % AMOUNT_CLIENTS;
                            if (tmp - (AMOUNT_CLIENTS - ROOT_COUNTER)) > 0 {
                                NODE_COUNTER -= 1;
                                end += 1;
                            }

                            if NODE_COUNTER == 0 {
                                NODE_COUNTER = -1;
                            }

                            let vartmp = ROOTLIST_ALL.lock().unwrap().to_vec();
                            let vartmp2 = split_vec(start as usize, (end - 1) as usize, vartmp);

                            ctx.text(MSG_ROOT);     // sending root trigger for client
                            ctx.text(vartmp2.as_str()); // sending startnodes
                            ROOT_COUNTER -= 1;
                            STATUS_SERVER = StatusServer::RecieveCalculationSuccess;
                        }
                        StatusServer::RecieveCalculationSuccess => {
                            println!(
                                "All data were sent to client {} calculation in progress!",
                                ROOT_COUNTER + 1
                            );
                            STATUS_SERVER = StatusServer::RecieveResult;
                        }
                        StatusServer::RecieveResult => {
                            println!(
                                "Client {} has sent his results to this server!",
                                ROOT_COUNTER+1
                            );
                            println!("");

                            let tmp_table_strings: Vec<&str> = text.split(":").collect(); // decode the table string received from client

                            for m in 0..tmp_table_strings.len() {
                                let (node_id, table) =
                                    create_table_from_string(tmp_table_strings[m]);

                                let tmp_mutexguard = TABLES.lock().unwrap().insert(node_id, table);      
                                drop(tmp_mutexguard);
                            }

                            JSON_COUNTER += 1;

                            ctx.text(MSG_CONFIRM);

                            if JSON_COUNTER == AMOUNT_CLIENTS {
                                // all results combined; user can make a request now
                                user_input(); 
                            }
                        }
                        StatusServer::ErrorStatus => {
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
    let connection_string= set_connection();
    let amount_workers = set_workers();         // getting amount of workers

    
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
    .workers(amount_workers as usize)
   //.bind(connection_string.to_socket_addrs().unwrap())?   //does not work, maybe Chris Mader will fix this
    .bind("127.0.0.1:8080")? 
    .run()
    .await
}

fn acknowledgeclient() {
    unsafe {
        JSON_COUNTER = AMOUNT_CLIENTS;
        ROOT_COUNTER = AMOUNT_CLIENTS;
        AMOUNT_CLIENTSREADY += 1;
        println!("");
        println!("total amount of clients is : {}", AMOUNT_CLIENTS);
        println!(
            "amount of clients ready for calculation is : {}",
            AMOUNT_CLIENTSREADY
        );
        if AMOUNT_CLIENTSREADY == AMOUNT_CLIENTS {
            println!("-------------------------------------------------------------------------");
            println!("All clients have acknowledged and would like to join; you can start the calculation right now!");
            println!("Press enter or any other input as often as many clients you hav in the distributed system..");
            println!("");
        } else {
            println!("");
            println!(
                "Clients still missing : {}",
                AMOUNT_CLIENTS - AMOUNT_CLIENTSREADY
            );
            println!("Request your clients to confirm!");
            println!("Waiting for all clients to acknowledge calculation partizipation...");
            println!("");
            println!("");
        }
    }
}

//function to build a string out of elements of a vec<string> with given start and end index
fn split_vec(start: usize, end: usize, vec: Vec<String>) -> String {
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

            let path = TABLES.lock().unwrap()[&start].get_path(end);

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
