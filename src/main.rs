use core::num;
use std::{
    collections::{BinaryHeap, HashMap},
    ops::{BitAnd, BitOr, Index, Not},
    rc::Rc,
};

use crate::dsu::Dsu;
use crate::vec2i::Vec2i;

mod dsu;
mod vec2i;

// ---

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

impl BitAnd for LogicLevel {
    type Output = LogicLevel;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LogicLevel::Low, _) | (_, LogicLevel::Low) => LogicLevel::Low,
            (LogicLevel::High, LogicLevel::High) => LogicLevel::High,
            _ => LogicLevel::Invalid,
        }
    }
}

impl BitOr for LogicLevel {
    type Output = LogicLevel;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LogicLevel::High, _) | (_, LogicLevel::High) => LogicLevel::High,
            (LogicLevel::Low, LogicLevel::Low) => LogicLevel::Low,
            _ => LogicLevel::Invalid,
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

impl BitAnd for LogicLevels {
    type Output = LogicLevels;

    fn bitand(self, rhs: Self) -> Self::Output {
        LogicLevels(self.0.into_iter().zip(rhs.0).map(|(l, r)| l & r).collect())
    }
}

impl BitOr for LogicLevels {
    type Output = LogicLevels;

    fn bitor(self, rhs: Self) -> Self::Output {
        LogicLevels(self.0.into_iter().zip(rhs.0).map(|(l, r)| l | r).collect())
    }
}

// --- logic gates

#[derive(Clone, Debug)]
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

#[derive(Debug)]
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

            LogicGateType::And {
                bit_width,
                num_inputs,
                ..
            } => Self::create_and_or_gate_nodes(self.pos, bit_width, num_inputs),

            LogicGateType::Or {
                bit_width,
                num_inputs,
                ..
            } => Self::create_and_or_gate_nodes(self.pos, bit_width, num_inputs),

            LogicGateType::Not { bit_width, .. } => {
                vec![
                    Node::new_in(self.pos - Vec2i(1, 0), bit_width),
                    Node::new_out(self.pos + Vec2i(1, 0), bit_width),
                ]
            }
        }
    }

    fn create_and_or_gate_nodes(pos: Vec2i, bit_width: i32, num_inputs: i32) -> Vec<Node> {
        let mut nodes = if num_inputs % 2 == 0 {
            let half = num_inputs / 2;

            (0..half)
                .flat_map(|i| {
                    [
                        Node::new_in(pos + Vec2i(-1, 1 + i), bit_width),
                        Node::new_in(pos + Vec2i(-1, -1 - i), bit_width),
                    ]
                })
                .collect::<Vec<_>>()
        } else {
            let floored_half = num_inputs / 2;

            (-floored_half..(floored_half + 1))
                .map(|i| Node::new_in(pos + Vec2i(-1, i), bit_width))
                .collect::<Vec<_>>()
        };

        nodes.push(Node::new_out(pos + Vec2i(1, 0), bit_width));

        nodes
    }

    pub fn process(
        &self,
        node_idxs: &[usize],
        sim_nodes: &[SimNode],
        sim_queue: &mut SimQueue,
        subtick: i32,
    ) {
        match &self.gate_type {
            LogicGateType::Circuit(_circuit) => todo!(),

            LogicGateType::Input { logic_levels, .. } => {
                sim_queue.push(QueueEvent(
                    subtick,
                    QueueEventType::UpdateNode(node_idxs[0], logic_levels.clone()),
                ));
            }

            LogicGateType::Output { logic_levels, .. } => {
                // TODO separate state from LogicGate
                // *logic_levels = sim_nodes[node_idxs[0]].logic_levels.clone()
            }

            LogicGateType::And {
                num_inputs,
                propagation_delay,
                ..
            } => {
                let res = &node_idxs[0..*num_inputs as usize]
                    .iter()
                    .map(|idx| sim_nodes[*idx].logic_levels.clone())
                    .reduce(|acc, logic_level| acc & logic_level)
                    .unwrap();

                sim_queue.push(QueueEvent(
                    subtick + propagation_delay,
                    QueueEventType::UpdateNode(*node_idxs.last().unwrap(), res.clone()),
                ));
            }

            LogicGateType::Or {
                num_inputs,
                propagation_delay,
                ..
            } => {
                let res = &node_idxs[0..*num_inputs as usize]
                    .iter()
                    .map(|idx| sim_nodes[*idx].logic_levels.clone())
                    .reduce(|acc, logic_level| acc | logic_level)
                    .unwrap();

                sim_queue.push(QueueEvent(
                    subtick + propagation_delay,
                    QueueEventType::UpdateNode(*node_idxs.last().unwrap(), res.clone()),
                ));
            }

            LogicGateType::Not {
                propagation_delay, ..
            } => {
                sim_queue.push(QueueEvent(
                    subtick + propagation_delay,
                    QueueEventType::UpdateNode(
                        node_idxs[1],
                        !sim_nodes[node_idxs[0] as usize].logic_levels.clone(),
                    ),
                ));
            }
        }
    }
}

