use core::num;
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
    Invalid,
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
        logic_levels: LogicLevels,
    },
    Output {
        bit_width: i32,
        logic_levels: LogicLevels,
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

impl LogicGateType {
    pub fn is_start(&self) -> bool {
        match self {
            LogicGateType::Input { .. } => true,
            LogicGateType::Circuit(_) => todo!(),
            _ => false,
        }
    }
}

struct LogicGate {
    pub gate_type: LogicGateType,
    pub pos: Vec2i,
}

impl LogicGate {
    pub fn create_nodes(&self) -> Vec<Node> {
        match self.gate_type {
            LogicGateType::Circuit(ref _circuit) => todo!(),

            LogicGateType::Input { bit_width, .. } => {
                vec![Node::new(self.pos + Vec2i(1, 0), bit_width)]
            }

            LogicGateType::Output { bit_width, .. } => {
                vec![Node::new(self.pos - Vec2i(1, 0), bit_width)]
            }

            LogicGateType::And { .. } => todo!(),

            LogicGateType::Or { .. } => todo!(),

            LogicGateType::Not { bit_width, .. } => {
                vec![
                    Node::new(self.pos - Vec2i(1, 0), bit_width),
                    Node::new(self.pos + Vec2i(1, 0), bit_width),
                ]
            }
        }
    }

    pub fn tick(&mut self, nodes: &[SimulationNode], pq: &mut SimulationQueue) {
        match self.gate_type {
            LogicGateType::Circuit(ref _circuit) => todo!(),

            LogicGateType::Input {
                ref logic_levels, ..
            } => nodes.get_mut(0).unwrap().logic_levels = logic_levels.to_vec(),

            LogicGateType::Output {
                ref mut logic_levels,
                ..
            } => *logic_levels = nodes.get(0).unwrap().logic_levels.clone(),

            LogicGateType::And { .. } => todo!(),

            LogicGateType::Or { .. } => todo!(),

            LogicGateType::Not {
                bit_width,
                propagation_delay,
            } => todo!(),
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
    pub fn create_nodes(&self) -> Vec<Node> {
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

/// A connection node of logic gates and wires.
#[derive(PartialEq, Eq, Clone)]
struct Node {
    pub pos: Vec2i,
    pub bit_width: i32,
}

impl Node {
    pub fn new(pos: Vec2i, bit_width: i32) -> Self {
        Node { pos, bit_width }
    }
}

// --- simulation

/// Source element of a simulation node; either a logic gate or a wire.
enum NodeSource<'a> {
    LogicGate(&'a LogicGate),
    Wire(&'a Wire),
}

#[derive(PartialEq, Eq, Clone)]
struct SimulationNode {
    pub logic_levels: LogicLevels,
}

impl SimulationNode {
    pub fn new(bit_width: i32) -> Self {
        SimulationNode {
            logic_levels: (0..bit_width - 1).map(|_| LogicLevel::Invalid).collect(),
        }
    }
}

type SimulationQueue = BinaryHeap<QueueEvent>;

#[derive(PartialEq, Eq, Clone)]
struct QueueEvent(i32, QueueEventType);

impl PartialOrd for QueueEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.0.cmp(&self.0))
    }
}

impl Ord for QueueEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.0.cmp(&self.0)
    }
}

#[derive(PartialEq, Eq, Clone)]
enum QueueEventType {
    UpdateGate(usize),
    UpdateNode(usize, LogicLevels),
}

struct Simulation<'a> {
    pub circuit: &'a Circuit,

    pub sim_nodes: Vec<SimulationNode>,
    // pub input_sim_node_ids: Vec<usize>,
    pub input_gate_indices: Vec<usize>,

    pub nodes_by_gate_indices: Vec<Vec<usize>>,

    pub node_sources: Vec<NodeSource<'a>>,
}

impl<'a> Simulation<'a> {
    pub fn new(circuit: &'a Circuit) -> Self {
        let gates = &circuit.gates;
        let wires = &circuit.wires;

        let mut dsu = Dsu::<usize>::new();
        let mut id_counter = 0;
        let mut all_nodes = Vec::<Node>::new();
        let mut node_sources = Vec::new(); // TODO separate vector; one for gates, another for wires
        let mut nodes_by_gate_indices = Vec::new();
        let mut input_gate_indices = Vec::new();

        // create nodes from logic gates
        for (gate_idx, gate) in gates.iter().enumerate() {
            let nodes = gate.create_nodes();
            let mut ids = Vec::new();

            for node in nodes {
                let id = id_counter;
                id_counter += 1;

                dsu.make_set(id);
                all_nodes.push(node);
                node_sources.push(NodeSource::LogicGate(&gate));

                ids.push(id);
            }

            nodes_by_gate_indices.push(ids);

            if gate.gate_type.is_start() {
                input_gate_indices.push(gate_idx);
            }
        }

        // create nodes from wires
        for wire in wires {
            let nodes = wire.create_nodes();
            let mut ids = Vec::new();

            for node in nodes {
                let id = id_counter;
                id_counter += 1;

                dsu.make_set(id);
                all_nodes.push(node);
                node_sources.push(NodeSource::Wire(&wire));

                ids.push(id);
            }

            // connect nodes within the same wire
            for id in &ids[1..] {
                dsu.union(ids[0], *id);
            }
        }

        // connect two nodes together if they share the same position
        for i in 0..all_nodes.len() - 1 {
            for j in 0..i {
                if all_nodes[i].pos == all_nodes[j].pos {
                    dsu.union(i, j);
                }
            }
        }

        // union
        // let input_sim_node_ids = input_node_ids.iter().map(|id| dsu.find(*id)).collect();

        // create simulation nodes
        let sim_nodes = all_nodes
            .iter()
            .map(|node| SimulationNode::new(node.bit_width))
            .collect();

        // create the struct
        Simulation {
            circuit,
            sim_nodes,
            input_gate_indices,
            nodes_by_gate_indices,
            // input_sim_node_ids,
            node_sources,
        }
    }

    pub fn tick(&mut self) {
        // a priority queue of (subtick, update event)
        let mut pq = SimulationQueue::new();

        // push all input gates into the queue
        for gate_idx in &self.input_gate_indices {
            pq.push(QueueEvent(0, QueueEventType::UpdateGate(*gate_idx)));
        }

        // for sim_node in &mut self.sim_nodes {
        //     for level in &mut sim_node.logic_levels {
        //         *level = LogicLevel::Z;
        //     }
        // }

        // loop while there is something in the queue to proceed
        while !pq.is_empty() {
            let curr_t = pq.peek().unwrap().0;

            // retrieve all to-be-processed queue items
            let mut to_update = Vec::new();

            while pq.peek().is_some_and(|item| item.0 == curr_t) {
                to_update.push(pq.pop().unwrap().1);
            }

            // update all logic gates within the current subtick
            while pq.peek().unwrap().0 == curr_t {
                let item = pq.pop().unwrap().1;

                match item {
                    QueueEventType::UpdateGate(gate_idx) => {
                        let gate = self.circuit.gates.get(gate_idx).unwrap();

                        // TODO obtain sim nodes from gate idx

                        // let nodes = self.node_sources.get(gate_idx).unwrap();

                        gate.tick(nodes, &mut pq);

                        // gate.process(nodes) -- update the nodes, e.g. change outputs given inputs
                        // then find what nodes have changed its states
                        // then find what gates got affected by said changed nodes
                        // then add all of those gates to the heap
                    }

                    QueueEventType::UpdateNode(node_idx) => todo!(),
                }
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

    let mut sim = Simulation::new(&circuit);

    sim.tick();
}
