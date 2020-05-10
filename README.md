# websocket

Instructions for running BDD.

## Running program

### server

```bash
cd examples/websocket
cargo run --bin websocket-server
```

### client

```bash
cd examples/websocket
cargo run --bin websocket-client
```

## JSON-file

### structure

One JSON-object needs to contain a "from"-value (ID of startnode), a "to"-value (ID of target node) and a costs-value. One of this objects contains the information for one path. All of these objects should be contained in the Array "paths":

```
{
  "paths": 
  [
    {
      "from": "0",
      "to": "1",
      "costs": 2
    },

    {
      "from": "1",
      "to": "2",
      "costs": 4
    }
    ]
  }
  ```
  The defined paths are interpreted as undirected, so you can go from "to"-node to "from"-node with the same costs.
  
  NOTE: Node-IDs have to be numbers! 
  
  ### location of json-file
  
  The JSON-File has to be placed in folde "input_data" and named "input.json". In the existing repository there are two examples which are both accepted by the program. The user is responsible for the correctness of the JSON-file by himself.
