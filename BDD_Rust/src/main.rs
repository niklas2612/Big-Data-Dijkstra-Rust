//! Simple echo websocket server.
//! Open `http://localhost:8080/ws/index.html` in browser
//! or [python console client](https://github.com/actix/examples/blob/master/websocket/websocket-client.py)
//! could be used for testing.


mod input_output;
mod dijkstra;
mod user_output;
use crate::user_output::*;
use std::mem;
use std::io::{stdin, stdout, Write};

use input_output::*;
use crate::input_output::RootList;

#[macro_use]
extern crate lazy_static;

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::{thread, time, io};
use std::collections::*;


use actix::prelude::*;
use actix_files as fs;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(25);

enum StatusServer {                      // enum for managing status of server during distributed calculation
    send_acknowledge,
    send_json,
    send_roots,
    receive_calculation_success,
    receive_result,
    error_status,
}


pub const MSG_ACKNOWLEDGE: &'static str = "y\r\n";
pub const MSG_JSON: &'static str = "json";
pub const MSG_ROOT: &'static str = "root";
pub const MSG_SUCCESS: &'static str = "success";
pub const MSG_ERROR: &'static str = "error";
pub const MSG_ENTER: &'static str = "\r\n";
pub const MSG_CONFIRM: &'static str="confirm";

static mut AmountClients: i32 = 0;            // variabel to save amount of clients    
static mut AmountClientsReady: i32 = 0;            // variabel to save amount of clients ready for calculating
static mut JsonCounter: i32=0;
static mut RootCounter: i32=0;
static mut status_server: StatusServer=StatusServer::send_acknowledge;
static mut node_counter:i32=-1;



lazy_static!{
    static ref tables:Mutex<HashMap<i32, Table>>={
        let mut m = HashMap::new();
       
        Mutex::new(m)
    };
}


lazy_static!{
    static ref RootList_all:Mutex<Vec<String>>=Mutex::new(Vec::new());
}


/// do websocket handshake and start `MyWebSocket` actor
async fn ws_index(r: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    unsafe 
    {
        AmountClients+= 1;
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
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        // process websocket messages
       // println!("WS: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => {        // when receiving ping -> comm with client
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {           // when receiving pong -> com with Web IF
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) =>{
           
            unsafe
            {  
                 match text.as_str()
                        {

                        MSG_ENTER=> status_server=StatusServer::error_status,
                        MSG_ACKNOWLEDGE => status_server=StatusServer::send_acknowledge,
                        MSG_ROOT  => status_server=StatusServer::send_roots,
                        MSG_SUCCESS  => status_server=StatusServer::receive_calculation_success,
                         _ => status_server=StatusServer::receive_result,
                       }
            
                 match status_server    
            {
                
                  StatusServer::send_acknowledge => 
                  {
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
                                println!("{}",MSG_ERROR);
                            }
                        });
                    }
                    
                        child.join().unwrap();
                        println!("sending json file to client {} ...", JsonCounter);               
                        ctx.text(MSG_JSON);
                        let  json_input: String =read_input();       // reading json file                                           
                        ctx.text(json_input); 
                        if AmountClients==JsonCounter
                        {

                            let mut rootlist= RootList{roots:Vec::new()};
                            input_parse(&mut rootlist,read_input()).unwrap();      //TODO: making RootList Global! 
                            for i in 0..rootlist.roots.len() 
                            {
                               RootList_all.lock().unwrap().push(rootlist.roots[i].clone());
                            }                  
                        }
                                          // sending JSON files
                        JsonCounter-=1;
                        status_server=StatusServer::send_json;
                    }
                  
                  StatusServer::send_json => {println!("fuck");}
                  StatusServer::send_roots => 
                  {
                    
                       println!("sending respective roots to client {} ...", RootCounter);
                       let AmountRoots=RootList_all.lock().unwrap().len() as i32;
                       println!("AMount roots: {}",AmountRoots);
                       let RootsPerClient= AmountRoots /AmountClients ;
                       /*let start=RootList_all.lock().unwrap().len() -(RootsPerClient*RootCounter as usize);
                       let end =(RootList_all.lock().unwrap().len() -(RootsPerClient*(RootCounter as usize -1)))-1;*/

                       if node_counter==-1
                       {
                           node_counter=AmountRoots;
                       }

                       let start=AmountRoots-node_counter;
                       node_counter= node_counter-RootsPerClient;
                       let mut end= AmountRoots-node_counter;

                        let tmp= AmountRoots % AmountClients;
                        if (tmp -((AmountClients-RootCounter)))>0
                        {
                            println!("tmp");
                            node_counter-=1;
                            end +=1;
                        }
                        println!("Amount{}",tmp -((AmountClients-RootCounter)+1));
                         
                      
                      
                       if node_counter==0
                       {
                           node_counter=-1;
                       }

                       println!(" NODECOUNTER {} AMOUNTROOT {}  ROOTPC {}  START {}   END  {}",node_counter, AmountRoots, RootsPerClient, start,end);

                       let vartmp=RootList_all.lock().unwrap().to_vec();
                       let vartmp2= splitVec(start as usize,(end-1) as usize, vartmp);
                    
                       ctx.text(MSG_ROOT);
                       ctx.text(vartmp2.as_str()); 
                       RootCounter-=1;
                       status_server=StatusServer::receive_calculation_success;
                       
                 }
                  StatusServer::receive_calculation_success => 
                  {
                      println!("all data were sent to client {} calculation in progress!", RootCounter+1);
                      status_server=StatusServer::receive_result;
                  }
                  StatusServer::receive_result => 
                  {
                      println!("");
                      println!("Calculation is finished, your result is: {}",text);
                      // TODO: Berechnetes Ergebnis is Leo Abi seine Datenstrktur einpflegen

                        

                      let tmp_table_strings: Vec<&str> = text.split(":").collect();

                      

                      for m in 0..tmp_table_strings.len(){

                        let (node_id, table) = create_table_from_string(tmp_table_strings[m]);

                        let tmp_mutexguard = tables.lock().unwrap().insert(node_id, table);
                        drop(tmp_mutexguard);

                        
                      }


                      JsonCounter+=1;

                    
                   
                      ctx.text(MSG_CONFIRM);

                      println!("jsoncounter:{}, rootcounter{}", JsonCounter, RootCounter);

                      if JsonCounter==AmountClients
                      {
                        
                          //user_input();
                      }
                   }
                  StatusServer::error_status=> 
                  {
                      println!("{}", MSG_ERROR);
                      ctx.text(MSG_ERROR);
                  }
            }
        }
              
       } , 
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
   
    let AmountWorkers=set_workers();

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
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn acknowledgeclient()
{
    unsafe {
    JsonCounter=AmountClients;
    RootCounter=AmountClients;
    AmountClientsReady+=1;       
    println!("");
    println!("total amount of clients is : {}", AmountClients);
    println!("amount of clients ready for calculation is : {}", AmountClientsReady);
    if AmountClientsReady==AmountClients
    {
        println!("-------------------------------------------------------------------------");
        println!("All clients have acknowledged and would like to join; you can start the calculation right now!");
        println!("NOTE: PRESS ENTER AS OFTEN AS MANY CLIENTS YOU HAVE CONNECTED!");
        println!("");
    }

    else 
    {
        println!("");
        println!("Clients still missing : {}", AmountClients-AmountClientsReady);
        println!("Request your clients to confirm!");
        println!("Waiting for all clients to acknowledge calculation partizipation...");
        println!("");
        println!("");
    }
   
          }
}

