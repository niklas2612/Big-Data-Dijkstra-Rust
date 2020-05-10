
mod dijkstra;
use dijkstra::*;

mod input_output;


use std::time::Duration;
use std::{io, thread, time};
use std::process;


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
    SendInquiry,
    RecieveJson,
    InquireRoots,
    RecieveRoots,
    InquireCalculationSuccess,
    SendCalculationSuccess,
    ConfirmCalculationSuccess,
    ErrorStatus,
}

// Messages to communicate with server
pub const MSG_ROOT: &'static str = "root";
pub const MSG_SUCCESS: &'static str = "success";
pub const MSG_ERROR: &'static str = "error";
pub const MSG_JSON: &'static str = "json";
pub const MSG_CONFIRM: &'static str = "confirm";

 // var to save JSON file as string
static mut JSON_INPUT: &'static str = "";       

//variable to store startnodes as decoded string
static mut STARTNODES_INPUT: &'static str = "";

//var to save resultstring
static mut RESULT_STRING: &'static str = "";       

// var for currenr server status
static mut STATUS_CLIENT: StatusClient = StatusClient::SendInquiry; 

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let mut connection_string = set_connection();
    connection_string = format!("{}{}{}","http://",connection_string.trim(), "/ws/");

    let sys = System::new("websocket-client");

    Arbiter::spawn(async {
        let (response, framed) = Client::new()
            .ws(connection_string)
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

        let delay = time::Duration::from_millis(200);
        // thread sleep required in following to avoid endless loop

        // start console loop to register clients
        thread::spawn(move || loop {
            let mut cmd = String::new();
            unsafe {
                match STATUS_CLIENT 
                // handle statuses of server
                {
                    StatusClient::SendInquiry => {
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
                        
                    StatusClient::RecieveJson => (),

                    // sending root trigger to server
                    StatusClient::InquireRoots => {
                        addr.do_send(ClientCommand(MSG_ROOT.to_string()));    
                        thread::sleep(delay);
                    }
                    StatusClient::RecieveRoots =>(),

                    // sending success trigger to server
                    StatusClient::InquireCalculationSuccess => {
                        addr.do_send(ClientCommand(MSG_SUCCESS.to_string()));   
                        thread::sleep(delay);
                    }

                    // sending result to server
                    StatusClient::SendCalculationSuccess => { 
                        addr.do_send(ClientCommand(RESULT_STRING.to_string()));    
                        // sending the result string
                        thread::sleep(delay);
                        println!(
                            "Your calculation was successfull transmitted. Thanks for your help!"
                        );
                    }
                    StatusClient::ConfirmCalculationSuccess => {process::exit(0)}
                    StatusClient::ErrorStatus => {
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
                    MSG_JSON => STATUS_CLIENT = StatusClient::RecieveJson,
                    MSG_ROOT => STATUS_CLIENT = StatusClient::RecieveRoots,
                    MSG_SUCCESS => STATUS_CLIENT = StatusClient::InquireCalculationSuccess,
                    MSG_ERROR => STATUS_CLIENT = StatusClient::ErrorStatus,
                    MSG_CONFIRM => STATUS_CLIENT = StatusClient::ConfirmCalculationSuccess,
                    _ => (),
                }


                match STATUS_CLIENT {
                    StatusClient::SendInquiry => (),
                    // base status when making connection
                    StatusClient::RecieveJson => {
                        println!("Received json file from server!");

                        STATUS_CLIENT = StatusClient::InquireRoots;
                    }
                    StatusClient::InquireRoots => {
                        JSON_INPUT = string_to_static_str(servermsg);
                        // uses boxing for coping the value of servermsg to static variable for saving json input
                        println!(
                            "Inquireing roots from server... Just Type 'y'; any other input is also legit but 'y' prefered"
                        );
                      
                    }
                    StatusClient::RecieveRoots => {
                        println!("Received startnodes from server!");

                        STATUS_CLIENT = StatusClient::InquireCalculationSuccess;
                    }
                    StatusClient::InquireCalculationSuccess => {
                        STARTNODES_INPUT = string_to_static_str(servermsg);
                        // uses boxing for coping the value of servermsg to static variable for saving root input
                        println!("You are responsible for the start nodes: {}", STARTNODES_INPUT);
                        println!("\nCalculating...");

                     
                        let st_nodes: Vec<&str> = STARTNODES_INPUT.split(";").collect();

                        let json_string_tmp = JSON_INPUT;

                        for i in 0..st_nodes.len() {
                            let dijkstra_temp_string =
                                dijkstra(st_nodes[i].parse::<i32>().unwrap(), json_string_tmp);  // doing djikstra calculation

                            if i == 0 {
                                RESULT_STRING =
                                    string_to_static_str(format!("{}", dijkstra_temp_string));
                            } else {
                                RESULT_STRING = string_to_static_str(format!(
                                    "{}:{}",
                                    RESULT_STRING, dijkstra_temp_string
                                ));
                            }
                        }

                        STATUS_CLIENT = StatusClient::SendCalculationSuccess;
                    }
                    StatusClient::SendCalculationSuccess => {
                        STATUS_CLIENT = StatusClient::ConfirmCalculationSuccess;
                    }
                    StatusClient::ConfirmCalculationSuccess => {
                        STATUS_CLIENT = StatusClient::ConfirmCalculationSuccess;
                    }
                    StatusClient::ErrorStatus => {
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

fn set_connection() -> String
{
    println!("Please type IP-Adress and port of your server you want to connect with! (e.g.: 127.0.0.1:8080)");
    
    let mut input = String::new();

    io::stdin().read_line(&mut input).unwrap();
    return input;

}

impl actix::io::WriteHandler<WsProtocolError> for ChatClient {}
