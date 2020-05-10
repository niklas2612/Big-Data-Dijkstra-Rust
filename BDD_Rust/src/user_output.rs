use std::collections::HashMap;

//struct simulates a table line
#[derive(Copy, Clone)]
struct TableLine {
    pre_node: i32,
    costs: i32,
}


//struct simulates a table
pub struct Table {
    table_lines: HashMap<i32, TableLine>,
    start_node: i32,
}

impl Table {
    fn add_line_to_table(&mut self, line: TableLine, node: i32) -> () {
        self.table_lines.insert(node, line);
    }


    //gets costs from own startnode to parameternode
    fn get_costs(&self, node: i32) -> i32 {
        return self.table_lines[&node].costs;
    }


    //get prenode of parameternode
    fn get_pre_node(&self, node: i32) -> i32 {
        return self.table_lines[&node].pre_node;
    }


    //get path from own startnode to node given as parameter
    pub fn get_path(&self, node: i32) -> String {
        let mut current_pre_node: i32 = self.get_pre_node(node);
        let mut path: String = format!(" - {}", node);

        while current_pre_node != self.start_node {
            path = format!("{}{}{}", " - ", current_pre_node, path);

            current_pre_node = self.get_pre_node(current_pre_node);
        }

        path = format!(
            "{}{}, costs:{}",
            self.start_node,
            path,
            self.get_costs(node)
        );
        return path;
    }
}


//returns an tuple (instance of tablestruct and startnode) built from an input string
pub fn create_table_from_string(string: &str) -> (i32, Table) {
    
    let tokens: Vec<&str> = string.split(";").collect();

    let lines = HashMap::new();
    let start_node: i32 = tokens[0].parse().unwrap();

    let mut table = Table {
        table_lines: lines,
        start_node: start_node as i32,
    };

    let mut current_line: Vec<&str>;
    let mut current_table_line: TableLine;
    let mut cur_pn: i32;
    let mut cur_co: i32;
    let mut cur_no: i32;

    for i in 1..tokens.len() {
        current_line = tokens[i].split(",").collect();
        cur_no = current_line[0].parse().unwrap();
        cur_pn = current_line[1].parse().unwrap();
        cur_co = current_line[2].parse().unwrap();

        current_table_line = TableLine {
            pre_node: cur_pn,
            costs: cur_co,
        };

        table.add_line_to_table(current_table_line, cur_no);
    }

    return (start_node, table);
}
