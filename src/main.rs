use std::{
    ops::{Add, Sub},
    rc::Rc,
};

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
}

// #[derive(Clone, PartialEq, Eq)]
// struct BitSlice {
//     pub bits: Vec<i32>,
// }

type BitSlice = Vec<i32>;

#[derive(PartialEq, Eq)]
struct PackedLogicLevels {
    pub logic_levels: Vec<LogicLevel>,
}

impl PackedLogicLevels {
    pub fn from_bit_slice(bit_slice: &BitSlice) -> Self {
        PackedLogicLevels {
            logic_levels: bit_slice.iter().map(|_| LogicLevel::Low).collect(),
        }
    }

    pub fn get_state_at_bit(&self, bit_slice: BitSlice, bit: i32) -> Option<LogicLevel> {
        let idx = bit_slice.iter().find(|b| **b == bit)?;
        self.logic_levels.get(*idx as usize).copied()
    }
}

#[derive(Clone)]
enum LogicGateType {
    Circuit(Rc<Circuit>),
    Input {
        bit_slice: BitSlice,
    },
    Output {
        bit_slice: BitSlice,
    },
    And {
        bit_slice: BitSlice,
        num_inputs: i32,
        propagation_delay: i32,
    },
    Or {
        bit_slice: BitSlice,
        num_inputs: i32,
        propagation_delay: i32,
    },
    Not {
        bit_slice: BitSlice,
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

            LogicGateType::Input {
                bit_slice: ref bits,
            } => {
                vec![Node::new(self.pos + Vec2i(1, 0), bits.clone())]
            }

            LogicGateType::Output {
                bit_slice: ref bits,
            } => {
                vec![Node::new(self.pos - Vec2i(1, 0), bits.clone())]
            }

            LogicGateType::And {
                ref bit_slice,
                num_inputs,
                propagation_delay,
            } => todo!(),

            LogicGateType::Or {
                ref bit_slice,
                num_inputs,
                propagation_delay,
            } => todo!(),

            LogicGateType::Not {
                ref bit_slice,
                propagation_delay,
            } => {
                vec![
                    Node::new(self.pos - Vec2i(1, 0), bit_slice.clone()),
                    Node::new(self.pos + Vec2i(1, 0), bit_slice.clone()),
                ]
            }
        }
    }
}

#[derive(PartialEq, Eq)]
struct Wire {
    pub pos1: Vec2i,
    pub pos2: Vec2i,
    pub bit_slice: BitSlice,
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
struct Node {
    pub pos: Vec2i,
    pub value: PackedLogicLevels,
}

impl Node {
    pub fn new(pos: Vec2i, bits: BitSlice) -> Self {
        Node {
            pos,
            value: PackedLogicLevels {
                logic_levels: bits.iter().map(|_| LogicLevel::Low).collect(),
            },
        }
    }
}

struct Simulation {
    pub circuit: Box<Circuit>,
    pub nodes: Vec<Node>,
    pub wire_states: Vec<PackedLogicLevels>,
}

impl Simulation {
    pub fn new(circuit: Circuit) -> Self {
        let nodes = circuit
            .gates
            .iter()
            .flat_map(|gate| gate.get_nodes())
            .collect();

        let wire_states = circuit
            .wires
            .iter()
            .map(|wire| PackedLogicLevels::from_bit_slice(&wire.bit_slice))
            .collect();

        Simulation {
            circuit: circuit.into(),
            nodes,
            wire_states,
        }
    }

    pub fn tick(&self) -> Self {}
}

fn main() {
    let mut circuit = Circuit::new();

    let input = LogicGate {
        gate_type: LogicGateType::Input { bit_slice: vec![0] },
        pos: Vec2i(0, 0),
    };

    let not = LogicGate {
        gate_type: LogicGateType::Not {
            bit_slice: vec![0],
            propagation_delay: 1,
        },
        pos: Vec2i(5, 0),
    };

    let output = LogicGate {
        gate_type: LogicGateType::Output { bit_slice: vec![0] },
        pos: Vec2i(10, 0),
    };

    let w1 = Wire {
        pos1: Vec2i(1, 0),
        pos2: Vec2i(4, 0),
        bit_slice: vec![0],
    };

    let w2 = Wire {
        pos1: Vec2i(6, 0),
        pos2: Vec2i(9, 0),
        bit_slice: vec![0],
    };

    circuit.gates.push(input);
    circuit.gates.push(not);
    circuit.gates.push(output);

    circuit.wires.push(w1);
    circuit.wires.push(w2);

    let sim = Simulation::new(circuit);
}
