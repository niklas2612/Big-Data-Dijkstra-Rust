use serde::{Deserialize, Serialize};


use crate::input_output::*;

#[derive(Serialize, Deserialize)]
struct Input {
    paths: Vec<PathInformation>,
}

#[derive(Serialize, Deserialize, Clone)]
struct PathInformation {
    // struct so save maxrelevant path information
    from: String,
    to: String,
    costs: u16,
}

pub fn dijkstra(start_node: i32, data: &str) -> String {
    //println!("dijkstra calculation started");

    let mut rootlist = RootList { roots: Vec::new() };
    let mut nodelist = Vec::new();
    input_parse(&mut rootlist, String::from(data)).unwrap();

    //println!( "nodelist");
    for a in 0..rootlist.roots.len() {
        nodelist.push(rootlist.roots[a].parse::<i32>().unwrap());
    }

    let input: Input = (serde_json::from_str(&data)).unwrap();

    let amount_nodes = nodelist.len();
    let amount_paths = input.paths.len();

    let mut max: i32 = i32::max_value();
    //println!("max {}", max);
    max = (max - 1) / 2;
    let mut node1 = Vec::new();
    let mut node2 = Vec::new();
    let mut distance = Vec::new();

    for a in 0..input.paths.len() {
        node1.push(input.paths[a].to.parse::<i32>().unwrap());
        node2.push(input.paths[a].from.parse::<i32>().unwrap());
        distance.push(input.paths[a].costs as i32);
    }

    //INITALISIERUNG

    let mut table_node = Vec::new();
    let mut table_precursor = Vec::new();
    let mut table_distance = Vec::new();

    for _b in 0..amount_nodes {
        table_node.push(0);
        table_precursor.push(0);
        table_distance.push(max);
    }

    let mut table_number: i32 = 0;

    for j in 0..amount_nodes {
        table_node[j] = nodelist[j];
    }

    for z in 0..amount_nodes {
        if start_node == table_node[z] {
            table_distance[z] = 0;
        }
    }

    //DIJKSTRA

    let mut iii = 1;
    loop {
        iii = iii + 1;
        let mut nodelist_empty = true;
        for d in 0..amount_nodes {
            if nodelist[d] != max {
                nodelist_empty = false;
            }
        }

        if nodelist_empty == true {
            break;
        }
        //println!("Q nicht leer");
        let mut u: i32 = max; //index von u aus nodelist
        let mut u_value: i32 = max; //wert von u aus nodelist
                                    //let mut maxhelp:i32=i32::max_value();
        let mut maxhelp: i32 = max;
        //println!("u ist {}", u);

        for i in 0..(amount_nodes) {
            //println!("i ist {}", i);

            if table_distance[i] <= maxhelp && nodelist[i] != max {
                //<= ??
                //println!("test 123, {}",i);
                maxhelp = table_distance[i];
                u = (i) as i32;
                u_value = table_node[i]; //eventuell tablenode statt nodelist
                                         //println!("question");
            }
        }

        nodelist[(u) as usize] = max;

        iii = iii + 1;

        for i in 0..(amount_paths) {
            if node1[i] == u_value || node2[i] == u_value {
                for j in 0..amount_nodes {
                    if nodelist[j] == node1[i] || nodelist[j] == node2[i] {
                        //alternative streckendistanz berechnen
                        let distance_alt = table_distance[(u) as usize] + distance[i];

                        if distance_alt < table_distance[j] {
                            table_distance[j] = distance_alt;
                            table_precursor[j] = u_value;
                        }
                    }
                }
            }
        }

        table_number = table_number + 1;
    }
    //Vorgänger von Startknoten setzen
    for d in 0..amount_nodes {
        if start_node == table_node[d] {
            table_precursor[d] = start_node;
        }
    }

    let mut result_string: String;
    result_string = start_node.to_string() + ";";

    for j in 0..amount_nodes {
        result_string = result_string.to_string()
            + &table_node[j].to_string()
            + ","
            + &table_precursor[j].to_string()
            + ","
            + &table_distance[j].to_string();
        if j != (amount_nodes - 1) {
            result_string = result_string + ";";
        }
    }

    return result_string;
}
