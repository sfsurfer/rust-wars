use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, PartialEq)]
pub struct Player {
    pub index: u32,
    pub color: u32,
    pub(crate) territories: Vec<u32>,
}
impl Player {
    pub fn capture_territory(&mut self, territory_index: u32) -> () {
        self.territories.push(territory_index);
    }
    pub fn is_eliminated(&self) -> bool {
        self.territories.is_empty()
    }
}