// ---

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct SimNode {
    pub logic_levels: LogicLevels,
}

impl SimNode {
    pub fn new(bit_width: i32) -> Self {
        SimNode {
            logic_levels: LogicLevels::filled(bit_width, LogicLevel::Invalid),
        }
    }
}

type SimQueue = BinaryHeap<QueueEvent>;

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

struct Sim<'a> {
    pub circuit: &'a Circuit,

    pub sim_nodes: Vec<SimNode>,
    // pub input_sim_node_ids: Vec<usize>,
    pub input_gate_idxs: Vec<usize>,

    pub sim_node_idxs_by_gate_idx: Vec<Vec<usize>>,
    pub affecting_gate_idxs_by_sim_node_idx: HashMap<usize, Vec<usize>>,
}

impl<'a> Sim<'a> {
    pub fn new(circuit: &'a Circuit) -> Result<Self, String> {
        let gates = &circuit.gates;
        let wires = &circuit.wires;

        let mut node_idx_counter = 0;
        let mut dsu = Dsu::<usize>::new();
        let mut gate_nodes = Vec::<Node>::new();
        let mut wire_nodes = Vec::<Node>::new();

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
                gate_nodes.push(node.clone());

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
                wire_nodes.push(node);

                node_idxs.push(node_idx);
            }

