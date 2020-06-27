extern crate web_sys;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Territory {
    pub(crate) vertices: Vec<u32>,
    pub center: u32,
    pub color: u32,
    pub troops: u32,
    pub(crate) state: TerritoryState,
    pub(crate) neighbors: Vec<u32>,
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, PartialEq)]
pub enum TerritoryState {
    Dormant = 0,
    Selected = 1,
    Targeted = 2,
    Highlighted = 3
}

#[wasm_bindgen]
impl Territory {
    pub fn is_selected(&self) -> bool { self.state == TerritoryState::Selected }
    pub fn is_targeted(&self) -> bool { self.state == TerritoryState::Targeted }
    pub fn is_highlighted(&self) -> bool {
        self.state == TerritoryState::Highlighted ||
            self.is_selected() || self.is_targeted()
    }
    pub fn set_troops(&mut self, troops: u32) -> () { self.troops = troops }
    pub fn add_troops(&mut self, troops: u32) -> () { self.troops = self.troops + troops }
    pub fn sub_troops(&mut self, troops: u32) -> () {
        let new_troops = self.troops - troops;
        self.troops = new_troops
    }
    pub fn vertices(&self) -> *const u32 { self.vertices.as_ptr() }
    pub fn troops(&self) -> u32 { self.troops }
}
