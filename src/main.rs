use std::{
    collections::{BinaryHeap, HashMap},
    ops::{Add, Sub},
    rc::Rc,
};

use crate::dsu::Dsu;

mod dsu;

// --- Vec2i

#[derive(Clone, Copy, PartialEq, Eq)]
struct Vec2i(i32, i32);

impl Add for Vec2i {
    type Output = Vec2i;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2i(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub for Vec2i {
    type Output = Vec2i;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2i(self.0 - rhs.0, self.1 - rhs.1)
    }
}

// ---

#[derive(Clone, Copy, PartialEq, Eq)]
enum LogicLevel {
    Low,
    High,
    Z,
}

type LogicLevels = Vec<LogicLevel>;

// --- logic gates

#[derive(Clone)]
enum LogicGateType {
    Circuit(Rc<Circuit>),
    Input {
        bit_width: i32,
    },
    Output {
        bit_width: i32,
    },
    And {
        bit_width: i32,
        num_inputs: i32,
        propagation_delay: i32,
    },
    Or {
        bit_width: i32,
        num_inputs: i32,
        propagation_delay: i32,
    },
    Not {
        bit_width: i32,
        propagation_delay: i32,
    },
}

struct LogicGate {
    pub gate_type: LogicGateType,
    pub pos: Vec2i,
}

impl LogicGate {
    pub fn get_nodes(&self) -> Vec<Node> {
        match self.gate_type {
            LogicGateType::Circuit(ref circuit) => todo!(),

            LogicGateType::Input { bit_width } => {
                vec![Node::new(self.pos + Vec2i(1, 0), bit_width)]
            }

            LogicGateType::Output { bit_width } => {
                vec![Node::new(self.pos - Vec2i(1, 0), bit_width)]
            }

            LogicGateType::And {
                bit_width,
                num_inputs,
                propagation_delay,
            } => todo!(),

            LogicGateType::Or {
                bit_width,
                num_inputs,
                propagation_delay,
            } => todo!(),

            LogicGateType::Not {
                bit_width,
                propagation_delay,
            } => {
                vec![
                    Node::new(self.pos - Vec2i(1, 0), bit_width),
                    Node::new(self.pos + Vec2i(1, 0), bit_width),
                ]
            }
        }
    }
}

// ---

#[derive(PartialEq, Eq)]
struct Wire {
    pub pos1: Vec2i,
    pub pos2: Vec2i,
    pub bit_width: i32,
}

impl Wire {
    pub fn get_nodes(&self) -> Vec<Node> {
        vec![
            Node::new(self.pos1, self.bit_width),
            Node::new(self.pos2, self.bit_width),
        ]
    }
}

struct Circuit {
    pub gates: Vec<LogicGate>,
    pub wires: Vec<Wire>,
}

impl Circuit {
    pub fn new() -> Self {
        Self {
            gates: vec![],
            wires: vec![],
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
struct Node {
    pub pos: Vec2i,
    pub logic_levels: Vec<LogicLevel>,
}

impl Node {
    pub fn new(pos: Vec2i, bit_width: i32) -> Self {
        Node {
            pos,
            logic_levels: [0..bit_width - 1].iter().map(|_| LogicLevel::Z).collect(),
        }
    }
}

// --- simulation

struct Simulation {
    pub circuit: Box<Circuit>,

    // pub nodes: Vec<LogicGateNode>,
    pub node_src_gate_indices: Vec<usize>,
    pub input_node_indices: Vec<usize>,

    pub sim_node_states: Vec<LogicLevels>,

    pub wire_src_sim_node_state_indices: Vec<usize>,
    pub node_src_sim_node_state_indices: Vec<usize>,
}

impl Simulation {
    pub fn new(circuit: Circuit) -> Self {
        let (node_src_gate_indices, gate_nodes): (Vec<usize>, Vec<Node>) = circuit
            .gates
            .iter()
            .enumerate()
            .flat_map(|(idx, gate)| {
                gate.get_nodes()
                    .into_iter()
                    .map(|node| (idx, node))
                    .collect::<Vec<_>>()
            })
            .unzip();

        let (node_src_wire_indices, wire_nodes): (Vec<usize>, Vec<Node>) = circuit
            .wires
            .iter()
            .enumerate()
            .flat_map(|(idx, wire)| {
                wire.get_nodes()
                    .into_iter()
                    .map(|node| (idx, node))
                    .collect::<Vec<_>>()
            })
            .unzip();

        let all_nodes = [gate_nodes, wire_nodes].concat();
        let num_nodes = all_nodes.len();

        let input_node_indices = node_src_gate_indices
            .iter()
            .enumerate()
            .filter_map(
                |(node_idx, gate_idx)| match circuit.gates[*gate_idx].gate_type {
                    LogicGateType::Input { .. } => Some(node_idx),
                    _ => None,
                },
            )
            .collect::<Vec<_>>();

        let sim_node_states = circuit
            .wires
            .iter()
            .map(|wire| {
                [0..wire.bit_width - 1]
                    .iter()
                    .map(|_| LogicLevel::Z)
                    .collect()
            })
            .collect();

        let mut dsu = Dsu::<usize>::new();

        for i in 0..num_nodes - 1 {
            for j in 0..i {
                if all_nodes[i].pos == all_nodes[j].pos {
                    dsu.union(i, j);
                }
            }
        }

        // for wire in wires
        // dsu.union the two nodes of wire

        let mut counter = 0;
        let mut id_to_gate_node = HashMap::new();
        let mut id_to_wire = HashMap::new();

        for gate_node in gate_nodes {
            let id = counter;
            counter += 1;

            dsu.make_set(id);
            id_to_gate_node.insert(id, gate_node);
        }

        for wire in circuit.wires {
            let id = counter;
            counter += 1;

            dsu.make_set(id);
            id_to_wire.insert(id, wire);
        }

        for wire in circuit.wires {}

        Simulation {
            circuit: circuit.into(),
            // nodes,
            node_src_gate_indices,
            input_node_indices,

            sim_node_states,
        }
    }

    pub fn tick(&mut self) {
        let mut pq = BinaryHeap::<(i32, usize)>::new();

        for idx in &self.input_node_indices {
            pq.push((-0, *idx));
        }

        for node_state in &mut self.sim_node_states {
            for level in node_state {
                *level = LogicLevel::Z;
            }
        }

        let mut curr_t = 0;

        while !pq.is_empty() {
            // update nodes & wires
            while pq.peek().unwrap().0 == curr_t {
                let node_idx = pq.pop().unwrap().1;
                todo!()
            }
            // update gates
            for gate in &self.circuit.gates {
                // TODO
            }
        }
    }
}

fn main() {
    let mut circuit = Circuit::new();

    let input = LogicGate {
        gate_type: LogicGateType::Input { bit_width: 1 },
        pos: Vec2i(0, 0),
    };

    let not = LogicGate {
        gate_type: LogicGateType::Not {
            bit_width: 1,
            propagation_delay: 1,
        },
        pos: Vec2i(5, 0),
    };

    let output = LogicGate {
        gate_type: LogicGateType::Output { bit_width: 1 },
        pos: Vec2i(10, 0),
    };

    let w1 = Wire {
        pos1: Vec2i(1, 0),
        pos2: Vec2i(4, 0),
        bit_width: 1,
    };

    let w2 = Wire {
        pos1: Vec2i(6, 0),
        pos2: Vec2i(9, 0),
        bit_width: 1,
    };

    circuit.gates.push(input);
    circuit.gates.push(not);
    circuit.gates.push(output);

    circuit.wires.push(w1);
    circuit.wires.push(w2);

    let mut sim = Simulation::new(circuit);

    sim.tick();
}
