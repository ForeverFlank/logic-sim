use std::{
    collections::BinaryHeap,
    ops::{Add, Sub},
    rc::Rc,
};

use crate::dsu::Dsu;

mod dsu;

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum LogicLevel {
    Low,
    High,
    Z,
}

type LogicLevels = Vec<LogicLevel>;

// #[derive(Clone, PartialEq, Eq)]
// struct BitSlice {
//     pub bits: Vec<i32>,
// }

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
    pub fn get_nodes(&self) -> Vec<LogicGateNode> {
        match self.gate_type {
            LogicGateType::Circuit(ref circuit) => todo!(),

            LogicGateType::Input { bit_width } => {
                vec![LogicGateNode::new(self.pos + Vec2i(1, 0), bit_width)]
            }

            LogicGateType::Output { bit_width } => {
                vec![LogicGateNode::new(self.pos - Vec2i(1, 0), bit_width)]
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
                    LogicGateNode::new(self.pos - Vec2i(1, 0), bit_width),
                    LogicGateNode::new(self.pos + Vec2i(1, 0), bit_width),
                ]
            }
        }
    }
}

#[derive(PartialEq, Eq)]
struct Wire {
    pub pos1: Vec2i,
    pub pos2: Vec2i,
    pub bit_width: i32,
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

#[derive(PartialEq, Eq)]
struct LogicGateNode {
    pub pos: Vec2i,
    pub logic_levels: Vec<LogicLevel>,
}

impl LogicGateNode {
    pub fn new(pos: Vec2i, bit_width: i32) -> Self {
        LogicGateNode {
            pos,
            logic_levels: [0..bit_width - 1].iter().map(|_| LogicLevel::Z).collect(),
        }
    }
}

struct Simulation {
    pub circuit: Box<Circuit>,

    // pub nodes: Vec<LogicGateNode>,
    pub node_src_gate_indices: Vec<usize>,
    pub input_node_indices: Vec<usize>,

    pub sim_node_states: Vec<LogicLevels>,

    pub wire_src_sim_node_state_indices: Vec<usize>,
    pub node_src_sim_node_state_indices: Vec<usize>,
}

// struct SimulationNode {
//     pub state: LogicLevels,
// }

impl Simulation {
    pub fn new(circuit: Circuit) -> Self {
        let (node_src_gate_indices, nodes): (Vec<usize>, Vec<LogicGateNode>) = circuit
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

        let dsu = Dsu::new();

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
