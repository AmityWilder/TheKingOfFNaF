use crate::game_state::{Camera, Duct, State, Vent};
use std::{
    collections::LinkedList,
    thread::sleep,
    time::{Duration, Instant},
};
use vidivici::{
    IVec2, VirtualKey, simulate_key_down, simulate_key_up, simulate_mouse_down,
    simulate_mouse_goto, simulate_mouse_up,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameStateDelta {
    Time(u16),
    State(State),
    NextFFShow(u16),
    OfficeYaw(f64),
    IsNmbbStanding(bool),
    Camera(Camera),
    VentSnare(Vent),
    ClosedDuct(Duct),
    AudioLure(IVec2),
    VentilationResetNeeded(bool),
    FlashlightOn(bool),
    DoorClosed(u8, bool),
    NMBBStanding(bool),
    VirtualKeyInput { key: VirtualKey, is_down: bool },
    VirtualMouseMove { pos: IVec2 },
    VirtualMouseButton { input_press: bool },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeltaRecord {
    pub timestamp: Instant,
    pub change: GameStateDelta,
}

impl DeltaRecord {
    pub fn new(change: GameStateDelta) -> Self {
        Self {
            timestamp: Instant::now(),
            change,
        }
    }
}

#[derive(Debug)]
pub struct GameStateHistory<const BLOCK_CAP: usize> {
    data: LinkedList<Vec<DeltaRecord>>,
}

impl<const BLOCK_CAP: usize> GameStateHistory<BLOCK_CAP> {
    pub const fn new() -> Self {
        Self {
            data: LinkedList::new(),
        }
    }

    pub fn push(&mut self, delta: GameStateDelta) {
        match self.data.back_mut() {
            Some(cur_block) if cur_block.len() < BLOCK_CAP => cur_block,
            _ => self.data.push_back_mut(Vec::with_capacity(BLOCK_CAP)),
        }
        .push(DeltaRecord::new(delta));
    }

    pub fn get(&self, index: usize) -> Option<&DeltaRecord> {
        let (inter, intra) = (index / BLOCK_CAP, index % BLOCK_CAP);
        self.data
            .iter()
            .nth(inter)
            .and_then(|block| block.get(intra))
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &DeltaRecord> + Clone {
        self.data.iter().flatten()
    }

    pub fn len(&self) -> usize {
        (self.data.len().saturating_sub(1)) * BLOCK_CAP
            + self.data.back().map_or(0, |block| block.len())
    }

    pub fn is_empty(&self) -> bool {
        self.data
            .back()
            .is_none_or(|block| block.is_empty() && self.data.len() == 1)
    }

    pub fn sim_key_down(&mut self, key: VirtualKey) {
        self.push(GameStateDelta::VirtualKeyInput { key, is_down: true });
        simulate_key_down(key);
    }

    pub fn sim_key_up(&mut self, key: VirtualKey) {
        self.push(GameStateDelta::VirtualKeyInput {
            key,
            is_down: false,
        });
        simulate_key_up(key);
    }

    pub fn sim_key_tap(&mut self, key: VirtualKey) {
        self.sim_key_down(key);
        sleep(Duration::from_millis(8));
        self.sim_key_up(key);
    }

    pub fn sim_mouse_goto(&mut self, pos: IVec2) {
        self.push(GameStateDelta::VirtualMouseMove { pos });
        simulate_mouse_goto(pos);
    }

    pub fn sim_mouse_down(&mut self) {
        self.push(GameStateDelta::VirtualMouseButton { input_press: true });
        simulate_mouse_down();
    }

    pub fn sim_mouse_up(&mut self) {
        self.push(GameStateDelta::VirtualMouseButton { input_press: false });
        simulate_mouse_up();
    }

    pub fn sim_mouse_click(&mut self) {
        self.sim_mouse_down();
        sleep(Duration::from_millis(8));
        self.sim_mouse_up();
    }

    pub fn sim_mouse_click_at(&mut self, p: IVec2) {
        self.sim_mouse_goto(p);
        self.sim_mouse_click();
    }
}
