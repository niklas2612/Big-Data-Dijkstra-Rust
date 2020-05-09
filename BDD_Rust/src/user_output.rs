use std::collections::HashMap;

#[derive(Copy, Clone)]
struct TableLine {
    pre_node: u16,
    costs: u16,
}

pub struct Table {
    table_lines: HashMap<u16, TableLine>,
    start_node: u16,
}

impl Table {
    fn add_line_to_table(&mut self, line: TableLine, node: u16) -> () {
        self.table_lines.insert(node, line);
    }

    

    fn get_costs(&self, node: u16) -> u16 {
        return self.table_lines[&node].costs;
    }

    fn get_pre_node(&self, node: u16) -> u16 {
        return self.table_lines[&node].pre_node;
    }

    pub fn get_path(&self, node: u16) -> String {
        let mut current_pre_node: u16 = self.get_pre_node(node);
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

pub fn create_table_from_string(string: &str) -> (i32, Table) {
    //0;0,0,0;1,2,3;...
    let tokens: Vec<&str> = string.split(";").collect();

    let lines = HashMap::new();
    let start_node: i32 = tokens[0].parse().unwrap();

    let mut table = Table {
        table_lines: lines,
        start_node: start_node as u16,
    };

    let mut current_line: Vec<&str>;
    let mut current_table_line: TableLine;
    let mut cur_pn: u16;
    let mut cur_co: u16;
    let mut cur_no: u16;

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
