use core::num;
use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    ops::{Add, Index, Not, Sub},
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LogicLevel {
    Invalid,
    Low,
    High,
    Z,
}

impl Not for LogicLevel {
    type Output = LogicLevel;

    fn not(self) -> Self::Output {
        match self {
            LogicLevel::Invalid => LogicLevel::Invalid,
            LogicLevel::Low => LogicLevel::High,
            LogicLevel::High => LogicLevel::Low,
            LogicLevel::Z => LogicLevel::Invalid,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LogicLevels(Vec<LogicLevel>);

impl LogicLevels {
    fn filled(bit_width: i32, logic_level: LogicLevel) -> Self {
        LogicLevels((0..bit_width).map(|_| logic_level).collect())
    }
}

impl Index<usize> for LogicLevels {
    type Output = LogicLevel;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl Not for LogicLevels {
    type Output = LogicLevels;

    fn not(self) -> Self::Output {
        LogicLevels(self.0.iter().map(|x| !*x).collect())
    }
}

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
                vec![Node::new_out(self.pos + Vec2i(1, 0), bit_width)]
            }

            LogicGateType::Output { bit_width, .. } => {
                vec![Node::new_in(self.pos - Vec2i(1, 0), bit_width)]
            }

            LogicGateType::And { .. } => todo!(),

            LogicGateType::Or { .. } => todo!(),

            LogicGateType::Not { bit_width, .. } => {
                vec![
                    Node::new_in(self.pos - Vec2i(1, 0), bit_width),
                    Node::new_out(self.pos + Vec2i(1, 0), bit_width),
                ]
            }
        }
    }

