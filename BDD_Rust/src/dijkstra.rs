use serde::{Deserialize, Serialize};
use serde_json::Result;

use crate::input_output::*;


#[derive(Serialize, Deserialize)]
struct Input {

  paths: Vec<PathInformation>, 
}

#[derive(Serialize, Deserialize, Clone)]
struct PathInformation{      // struct so save maxrelevant path information

    from: String,
    to: String,
    costs: u16,
}

pub fn dijkstra (start_node:i32, data: &str) -> String{

    //println!("dijkstra calculation started");

    let mut rootlist= RootList{roots:Vec::new()};
    let mut nodelist = Vec::new();
    input_parse(&mut rootlist,String::from(data)).unwrap();     

    //println!( "nodelist");
    for a in 0..rootlist.roots.len(){

        //println!("{}",rootlist.roots[a]);
        nodelist.push(rootlist.roots[a].parse::<i32>().unwrap());

    }

    let input: Input = (serde_json::from_str(&data)).unwrap();

    let amount_nodes = nodelist.len();
    let amount_paths = input.paths.len();

    
    let mut max:i32=i32::max_value();
    //println!("max {}", max);
    max = (max-1)/2;
    //println!("max {}", max);
    
    

    //println!("Distance from {} to {} is {}", input.paths[0].from, input.paths[0].to, input.paths[0].costs);
    //println!("Distance from {} to {} is {}", input.paths[1].from, input.paths[1].to, input.paths[1].costs);

    let mut node1 = Vec::new();
    let mut node2 = Vec::new();
    let mut distance = Vec::new();
    



    //println!("length von noelist{}", nodelist.len());
 
    for a in 0..input.paths.len(){
 
        node1.push(input.paths[a].to.parse::<i32>().unwrap());
        node2.push(input.paths[a].from.parse::<i32>().unwrap());
        distance.push(input.paths[a].costs as i32);
 
    }
    //println!("alle pfade:");
    for p in 0..node1.len() {
        //println!("{} {} {}", node1[p], node2[p], distance[p])
    }
    


   // for i in 0..7 {
   //     println!("From {} to {} is the distance {}",
   //     node1[i],
   //     node2[i],
    //    distance[i],
   // );  
    //}
  
   
    //println!("kurz vor init");

    //INITALISIERUNG

    let mut table_node = Vec::new();
    let mut table_precursor= Vec::new();
    let mut table_distance= Vec::new();

    for b in 0..amount_nodes{
        table_node.push(0);
        table_precursor.push(0);
        table_distance.push(max);
    }

    let mut table_number:i32=0;
    //println!("Table {}", table_number);
    for j in 0..amount_nodes {
        table_node[j]=nodelist[j];
      
    }


    //println!("startnode {}", start_node);
    for z in 0..amount_nodes {
        //println!("table_node[z] {}", table_node[z]);
        if start_node==table_node[z]{
            
            table_distance[z]=0;
            //println!("table_distance[z] {}", table_distance[z]);
        }
    }
println!("\n\n\n\n\n\n\n");
    //table_distance[(start_node) as usize]=0;
    for j in 0..amount_nodes {
        
      //println!("nach init: {} {} {}", table_node[j], table_precursor[j], table_distance[j])
    }


//DIJKSTRA

//println!("hier beginnt die dijkstra berechnung");

let mut iii=1;
    loop{
        
        //println!("length von nodelist{}", nodelist.len());
        //println!("Q: ");
        iii=iii+1;
        let mut nodelist_empty=true;
        for d in 0..amount_nodes {
            //println!(" {} ", nodelist[d]);
            if nodelist[d]!=max{
                nodelist_empty=false;
            }
        }

        if nodelist_empty==true{
            break;
        }
        //println!("Q nicht leer");
        let mut u:i32=max; //index von u aus nodelist
        let mut u_value:i32=max; //wert von u aus nodelist
        //let mut maxhelp:i32=i32::max_value();
        let mut maxhelp:i32=max;
        //println!("u ist {}", u);



        for i in 0..(amount_nodes) {
            //println!("i ist {}", i);

            
            if table_distance[i]<=maxhelp && nodelist[i]!=max {      //<= ??
                //println!("test 123, {}",i);
                maxhelp=table_distance[i];
                u=(i) as i32;
                u_value=table_node[i]; //eventuell tablenode statt nodelist
                //println!("question");
            }
            
        }
        //println!("u value {}", u_value);
        //println!("u index {}", u);
        //entferne u aus Q(Nodelist)
        //println!("nodelist {}", nodelist[0]);        
        //println!("nodelist {}", nodelist[1]);
        //println!("nodelist {}", nodelist[2]);
        //println!("nodelist {}", nodelist[3]);        
        //println!("nodelist {}", nodelist[4]);
        //println!("nodelist {}", nodelist[5]);
        //println!("u ist {}", u);
        nodelist[(u) as usize]=max;

        //println!("Q: ");
        iii=iii+1;
        
        for d in 0..amount_nodes {
            //println!(" {} ", nodelist[d]);
            
        }
        

        //println!("Für jeden nachbarn v von u");

        for i in 0..(amount_paths) {
            //println!("aaaa");
            if node1[i]==u_value || node2[i]==u_value{
                //println!("bbb");
                //println!("{} {} {}", node1[i], node2[i], distance[i]);

                for j in 0..amount_nodes {
                    //println!("ccc");

                    
                    if nodelist[j]==node1[i] || nodelist[j]==node2[i]{
                        //println!("{} {} {}", node1[i], node2[i], distance[i]);

                        
                        //alternative streckendistanz berechnen
                        let mut distance_alt=0;
                        //println!("distance alt {}", distance_alt);
                        //println!("table_distance[(u) as usize]) {}", table_distance[(u) as usize]);
                        //println!("distance[i] {}", distance[i]);
                        
                        distance_alt=table_distance[(u) as usize] + distance[i];
                        
                        //println!("distance alt is {}", distance_alt);
                        //println!("node {}", table_node[j]);
                        if distance_alt<table_distance[j]{
                            //println!("hihihi");
                            //println!("vor: {} {} {}", table_node[j], table_precursor[j], table_distance[j]);
                            //println!("start_node {}", start_node);
                            //println!("u-value {}", u_value);
                            table_distance[j]= distance_alt;
                            table_precursor[j]=u_value;
                            //println!("nach: {} {} {}", table_node[j], table_precursor[j], table_distance[j]);
                        }



                    }

                   // println!("contain vergleich fertig");
                }

                //println!("schleife ter,miniertt");
                

            }
        }
        
        //println!("äußere schleife terminiert\n nodelist: ");

        table_number=table_number+1;
        //println!("u: {}, length from nodelist: {}", u, nodelist.len());
        for z in 0..nodelist.len(){
            //print!("{}, ", nodelist[z]);
        }
        //println!("table_node");

        for z in 0..table_node.len(){
            //print!("{}, ", table_node[z]);
        }
        //println!("");


        /*for t in 0..nodelist.len(){

            if nodelist[t] == table_node[u as usize]{
                nodelist.remove(t);
                break;
            }
        }*/

        

        //println!("length von noelist{}", nodelist.len());
        
        //break;
    }
    //Vorgänger von Startknoten setzen
    for d in 0..amount_nodes {
        if start_node==table_node[d]{
            table_precursor[d]=start_node;
        }
    }


    //Leos string bauen
    let mut leosendstring:String;
    leosendstring= start_node.to_string() + ";";
    
    

    for j in 0..amount_nodes {

        leosendstring=leosendstring.to_string()+ &table_node[j].to_string()+","+ &table_precursor[j].to_string()+","+ &table_distance[j].to_string();
        if j!=(amount_nodes-1){
            leosendstring=leosendstring.to_string()+";";
        }
    }
   
    println!("gleich wird returned folgender string: {}", leosendstring);
    return leosendstring;

}