            // connect nodes within the same wire
            for i in &node_idxs[1..] {
                dsu.union(node_idxs[0], *i);
            }
        }

        let all_nodes = [gate_nodes.clone(), wire_nodes.clone()].concat();

        // connect nodes
        for i in 0..all_nodes.len() {
            for j in 0..i {
                let node_i = &all_nodes[i];
                let node_j = &all_nodes[j];

                // connect two nodes together if they share the same position
                if node_i.pos != node_j.pos {
                    continue;
                }

                if node_i.bit_width != node_j.bit_width {
                    return Err(format!("mismatched node bit width at {}", node_i.pos));
                }

                dsu.union(i, j);
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

        let mut affecting_gate_idxs_by_sim_node_idx = HashMap::new();

        for (node_idx, gate_idx) in &gate_idxs_by_node_idx {
            let node = &gate_nodes[*node_idx];
            let sim_node_idx = sim_node_idxs_by_node_idx[*node_idx];

            match node.node_type {
                NodeType::In | NodeType::InOut => affecting_gate_idxs_by_sim_node_idx
                    .entry(sim_node_idx)
                    .or_insert(Vec::new())
                    .push(*gate_idx),

                NodeType::Out => (),
            }
        }

        println!("gate idx -> gate");
        for i in 0..gates.len() {
            println!("{} -> {:?}", i, gates[i]);
        }

        println!("gate idx -> [node idx]");
        for i in 0..node_idxs_by_gate_idx.len() {
            println!("{} -> {:?}", i, node_idxs_by_gate_idx[i]);
        }

        println!("node idx -> gate idx");
        for i in 0..gate_idxs_by_node_idx.len() {
            println!("{} -> {}", i, gate_idxs_by_node_idx[&i]);
        }

        println!("node idx -> sim node idx");
        for i in 0..node_idx_counter {
            println!("{} -> {}", i, sim_node_idxs_by_node_idx[i as usize]);
        }

        // create simulation nodes
        let sim_nodes = (0..sim_node_idx_counter)
            .map(|node| SimNode::new(1))
            .collect::<Vec<_>>();

        // todo!();
        let sim_node_idxs_by_gate_idx = node_idxs_by_gate_idx
            .iter()
            .map(|node_idxs| {
                node_idxs
                    .iter()
                    .map(|node_idx| sim_node_idxs_by_node_idx[*node_idx])
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // create the struct
        Ok(Sim {
            circuit,
            sim_nodes,
            input_gate_idxs,
            sim_node_idxs_by_gate_idx,
            affecting_gate_idxs_by_sim_node_idx,
        })
    }

    pub fn tick(&mut self) {
        // a priority queue of (subtick, update event)
        let mut sim_queue = SimQueue::new();

        // push all input gates into the queue
        for gate_idx in &self.input_gate_idxs {
            sim_queue.push(QueueEvent(0, QueueEventType::UpdateGate(*gate_idx)));
        }

        const MAX_ITERS: i32 = 100;
        let mut iter = 0;

        // loop while there is something in the queue to proceed
        while !sim_queue.is_empty() && iter < MAX_ITERS {
            iter += 1;

            let curr_subtick = sim_queue.peek().unwrap().0;

            // retrieve all to-be-processed queue items
            let mut to_update = Vec::new();

            while sim_queue.peek().is_some_and(|item| item.0 == curr_subtick) {
                to_update.push(sim_queue.pop().unwrap().1);
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

                        let sim_node_idxs = &self.sim_node_idxs_by_gate_idx[gate_idx];
                        // let sim_node_idxs = &self.node_idxs_by_gate_idx[gate_idx];
                        gate.process(sim_node_idxs, &self.sim_nodes, &mut sim_queue, curr_subtick);
                    }

                    QueueEventType::UpdateNode(sim_node_idx, logic_levels) => {
                        self.sim_nodes[sim_node_idx].logic_levels = logic_levels;

                        let affecting_gate_idxs =
                            self.affecting_gate_idxs_by_sim_node_idx.get(&sim_node_idx);

                        let Some(gate_idxs) = affecting_gate_idxs else {
                            continue;
                        };

                        for gate_idx in gate_idxs {
                            sim_queue.push(QueueEvent(
                                curr_subtick,
                                QueueEventType::UpdateGate(*gate_idx),
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut circuit = Circuit::new();

    let input_1 = LogicGate {
        gate_type: LogicGateType::Input {
            bit_width: 1,
            logic_levels: LogicLevels::filled(1, LogicLevel::High),
        },
        pos: Vec2i(0, 1),
    };

    let input_2 = LogicGate {
        gate_type: LogicGateType::Input {
            bit_width: 1,
            logic_levels: LogicLevels::filled(1, LogicLevel::Low),
        },
        pos: Vec2i(0, -1),
    };

    let not = LogicGate {
        gate_type: LogicGateType::Not {
            bit_width: 1,
            propagation_delay: 1,
        },
        pos: Vec2i(4, -1),
    };

    let and = LogicGate {
        gate_type: LogicGateType::And {
            bit_width: 1,
            num_inputs: 2,
            propagation_delay: 1,
        },
        pos: Vec2i(7, 0),
    };

    let output = LogicGate {
        gate_type: LogicGateType::Output {
            bit_width: 1,
            logic_levels: LogicLevels::filled(1, LogicLevel::Invalid),
        },
        pos: Vec2i(10, 0),
    };

    let w_1 = Wire {
        pos1: Vec2i(1, 1),
        pos2: Vec2i(6, 1),
        bit_width: 1,
    };

    let w_2 = Wire {
        pos1: Vec2i(1, -1),
        pos2: Vec2i(3, -1),
        bit_width: 1,
    };

    let w_3 = Wire {
        pos1: Vec2i(5, -1),
        pos2: Vec2i(6, -1),
        bit_width: 1,
    };

    let w_4 = Wire {
        pos1: Vec2i(8, 0),
        pos2: Vec2i(9, 0),
        bit_width: 1,
    };

    circuit.gates.push(input_1);
    circuit.gates.push(input_2);
    circuit.gates.push(not);
    circuit.gates.push(and);
    circuit.gates.push(output);

    circuit.wires.push(w_1);
    circuit.wires.push(w_2);
    circuit.wires.push(w_3);
    circuit.wires.push(w_4);

    let mut sim = Sim::new(&circuit)?;

    sim.tick();

    println!();
    for sim_node in sim.sim_nodes {
        println!("{:?}", sim_node);
    }

    Ok(())
}
