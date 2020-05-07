
mod dijkstra;
use dijkstra::*;

mod input_output;

use std::time::Duration;
use std::{io, thread, time};

use actix::io::SinkWrite;
use actix::*;
use actix_codec::Framed;
use awc::{
    error::WsProtocolError,
    ws::{Codec, Frame, Message},
    BoxedSocket, Client,
};
use bytes::Bytes;
use futures::stream::{SplitSink, StreamExt};

enum StatusClient {
    // enum for managing status of client during distributed calculation
    send_inquiry,
    receive_json,
    inquire_roots,
    receive_roots,
    inquire_calculation_success,
    send_calculation_success,
    confirm_calculation_success,
    error_status,
}

// Messages to communicate with server
pub const MSG_ROOT: &'static str = "root";
pub const MSG_SUCCESS: &'static str = "success";
pub const MSG_ERROR: &'static str = "error";
pub const MSG_JSON: &'static str = "json";
pub const MSG_RESULT: &'static str = "result";
pub const MSG_CONFIRM: &'static str = "confirm";

static mut json_input: &'static str = "";          // var to save JSON file
static mut roots_input: &'static str = "";         // var t save nodes
static mut result_string: &'static str = "";       // var to save result 

static mut status_client: StatusClient = StatusClient::send_inquiry; // var for currenr server status

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let sys = System::new("websocket-client");

    Arbiter::spawn(async {
        let (response, framed) = Client::new()
            .ws("http://127.0.0.1:8080/ws/")
            .connect()
            .await
            .map_err(|e| {
                println!("Error: {}", e);
            })
            .unwrap();

        println!("{:?}", response);
        let (sink, stream) = framed.split();
        let addr = ChatClient::create(|ctx| {
            ChatClient::add_stream(stream, ctx);
            ChatClient(SinkWrite::new(sink, ctx))
        });

        let ten_millis = time::Duration::from_millis(200);
        // thread sleep required in following to avoid endless loop

        // start console loop to register clients
        thread::spawn(move || loop {
            let mut cmd = String::new();
            unsafe {
                match status_client 
                // handle statuses of server
                {
                    StatusClient::send_inquiry => {
                        if io::stdin().read_line(&mut cmd).is_err() {
                            println!("{}",MSG_ERROR);
                          }

                          if cmd.trim()=="y" {
                            addr.do_send(ClientCommand(cmd));
                            }
                            
                         else 
                         {
                             println!("Better inputting 'y'");
                         }
                        }
                        
                    StatusClient::receive_json => (),
                    StatusClient::inquire_roots => {
                        addr.do_send(ClientCommand(MSG_ROOT.to_string()));    // sending root trigger to server
                        thread::sleep(ten_millis);
                    }
                    StatusClient::receive_roots =>(),
                    StatusClient::inquire_calculation_success => {
                        addr.do_send(ClientCommand(MSG_SUCCESS.to_string()));   // sending success trigger to server
                        thread::sleep(ten_millis);
                    }
                    StatusClient::send_calculation_success => {
                        addr.do_send(ClientCommand(MSG_RESULT.to_string()));     // sending result trigger to server
                        addr.do_send(ClientCommand(result_string.to_string()));    
                        // sending the result
                        thread::sleep(ten_millis);
                        println!(
                            "Your calculation was successfull transmitted. Thanks for your help!"
                        );
                    }
                    StatusClient::confirm_calculation_success => {}
                    StatusClient::error_status => {
                        addr.do_send(ClientCommand(MSG_ERROR.to_string()));
                    }
                }
            }
        });
    });
    sys.run().unwrap();
}

struct ChatClient(SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>);

#[derive(Message)]
#[rtype(result = "()")]
struct ClientCommand(String);

impl Actor for ChatClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopped(&mut self, _: &mut Context<Self>) {
        println!("Disconnected");

        // Stop application on disconnect
        System::current().stop();
    }
}

impl ChatClient {
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(1, 0), |act, ctx| {
            act.0.write(Message::Ping(Bytes::from_static(b""))).unwrap();
            act.hb(ctx);
        });
    }
}

/// Handle stdin commands
impl Handler<ClientCommand> for ChatClient {
    type Result = ();

    fn handle(&mut self, msg: ClientCommand, _ctx: &mut Context<Self>) {
            self.0.write(Message::Text(msg.0)).unwrap();
        
    }
}

/// Handle server websocket messages
impl StreamHandler<Result<Frame, WsProtocolError>> for ChatClient {
    fn handle(&mut self, msg: Result<Frame, WsProtocolError>, _: &mut Context<Self>) {
        if let Ok(Frame::Text(txt)) = msg {
            let servermsg: String = std::str::from_utf8(&txt).unwrap().to_string(); // converting byte to string for easier handling
            unsafe {
                 // go to corresponding modus according to server message
                match servermsg.as_str() {
                    MSG_JSON => status_client = StatusClient::receive_json,
                    MSG_ROOT => status_client = StatusClient::receive_roots,
                    MSG_SUCCESS => status_client = StatusClient::inquire_calculation_success,
                    MSG_ERROR => status_client = StatusClient::error_status,
                    MSG_CONFIRM => status_client = StatusClient::confirm_calculation_success,
                    _ => (),
                }


                match status_client {
                    StatusClient::send_inquiry => (),
                    // base status when making connection
                    StatusClient::receive_json => {
                        println!("Received json file from server!");

                        status_client = StatusClient::inquire_roots;
                    }
                    StatusClient::inquire_roots => {
                        json_input = string_to_static_str(servermsg);
                        // uses boxing for coping the value of servermsg to static variable for saving json input
                        println!(
                            "Inquireing roots from server... Just Type 'y'; any other input is also legit but 'y' prefered"
                        );
                      
                    }
                    StatusClient::receive_roots => {
                        println!("Received startnodes from server!");

                        status_client = StatusClient::inquire_calculation_success;
                    }
                    StatusClient::inquire_calculation_success => {
                        roots_input = string_to_static_str(servermsg);
                        // uses boxing for coping the value of servermsg to static variable for saving root input
                        println!("You are responsible for the start nodes: {}", roots_input);
                        println!("\nCalculating...");

                     
                        let st_nodes: Vec<&str> = roots_input.split(";").collect();

                        let json_string_tmp = json_input;

                        for i in 0..st_nodes.len() {
                            let dijkstra_temp_string =
                                dijkstra(st_nodes[i].parse::<i32>().unwrap(), json_string_tmp);  // doing djikstra calculation

                            if i == 0 {
                                result_string =
                                    string_to_static_str(format!("{}", dijkstra_temp_string));
                            } else {
                                result_string = string_to_static_str(format!(
                                    "{}:{}",
                                    result_string, dijkstra_temp_string
                                ));
                            }
                        }

                        status_client = StatusClient::send_calculation_success;
                    }
                    StatusClient::send_calculation_success => {
                        status_client = StatusClient::confirm_calculation_success;
                    }
                    StatusClient::confirm_calculation_success => {
                        status_client = StatusClient::confirm_calculation_success;
                    }
                    StatusClient::error_status => {
                        println!("{}", MSG_ERROR);
                    }
                }
            }
        }
    }

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("Connected to server");
        ask_client();
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        println!("Server disconnected");
        ctx.stop()
    }
}

fn ask_client() {
    println!("Would you like to join our distributed system for calculating shortest paths with djikstra algorithm? Type 'y' !");
}
fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

impl actix::io::WriteHandler<WsProtocolError> for ChatClient {}