    pub fn tick(
        &self,
        node_idxs: &[usize],
        sim_nodes: &[SimulationNode],
        pq: &mut SimulationQueue,
        subtick: i32,
    ) {
        match &self.gate_type {
            LogicGateType::Circuit(_circuit) => todo!(),

            LogicGateType::Input { logic_levels, .. } => {
                pq.push(QueueEvent(
                    subtick,
                    QueueEventType::UpdateNode(node_idxs[0], logic_levels.clone()),
                ));
            }
            // nodes.get_mut(0).unwrap().logic_levels = logic_levels.to_vec(),
            LogicGateType::Output { logic_levels, .. } => {
                // TODO separate state from LogicGate
                // *logic_levels = sim_nodes[node_idxs[0]].logic_levels.clone()
            }

            LogicGateType::And { .. } => todo!(),

            LogicGateType::Or { .. } => todo!(),

            LogicGateType::Not {
                bit_width,
                propagation_delay,
            } => {
                pq.push(QueueEvent(
                    subtick + propagation_delay,
                    QueueEventType::UpdateNode(
                        node_idxs[1],
                        !sim_nodes[node_idxs[0]].logic_levels.clone(),
                    ),
                ));
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
    pub fn create_nodes(&self) -> Vec<Node> {
        vec![
            Node::new_in_out(self.pos1, self.bit_width),
            Node::new_in_out(self.pos2, self.bit_width),
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
enum NodeType {
    In,
    Out,
    InOut,
}

/// A connection node of logic gates and wires.
#[derive(PartialEq, Eq, Clone)]
struct Node {
    pub node_type: NodeType,
    pub pos: Vec2i,
    pub bit_width: i32,
}

impl Node {
    pub fn new_in(pos: Vec2i, bit_width: i32) -> Self {
        Node {
            node_type: NodeType::In,
            pos,
            bit_width,
        }
    }

    pub fn new_out(pos: Vec2i, bit_width: i32) -> Self {
        Node {
            node_type: NodeType::Out,
            pos,
            bit_width,
        }
    }

    pub fn new_in_out(pos: Vec2i, bit_width: i32) -> Self {
        Node {
            node_type: NodeType::InOut,
            pos,
            bit_width,
        }
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
            logic_levels: LogicLevels::filled(bit_width, LogicLevel::Invalid),
        }
    }
}

type SimulationQueue = BinaryHeap<QueueEvent>;

#[derive(PartialEq, Eq, Clone, Debug)]
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

#[derive(PartialEq, Eq, Clone, Debug)]
enum QueueEventType {
    UpdateGate(usize),
    UpdateNode(usize, LogicLevels),
}

struct Simulation<'a> {
    pub circuit: &'a Circuit,

    pub sim_nodes: Vec<SimulationNode>,
    // pub input_sim_node_ids: Vec<usize>,
    pub input_gate_idxs: Vec<usize>,

    pub node_idxs_by_gate_idx: Vec<Vec<usize>>,
    pub gate_idxs_by_node_idx: Vec<usize>,

    pub node_sources: Vec<NodeSource<'a>>,
}

impl<'a> Simulation<'a> {
    pub fn new(circuit: &'a Circuit) -> Self {
        let gates = &circuit.gates;
        let wires = &circuit.wires;

        let mut dsu = Dsu::<usize>::new();
        let mut node_idx_counter = 0;
        let mut all_nodes = Vec::<Node>::new();

        let mut node_sources = Vec::new();

        let mut node_idxs_by_gate_idx = Vec::new();
        let mut gate_idxs_by_node_idx = HashMap::new();

        let mut input_gate_idxs = Vec::new();

        // create nodes from logic gates
        for (gate_idx, gate) in gates.iter().enumerate() {
            let nodes = gate.create_nodes();
            let mut ids = Vec::new();

            for node in nodes {
                let node_idx = node_idx_counter;
                node_idx_counter += 1;

                dsu.make_set(node_idx);
                all_nodes.push(node);
                node_sources.push(NodeSource::LogicGate(&gate));

                gate_idxs_by_node_idx.insert(node_idx, gate_idx);

                ids.push(node_idx);
            }

            node_idxs_by_gate_idx.push(ids);

            if gate.gate_type.is_start() {
                input_gate_idxs.push(gate_idx);
            }
        }

        // create nodes from wires
        for wire in wires {
            let nodes = wire.create_nodes();
            let mut node_idxs = Vec::new();

            for node in nodes {
                let node_idx = node_idx_counter;
                node_idx_counter += 1;

                dsu.make_set(node_idx);
                all_nodes.push(node);
                node_sources.push(NodeSource::Wire(&wire));

                node_idxs.push(node_idx);
            }

            // connect nodes within the same wire
            for i in &node_idxs[1..] {
                dsu.union(node_idxs[0], *i);
            }
        }

        // connect two nodes together if they share the same position
        for i in 0..all_nodes.len() {
            for j in 0..i {
                if all_nodes[i].pos == all_nodes[j].pos {
                    dsu.union(i, j);
                }
            }
        }

        let mut sim_node_idxs_by_root = HashMap::new();
        let mut sim_node_idxs_by_node_idx = Vec::new();
        let mut sim_node_idx_counter = 0;

        for node_idx in 0..node_idx_counter {
            let root = dsu.find(node_idx);

            let sim_node_idx = sim_node_idxs_by_root.entry(root).or_insert_with(|| {
                let sim_node_idx = sim_node_idx_counter;
                sim_node_idx_counter += 1;
                sim_node_idx
            });

            sim_node_idxs_by_node_idx.push(sim_node_idx.clone());
        }

        println!("node idx -> sim node idx");
        for i in 0..node_idx_counter {
            println!("{} -> {}", i, sim_node_idxs_by_node_idx[i]);
        }

        // create simulation nodes
        let sim_nodes = all_nodes
            .iter()
            .map(|node| SimulationNode::new(node.bit_width))
            .collect::<Vec<_>>();

        todo!();

        // create the struct
        // Simulation {
        //     circuit,
        //     sim_nodes,
        //     input_gate_idxs,
        //     node_idxs_by_gate_idx,
        //     gate_idxs_by_node_idx,
        //     // input_sim_node_ids,
        //     node_sources,
        // }
    }

    pub fn tick(&mut self) {
        // a priority queue of (subtick, update event)
        let mut pq = SimulationQueue::new();

        // push all input gates into the queue
        for gate_idx in &self.input_gate_idxs {
            pq.push(QueueEvent(0, QueueEventType::UpdateGate(*gate_idx)));
        }

        // for sim_node in &mut self.sim_nodes {
        //     for level in &mut sim_node.logic_levels {
        //         *level = LogicLevel::Z;
        //     }
        // }

        // loop while there is something in the queue to proceed
        const MAX_ITERS: i32 = 10;
        let mut iter = 0;

        while !pq.is_empty() && iter < MAX_ITERS {
            iter += 1;

            let curr_subtick = pq.peek().unwrap().0;

            // retrieve all to-be-processed queue items
            let mut to_update = Vec::new();

            while pq.peek().is_some_and(|item| item.0 == curr_subtick) {
                to_update.push(pq.pop().unwrap().1);
            }

            println!("subtick {}", curr_subtick);
            for item in &to_update {
                println!("{:?}", item);
            }

            // update all logic gates within the current subtick
            for item in to_update {
                match item {
                    QueueEventType::UpdateGate(gate_idx) => {
                        let gate = &self.circuit.gates[gate_idx];

                        let node_idxs = &self.node_idxs_by_gate_idx[gate_idx];
                        gate.tick(node_idxs, &self.sim_nodes, &mut pq, curr_subtick);

                        // gate.process(nodes) -- update the nodes, e.g. change outputs given inputs
                        // then find what nodes have changed its states
                        // then find what gates got affected by said changed nodes
                        // then add all of those gates to the heap
                    }

                    QueueEventType::UpdateNode(node_idx, logic_levels) => {
                        self.sim_nodes[node_idx].logic_levels = logic_levels;

                        let affecting_gate_idx = self.gate_idxs_by_node_idx[node_idx];

                        pq.push(QueueEvent(
                            curr_subtick,
                            QueueEventType::UpdateGate(affecting_gate_idx),
                        ));
                    }
                }
            }
        }
    }
}

fn main() {
    let mut circuit = Circuit::new();

    let input = LogicGate {
        gate_type: LogicGateType::Input {
            bit_width: 1,
            logic_levels: LogicLevels::filled(1, LogicLevel::Low),
        },
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
        gate_type: LogicGateType::Output {
            bit_width: 1,
            logic_levels: LogicLevels::filled(1, LogicLevel::Invalid),
        },
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

    println!("gate idx -> [node idx]");
    for i in 0..sim.node_idxs_by_gate_idx.len() {
        println!("{} -> {:?}", i, sim.node_idxs_by_gate_idx[i]);
    }

    println!("node idx -> gate idx");
    for i in 0..sim.gate_idxs_by_node_idx.len() {
        println!("{} -> {}", i, sim.gate_idxs_by_node_idx[i]);
    }

    sim.tick();
}
