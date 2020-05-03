//! Simple websocket client.
//! mod input_output;

mod dijkstra;
use dijkstra::*;
use std::time::Duration;
use std::{io, thread,time};

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

enum StatusClient {                  // enum for managing status of client during distributed calculation
    send_inquiry,
    receive_json,
    inquire_roots,
    receive_roots,
    inquire_calculation_success,
    send_calculation_success,
    confirm_calculation_success,
    error_status,
}

pub const MSG_ROOT: &'static str = "root";
pub const MSG_SUCCESS: &'static str = "success";
pub const MSG_ERROR: &'static str = "error";
pub const MSG_JSON: &'static str = "json";
pub const MSG_RESULT: &'static str = "result";
pub const MSG_CONFIRM: &'static str="confirm";

static mut json_input : & 'static str ="";
static mut roots_input:&'static str="";
static mut result_string:&'static str="";

static mut status_client: StatusClient = StatusClient::send_inquiry;

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

        // start console loop
        thread::spawn(move || loop {
            let mut cmd = String::new();
            unsafe {
                match status_client
                {
                    StatusClient::send_inquiry => 
                      {
                          if io::stdin().read_line(&mut cmd).is_err() {
                            println!("{}",MSG_ERROR);
                            return;                                      // change algotithm to have a automated answer
                        }
                        addr.do_send(ClientCommand(cmd));
                      }
                    StatusClient::receive_json => (),
                    StatusClient::inquire_roots=>
                    {
                      
                      addr.do_send(ClientCommand(MSG_ROOT.to_string()));
                      thread::sleep(ten_millis);
                    }
                    StatusClient::receive_roots=> (),
                    StatusClient::inquire_calculation_success =>
                    {
                     
                     addr.do_send(ClientCommand(MSG_SUCCESS.to_string()));
                     thread::sleep(ten_millis);
                     
                    } 
                    StatusClient::send_calculation_success=>
                    {
                        
                        addr.do_send(ClientCommand(MSG_RESULT.to_string()));
                        addr.do_send(ClientCommand(json_input.to_string()));  
                                         // Here is the result
                        thread::sleep(ten_millis);
                        println!("Your calculation was successfull transmitted. Thanks for your help!");
                    }
                    StatusClient::confirm_calculation_success => (),
                    StatusClient::error_status => {println!("{}",MSG_ERROR);}
                      
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

            // client should also check for a timeout here, similar to the
            // server code
        });
    }
}

/// Handle stdin commands
impl Handler<ClientCommand> for ChatClient {
    type Result = ();

    fn handle(&mut self, msg: ClientCommand, _ctx: &mut Context<Self>) {
        if msg.0.trim() =="y"  // only check from commandline input 
        {
            self.0.write(Message::Text(msg.0)).unwrap();
            println!("");
        }         
        else {self.0.write(Message::Text(msg.0)).unwrap();}        // Needs work, catching other input then y
       
        }
    
    }

/// Handle server websocket messages
impl StreamHandler<Result<Frame, WsProtocolError>> for ChatClient {
    fn handle(&mut self, msg: Result<Frame, WsProtocolError>, _: &mut Context<Self>) {
        if let Ok(Frame::Text(txt)) = msg {
            let servermsg:String= std::str::from_utf8(&txt).unwrap().to_string(); // converting byte to string for easier handling
     unsafe {
            match servermsg.as_str()
            {
           
           MSG_JSON => status_client=StatusClient::receive_json,
           MSG_ROOT => status_client=StatusClient::receive_roots,
           MSG_SUCCESS  => status_client=StatusClient::inquire_calculation_success,
           MSG_ERROR => status_client=StatusClient::error_status,
           MSG_CONFIRM=> status_client=StatusClient::confirm_calculation_success,
             _=> (),
           }

         //   println!("{}", servermsg);          
                                                            
                match status_client 
                {
                      StatusClient::send_inquiry => (),  
                      // base status when making connection                 
                      StatusClient::receive_json => 
                      {
                          
                          println!("getting djkstra input from server...");
                          println!("received json file!");  

                          status_client=StatusClient::inquire_roots;
                      }
                      StatusClient::inquire_roots=>
                      {
                          json_input =string_to_static_str(servermsg);
                        // uses boxing for coping the value of servermsg to static variable for saving json input                                                              
                          println!("inquireing roots from server... Just Type Return to start calculation");
                          println!("NOTE: IF YOU DO ANY OUTHER INPUT THEN JUST ENTER YOU WILL QUIT THE PROCESS!");
                      }
                         StatusClient::receive_roots=> 
                      {
                          println!("getting roots from server...");
                          println!("received roots!");

                          status_client=StatusClient::inquire_calculation_success;
                        
                      }
                      StatusClient::inquire_calculation_success=>
                       {
                          println!("{}", servermsg);
                          roots_input=string_to_static_str(servermsg);
                         // uses boxing for coping the value of servermsg to static variable for saving root input 
                          println!("");
                          println!("calculating..");
                          // HERE HAS TO BE THE CALCULATION!!
                          let st_nodes:Vec<&str> = roots_input.split(";").collect();
                          for i in 0..st_nodes.len()
                          {
                             result_string=string_to_static_str(format!("{}:{}", result_string, dijkstra(st_nodes[i].parse::<i32>().unwrap()).to_string()));
                          }
                          status_client=StatusClient::send_calculation_success;
                       }
                      StatusClient::send_calculation_success => 
                      {
                          status_client=StatusClient::confirm_calculation_success;
                          println!("done");
                      }
                      StatusClient::confirm_calculation_success=>
                      {
                          status_client=StatusClient::confirm_calculation_success;
                      }
                      StatusClient::error_status=> {println!("{}", MSG_ERROR);}
                       
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

fn ask_client()
{
    println!("Would you like to join our distributed system for calculating shortest paths with djikstra algorithm? Type 'y' !");
    println!("NOTE: IF YOU DO ANY OUTHER INPUT THEN 'y'YOU WILL QUIT THE PROCESS!");
}
fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

impl actix::io::WriteHandler<WsProtocolError> for ChatClient {}
