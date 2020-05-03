use serde::{Deserialize, Serialize};
use serde_json::Result;


#[derive(Serialize, Deserialize)]
struct Input {

  paths: Vec<PathInformation>, 
}

#[derive(Serialize, Deserialize, Clone)]
struct PathInformation{      // struct so save relevant path information

    from: String,
    to: String,
    costs: u16,
}

pub fn dijkstra (start_node:i32, data: &str, Roots: Vec<i32>) -> String{
    
    let input: Input = (serde_json::from_str(&data)).unwrap();

    let amount_nodes = Roots.len();
    let amount_paths = input.paths.len();

    
    let max:i32=i32::max_value();
    

    

    //println!("Distance from {} to {} is {}", input.paths[0].from, input.paths[0].to, input.paths[0].costs);
    //println!("Distance from {} to {} is {}", input.paths[1].from, input.paths[1].to, input.paths[1].costs);

    let mut node1 = Vec::new();
    let mut node2 = Vec::new();
    let mut distance = Vec::new();
    let mut nodelist = Roots;


      
 
    for a in 0..input.paths.len(){
 
        node1[a] = input.paths[a].to.parse::<i32>().unwrap();
        node2[a] = input.paths[a].from.parse::<i32>().unwrap();
        distance[a] = input.paths[a].costs as i32;
 
    }
    
    /*node1[0]=0;
    node2[0]=1;
    distance[0]=2;

    node1[1]=2;
    node2[1]=1;
    distance[1]=2;

    node1[2]=2;
    node2[2]=0;
    distance[2]=7;

    node1[3]=0;
    node2[3]=3;
    distance[3]=3;

    node1[4]=2;
    node2[4]=4;
    distance[4]=1;
  
    node1[5]=5;
    node2[5]=1;
    distance[5]=14;

    node1[6]=5;
    node2[6]=4;
    distance[6]=3;

    node1[7]=3;
    node2[7]=4;
    distance[7]=4;*/


   // for i in 0..7 {
   //     println!("From {} to {} is the distance {}",
   //     node1[i],
   //     node2[i],
    //    distance[i],
   // );  
    //}
  
   


    //INITALISIERUNG

    let mut table_node = Vec::new();
    let mut table_precursor= Vec::new();
    let mut table_distance= Vec::new();

    for b in 0..amount_nodes{
        table_node.push(0);
        table_precursor.push(0);
        table_distance.push(max);
    }

    table_distance[(start_node) as usize]=0;
    let mut table_number:i32=0;
    //println!("Table {}", table_number);
    for j in 0..amount_nodes {
        table_node[j]=nodelist[j];
      //  println!("{} {} {}", table_node[j], table_precursor[j], table_distance[j])
    }


//DIJKSTRA

let mut iii=1;
    loop{
        
        //println!("run {}", iii);
        iii=iii+1;
        if nodelist[0]==max && nodelist[1]==max && nodelist[2]==max && nodelist[3]==max && nodelist[4]==max && nodelist[5]==max{
            break;
        }

        let mut u:i32=max; //index von u aus nodelist
        let mut u_value:i32=max; //wert von u aus nodelist
        //let mut maxhelp:i32=i32::max_value();
        let mut maxhelp:i32=max;
        //println!("u ist {}", u);
        for i in 0..(amount_nodes) {
            //println!("i ist {}", i);
            if table_distance[i]<=maxhelp && nodelist[i]!=max{
                maxhelp=table_distance[i];
                u=(i) as i32;
                u_value=nodelist[i];
                //println!("question");
            }
        }
        //entferne u aus Q(Nodelist)
        //println!("nodelist {}", nodelist[0]);        
        //println!("nodelist {}", nodelist[1]);
        //println!("nodelist {}", nodelist[2]);
        //println!("nodelist {}", nodelist[3]);        
        //println!("nodelist {}", nodelist[4]);
        //println!("nodelist {}", nodelist[5]);
        //println!("u ist {}", u);
        nodelist[(u) as usize]=max;

       // println!("u ist {}", u);

        for i in 0..(amount_paths-1) {
            //println!("aaaa");
            if node1[i]==u || node2[i]==u{
                //println!("bbb");
                //println!("{} {} {}", node1[i], node2[i], distance[i]);

                for j in 0..(amount_nodes) {
                    //println!("ccc");
                    if nodelist[j]==node1[i] || nodelist[j]==node2[i]{
                        //println!("{} {} {}", node1[i], node2[i], distance[i]);

                        let mut v_value:i32=max; //wert von v aus nodes
                        if nodelist[j]==node1[i]{
                            v_value=node1[i];

                        }else if nodelist[j]==node2[i]{
                            v_value=node2[i];
                        }
                        //println!("v_value is {}", v_value);

                        //alternative streckendistanz berechnen
                        let distance_alt;

                        distance_alt=table_distance[(u) as usize] + distance[i];
                        //println!("distance alt is {}", distance_alt);
                        //println!("node {}", table_node[j]);
                        if distance_alt<table_distance[j]{
                            table_distance[j]= distance_alt;
                            table_precursor[j]=u_value;
                        }



                    }
                }

            }
        }
        
        table_number=table_number+1;
        
        //break;
    }
    //Leos string bauen
    let mut leosendstring:String;
    leosendstring= start_node.to_string() + ";";
    

    for j in 0..amount_nodes {

        leosendstring=leosendstring.to_string()+ &table_node[j].to_string()+","+ &table_precursor[j].to_string()+","+ &table_distance[j].to_string()+";";
        
    }
   
    return leosendstring;

}