fn splitVec (start:usize, end:usize, vec: Vec<String>) -> String
{
    let mut returnString:String=String::from("");
    for i in start..(end+1) 
    {
        println!("INDEX {} ", i);
        if i == start{
            returnString = vec[i].clone();
        }
        else{
            returnString=format!("{};{}", returnString, vec[i].clone());
        }
       
        println!("{}", returnString);
    }
    
    return returnString;
}

fn set_workers() -> i32
{
    println!("Welcome to Big Data Djikstra! This is a distributed system for calculating the shortest path using actix and websockets for communication");
    println!("");
    println!("To optimize your performance please give the max amounts of clients you want to join:");

    let mut input = String::new();

    io::stdin().read_line(&mut input).unwrap();
    let n: i32 = input.trim().parse().unwrap();

    println!("");
    println!("Setting up the server for you...");

    let ten_millis = time::Duration::from_millis(2000);
    thread::sleep(ten_millis);

    return n;
}

fn user_input(){

    let mut exit = String::from("n");

    while !(exit.trim() == "y") {
        
        let mut start = String::new();
        let mut end = String::new();

        print!("Startknoten eingeben:\n");
        let _=stdout().flush();
        stdin().read_line(&mut start).expect("Did not enter a correct string\n");
        if let Some('\n')=start.chars().next_back() {
            start.pop();
        }
        if let Some('\r')=start.chars().next_back() {
            start.pop();
        }
        let start = start.parse::<i32>().unwrap();

        
        print!("Zielknoten eingeben:\n");
        let _=stdout().flush();
        stdin().read_line(&mut end).expect("Did not enter a correct string\n");
        if let Some('\n')=end.chars().next_back() {
            end.pop();
        }
        if let Some('\r')=end.chars().next_back() {
            end.pop();
        }   
        let end = end.parse::<u16>().unwrap();

        let path = tables.lock().unwrap()[&start].get_path(end);

        

        



        println!("{}", path);

        print!("Beenden?(any key for no or y for yes):\n");
        let _=stdout().flush();
        stdin().read_line(&mut exit).expect("Did not enter a correct string\n");      
        

        println!("{}", exit);

    }

}