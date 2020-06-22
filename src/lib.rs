mod utils;

use rand::prelude::*;
use std::collections::HashMap;
use std::ops::Range;
use wasm_bindgen::prelude::*;
use wasm_bindgen::__rt::std::thread::current;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[derive(Clone, PartialOrd, PartialEq)]
pub enum TurnPhase {
    Place = 0,
    Attack = 1,
    Fortify = 2
}

#[wasm_bindgen]
#[derive(Clone, PartialEq)]
pub struct Player {
    index: u32,
    color: u32,
    territories: Vec<u32>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Turn {
    player_index: u32,
    phase: TurnPhase,
    new_troops: u32,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Game {
    map: Map,
    players: Vec<Player>,
    turn: Turn
}

#[wasm_bindgen]
impl Game {
    pub fn new() -> Game {
        let map = Map::new();

        let players = vec!(
            Player{ index: 0, color: 0xAA1111, territories: vec!() },
            Player{ index: 0, color: 0x11AA11, territories: vec!() }
        );

        let turn = Turn { player_index: 0, phase: TurnPhase::Place, new_troops: 0 };

        let mut game = Game {
            map,
            players,
            turn
        };
        game.assign_territories();
        game.update_colors();
        game.turn.new_troops = game.calc_troop_bonus() as u32;
        game
    }

    pub fn hit_troop_placement_limit(&self) -> bool {
        self.troops_staged_for_placement() >= self.troops_available_for_placement()
    }
    pub fn map_click_action(&mut self, territory: usize) -> bool {
        // TODO: Add all top level clicking logic here
        match self.turn.phase {
            TurnPhase::Place => {
                if self.on_player().territories.contains(&(territory as u32)) && !self.hit_troop_placement_limit(){
                    self.map.territories[territory].state = TerritoryState::Selected;
                    self.map.cache_troop_placement(territory, 1); // todo -> pass in have value cached
                    true
                } else { false }
            },
            TurnPhase::Attack => {
                if self.on_player().territories.contains(&(territory as u32)) {
                    if !self.on_player().territories.iter().any(|t| self.map.territories[*t as usize].is_selected()) {
                        self.map.territories[territory].state = TerritoryState::Selected;
                        self.map.territories[territory].neighbors.clone().iter().for_each(|t|
                            if !self.on_player().territories.contains(t) {
                                self.map.territories[*t as usize].state = TerritoryState::Highlighted;
                            }
                        );
                        true
                    } else if self.map.territories[territory].is_selected() {
                        self.unselect_all();
                        true
                    } else { false }
                } else if territory != self.map.background_index && self.map.territories[territory].is_highlighted() {
                    let owned_territories = self.on_player().clone().territories;
                    self.map.territories.iter_mut().enumerate().for_each(|t|
                        if !(owned_territories.contains(&(t.0 as u32))) {
                            t.1.state = TerritoryState::Dormant;
                        }
                    );
                    self.map.territories[territory].state = TerritoryState::Targeted;
                    true
                } else { false }
            },
            TurnPhase::Fortify => {
                if self.on_player().territories.contains(&(territory as u32)) {
                    if !self.on_player().territories.iter().any(|t| self.map.territories[*t as usize].is_selected()) {
                        self.map.territories[territory].state = TerritoryState::Selected;
                        self.map.territories[territory].neighbors.clone().iter().for_each(|t|
                            if self.on_player().territories.contains(t) {
                                self.map.territories[*t as usize].state = TerritoryState::Highlighted;
                            }
                        );
                        true
                    } else if self.map.territories[territory].is_selected() {
                        self.unselect_all();
                        true
                    } else if self.map.territories[territory].is_highlighted() {
                        self.map.territories[territory].state = TerritoryState::Targeted;
                        true
                    } else { false }
                } else { false }
            }
        }
    }
    pub fn clear_placement_cache(&mut self) -> () {
        self.map.troop_placement_cache.clear();
        self.map.territories.iter_mut().for_each(|t| t.state = TerritoryState::Dormant);
    }
    pub fn commit_placement_cache(&mut self) -> () {
        let troops_placed: usize = self.map.troop_placement_cache.values().sum();
        let updated_troops_available = self.turn.new_troops - troops_placed as u32;
        self.map.troop_placement_cache.clone().iter().for_each(|(territory, troops)| self.add_troops(territory, troops));
        self.unselect_all();
        self.map.troop_placement_cache.clear();
        self.turn.new_troops = updated_troops_available;
    }

    pub fn place_phase(&mut self) -> () {
        // TODO: Needs checks that this is allowed
        //       for now just move to attack whenever button is pressed
        self.turn.phase = TurnPhase::Place;
    }
    pub fn is_place_phase(&self) -> bool {
        self.turn.phase == TurnPhase::Place
    }
    pub fn attack_phase(&mut self) -> () {
        // TODO: Needs checks that this is allowed
        //       for now just move to attack whenever button is pressed
        self.turn.phase = TurnPhase::Attack;
    }
    pub fn fortify_phase(&mut self) -> () {
        // TODO: Needs checks that this is allowed
        //       for now just move to fortify whenever button is pressed
        self.turn.phase = TurnPhase::Fortify;
    }

    pub fn on_player_index(&self) -> usize {
        self.turn.player_index.clone() as usize
    }

    pub fn turn_phase(&self) -> TurnPhase {
        self.turn.phase.clone()
    }

    pub fn update_colors(&mut self) -> () {
        self.map.set_all_territory_colors(&self.players);
    }
    pub fn assign_territories(&mut self) -> () {
        let player_count = self.players.len();
        let mut unassigned: Vec<usize> = (0..self.map.territories.len()).collect();
        let mut counter: usize = 0;
        while unassigned.len() > 0 {
            let next_index = 0; //thread_rng().gen_range(0,unassigned.len() - 1);
            let next_territory = unassigned.remove(next_index);
            self.players[counter % player_count].territories.push(next_territory as u32);
            counter = counter + 1;
        };
    }
    pub fn assign_territory(&mut self, territory: u32, player_index: u32) -> () {
        for (i,player) in self.players.clone().iter().enumerate() {
            if i != player_index as usize {
                self.players[i].territories.retain(|x| x != &territory)
            } else if !player.territories.contains(&territory) {
                self.players[i].territories.push(territory)
            }
        }
    }

    pub fn get_map(&self) -> Map { self.map.clone() }

    pub fn init_turn(&mut self) -> () {
        self.turn.player_index = (self.turn.player_index + 1) % self.players.len() as u32;
        let troops = self.calc_troop_bonus() as u32;
        self.turn.new_troops = troops;
        self.turn.phase = TurnPhase::Place;
    }
    pub fn troops_available_for_placement(&self) -> usize {
        self.turn.new_troops.clone() as usize
        // 5 as usize
    }
    pub fn troops_staged_for_placement(&self) -> usize {
        self.map.troop_placement_cache.values().sum()
        // 0 as usize
    }

}
impl Game {
    pub fn on_player(&self) -> &Player {
        &(self.players[self.on_player_index()])
    }
    pub fn add_troops(&mut self, target: &usize, troops: &usize) -> () {
        if self.on_player().territories.contains(&(*target as u32)) {
            let current_troops = &self.map.territories[*target].troops;
            let new_troops = *current_troops + (*troops as u32);
            self.map.territories[*target].troops = new_troops;
        }
    }
    pub fn unselect_all(&mut self) -> () {
        self.map.territories.iter_mut().for_each(|t| t.state = TerritoryState::Dormant);
    }
    pub fn calc_troop_bonus(&self) -> usize {
        5 as usize
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Map {
    width: u32,
    height: u32,
    territories: Vec<Territory>,
    background_color: usize,
    background_index: usize,
    troop_placement_cache: HashMap<usize, usize>
}

#[wasm_bindgen]
impl Map {
    pub fn new() -> Map {
        // Todo: This should fetch from DB
        Map::build_map()
    }
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn background_color(&self) -> usize { self.background_color }
    pub fn territories(&self) -> *const Territory { self.territories.as_ptr() }
    pub fn vertices(&self) -> Vec<u32> {
       self.territories.iter().flat_map(|x| x.clone().vertices ).collect()
    }
    pub fn territory_count(&self) -> usize {
        self.territories.len()
    }
    pub fn vertices_for(&self, index: usize) -> Vec<u32> {
        self.territories[index].vertices.clone()
    }
    pub fn centers(&self) -> Vec<u32> {
        self.territories.iter().map(|x| x.center).collect()
    }
    pub fn bg_color_for(&self, index: usize) -> usize {
        self.background_color + index + 1
    }

    pub fn set_color_for(&mut self, index: usize, color: usize) -> () {
        self.territories[index].color = color as u32;
    }
    // todo: convert to owner color, default neutral
    pub fn color_for(&self, index: usize) -> u32 {
        self.territories[index].color.clone()
    }

    pub fn territory_with_color(&self, color: usize) -> usize {
        *self.territories.iter().enumerate().find(|(i, t)|
            color - self.background_color - 1 == *i
        ).map(|x| x.0).get_or_insert(self.background_index)
    }

    pub fn troops(&self) -> Vec<u32> {
        self.territories.iter().enumerate().map(|x| self.troops_to_display(x)).collect()
    }

    pub fn is_selected(&self, i: usize) -> bool { self.territories[i].is_selected() }
    pub fn is_targeted(&self, i: usize) -> bool { self.territories[i].is_targeted() }
    pub fn is_highlighted(&self, i: usize) -> bool { self.territories[i].is_highlighted() }

    pub fn movement_eminent(&self) -> bool {
        self.territories.iter().any(|t| t.is_selected()) &&
            self.territories.iter().any(|t| t.is_targeted())
    }

    // unsafe
    pub fn get_movement_arrow_start(&self) -> usize {
        self.territories.iter().find(|t| t.is_selected()).unwrap().center as usize
    }
    // unsafe
    pub fn get_movement_arrow_end(&self) -> usize {
        self.territories.iter().find(|t| t.is_targeted()).unwrap().center as usize
    }

    pub fn find_selected_index(&self) -> Option<usize> {
        self.territories.iter().position(|t| t.state == TerritoryState::Selected)
    }
    pub fn select_as_source(&mut self, index: usize) -> bool {
        let selected = self.find_selected_index();
        if selected == Some(index) {
            self.territories[index].state = TerritoryState::Dormant;
            true
        } else if selected.is_some() {
            false
        } else {
            self.territories[index].state = TerritoryState::Selected;
            true
        }
    }

    pub fn select_as_place_target(&mut self, index: usize) -> bool { true }
    pub fn select_as_attack_target(&mut self, index: usize) -> bool { true }
    pub fn select_as_fortify_target(&mut self, index: usize) -> bool { true }
    pub fn unselect(&mut self, index: usize) -> () {
        self.territories[index].state = TerritoryState::Dormant
    }
    pub fn can_attack(self, attacker: usize, target: usize) -> bool {
        let x = target as u32;
        self.territories[attacker].neighbors.contains(&x)
    }

    pub fn cache_troop_placement(&mut self, index: usize, troops: usize) -> usize {
        self.territories[index].state = TerritoryState::Selected;
        self.troop_placement_cache(&index, troops)
    }

    pub fn commit_troop_placement(&mut self) -> () {
        for (i,t) in &self.troop_placement_cache {
            self.territories[*i].troops = *t as u32;
            self.territories[*i].state = TerritoryState::Dormant;
        };
        self.troop_placement_cache.clear();
    }

}

impl Map {
    fn get_index(&self, row: u32, col: u32) -> usize {
        (row * self.width + col) as usize
    }
    fn find_selected(&self) -> Option<&Territory> {
        self.territories.iter().find(|&t| t.state == TerritoryState::Selected)
    }
    // Index of territory + 1, to account for outside = 0
    pub fn match_color_with_index(index: &usize, territory_blue: isize, click_blue: isize) -> bool {
        let sign = (128 - territory_blue).signum();
        let i = *index as isize;
        territory_blue + (sign * i as isize) == click_blue
    }
    pub fn troop_placement_cache(&mut self, index: &usize, troops: usize) -> usize {
        let new_troops: usize = self.troop_placement_cache.get(index).map(|x| x + troops).get_or_insert(troops).clone();
        self.troop_placement_cache.insert(*index, new_troops);
        self.territories[*index].state = TerritoryState::Selected;
        new_troops
    }
    pub fn set_all_territory_colors(&mut self, players: &Vec<Player>) -> () {
        for player in players {
            for territory in &player.territories {
                self.set_color_for(*territory as usize, player.color as usize);
            }
        }
    }
    fn troops_to_display(&self, indexed_territory: (usize, &Territory)) -> u32 {
        let cached = self.troop_placement_cache.get(&indexed_territory.0).get_or_insert(&0).clone();
        indexed_territory.1.troops as u32 + (cached as u32)
    }
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, PartialEq)]
pub enum TerritoryState {
    Dormant = 0,
    Selected = 1,
    Targeted = 2,
    Highlighted = 3 //TODO: Make this work so fortify will work
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Territory {
    vertices: Vec<u32>,
    center: u32,
    color: u32,
    troops: u32,
    state: TerritoryState,
    neighbors: Vec<u32>,
}

#[wasm_bindgen]
impl Territory {
    pub fn is_selected(&self) -> bool { self.state == TerritoryState::Selected }
    pub fn is_targeted(&self) -> bool { self.state == TerritoryState::Targeted }
    pub fn is_highlighted(&self) -> bool {
        self.state == TerritoryState::Highlighted ||
            self.is_selected() || self.is_targeted()
    }
    pub fn add_troops(&mut self, troops: u32) -> () { self.troops = self.troops + troops }
    pub fn sub_troops(&mut self, troops: u32) -> () {
        let new_troops = self.troops - troops ;
        self.troops = new_troops
    }
}

// TODO: Convert color to single u32. Needs lots of helper functions!!
// TODO After: each territory in country adds index -> to be used in selecting
//             territory associated by click using color on js canvas
// TODO After After: Add safeguards around which colors can be used (yagni)
#[derive(Clone)]
pub struct Country {
    territories: Vec<u32>,
    border_color: u32
}

#[wasm_bindgen]
impl Territory {
    pub fn vertices(&self) -> *const u32 { self.vertices.as_ptr() }
    pub fn troops(&self) -> u32 { self.troops }
}

impl Map {
    pub fn build_map() -> Map {
        let c1 = Country {
            territories: vec![0,1],
            border_color: 0xFF0000
        };
        let c2 = Country {
            territories: vec![2,3],
            border_color: 0x00FFCC
        };
        let t1 = Territory {
            vertices: vec![17,23,39,85,66,17],
            center: 52,
            color: c1.border_color.clone(),
            troops: 167,
            state: TerritoryState::Dormant,
            neighbors: vec!(1,2,3),
        };
        let t2 = Territory {
            vertices: vec![23,39,56,107,94,45,23],
            center: 75,
            color: c1.border_color.clone(),
            troops: 289,
            state: TerritoryState::Dormant,
            neighbors: vec!(0,2,4)
        };
        let t3 = Territory {
            vertices: vec![85,39,56,107,169,216,85],
            center: 104,
            color: c2.border_color.clone(),
            troops: 3,
            state: TerritoryState::Dormant,
            neighbors: vec!(0,1,3,4)
        };
        let t4 = Territory {
            vertices: vec![66,85,216,246,128,66],
            center: 148,
            color: c2.border_color.clone(),
            troops: 4,
            state: TerritoryState::Dormant,
            neighbors: vec!(0,2)
        };
        let t5 = Territory {
            vertices: vec![94,220,216,169,107,94],
            center: 156,
            color: c2.border_color.clone(),
            troops: 5,
            state: TerritoryState::Dormant,
            neighbors: vec!(1,2)
        };
        Map {
            width: 16,
            height: 16,
            territories: vec![t1,t2,t3,t4,t5],
            background_color: 0,
            background_index: 16777215,
            troop_placement_cache: HashMap::new()
        }
    }
}
