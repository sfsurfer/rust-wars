extern crate web_sys;
extern crate rand;

mod utils;

pub mod player;
pub mod map;

use wasm_bindgen::prelude::*;
use rand::Rng;
use rand::prelude::*;

pub use crate::map::*;
pub use crate::map::territory::*;
pub use crate::player::Player;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

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
    Fortify = 2,
    PostAttackFortify = 3,
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
    turn: Turn,
    rng: StdRng
}

#[wasm_bindgen]
impl Game {
    pub fn new() -> Game {
        utils::set_panic_hook();

        let seed: u64 = 1234;
        let rng: StdRng = rand::SeedableRng::seed_from_u64(seed);
        let map: Map = Map::new();

        let players = vec!(
            Player{ index: 0, color: 0xAA1111, territories: vec!() },
            Player{ index: 0, color: 0x11AA11, territories: vec!() }
        );

        let turn = Turn { player_index: 0, phase: TurnPhase::Place, new_troops: 0 };

        let mut game = Game {
            map,
            players,
            turn,
            rng
        };
        game.assign_territories();
        game.update_colors();
        game.turn.new_troops = game.calc_troop_bonus() as u32;
        game
    }

    pub fn is_over(&self) -> bool {
        let active_players: Vec<usize> = self.active_players();
        active_players.len() <= 1
    }
    pub fn active_players(&self) -> Vec<usize> {
        self.players.iter().filter(|p| !p.is_eliminated()).map(|p| p.index as usize).collect()
    }

    pub fn hit_troop_placement_limit(&self) -> bool { self.troops_available_for_placement() <= 0 }

    pub fn map_click_action(&mut self, territory: usize) -> bool {
        match self.turn.phase {
            TurnPhase::Place => {
                if self.on_player().territories.contains(&(territory as u32)) && !self.hit_troop_placement_limit() {
                    self.map.territories[territory].state = TerritoryState::Selected;
                    self.map.cache_troop_placement(territory); // todo -> pass in have value cached
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
            },
            TurnPhase::PostAttackFortify => false,
        }
    }
    pub fn get_troops_to_place(&self) -> usize { self.map.troops_to_place }
    pub fn set_troops_to_place(&mut self, t: usize) -> () { self.map.troops_to_place = t; }
    pub fn clear_placement_cache(&mut self) -> () {
        self.map.troop_placement_cache.clear();
        self.map.territories.iter_mut().for_each(|t| t.state = TerritoryState::Dormant);
    }

    pub fn new_troops(&self) -> u32 { self.turn.new_troops }
    pub fn troops_available_for_placement(&self) -> u32 {
        let uncommitted: u32 = self.turn.new_troops;
        let cached: usize = self.map.troop_placement_cache.values().sum();
        uncommitted - (cached as u32)
    }

    pub fn commit_placement_cache(&mut self) -> () {
        let troops_placed: usize = self.map.troop_placement_cache.values().sum();
        let updated_troops_available = self.turn.new_troops - troops_placed as u32;
        self.map.troop_placement_cache.clone().iter().for_each(|(territory, troops)| self.add_troops(territory, troops));
        self.unselect_all();
        self.map.troop_placement_cache.clear();
        self.turn.new_troops = updated_troops_available;
    }

    // TODO: These need checks that each is allowed
    //       for now just move to segment whenever button is pressed
    pub fn place_phase(&mut self) -> () { self.turn.phase = TurnPhase::Place; }
    pub fn attack_phase(&mut self) -> () { self.turn.phase = TurnPhase::Attack; }
    pub fn fortify_phase(&mut self) -> () { self.turn.phase = TurnPhase::Fortify; }

    pub fn is_place_phase(&self) -> bool { self.turn.phase == TurnPhase::Place }
    pub fn is_attack_phase(&self) -> bool { self.turn.phase == TurnPhase::Attack }
    pub fn is_fortify_phase(&self) -> bool {
        vec!(TurnPhase::Fortify, TurnPhase::PostAttackFortify).contains(&self.turn.phase)
    }

    pub fn target_selected(&self) -> bool {
        self.map.territories.iter().find(|t| t.is_targeted()).is_some()
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
    // pub fn troops_available_for_placement(&self) -> usize {
    //     self.turn.new_troops.clone() as usize
    // }
    pub fn troops_staged_for_placement(&self) -> usize {
        self.map.troop_placement_cache.values().sum()
    }
    pub fn troops_available_for_movement(&self) -> usize {
        let total_troops = self.map.territories.iter().find(|t| t.is_selected()).map(|t| t.troops);
        *(total_troops.map(|t| t as usize).get_or_insert(0))
    }

    // Todo: Figure out how to pass functions around
    // Todo: should stop at 3 left to attack
    pub fn attack_tail(&mut self) -> () {
        let selected_idx = self.selected_territory_index();
        let targeted_idx = self.targeted_territory_index();
        match (selected_idx, targeted_idx) {
            (Some(attacker), Some(defender)) => {
                let attack_dice = self.map.territories[attacker].troops - 1;
                let defend_dice = self.map.territories[defender].troops;
                let survivors = self.roll_all(attack_dice, defend_dice);
                if survivors.defend_dice == 0 { // Attacker won;
                    self.map.territories[attacker].troops = 1;
                    self.map.territories[defender].troops = survivors.attack_dice;
                    let player_idx = self.on_player_index().clone();
                    self.players.iter_mut().find(|p| p.territories.contains(&(defender as u32)))
                        .map(|p| p.territories.retain(|i| i != &(defender as u32)));
                    self.players[player_idx].capture_territory(defender as u32);
                    self.update_colors();
                } else {
                    self.map.territories[attacker].troops = survivors.attack_dice + 1;
                    self.map.territories[defender].troops = survivors.defend_dice;
                }
            },
            _ => ()
        };
        self.unselect_all();
    }
    // Returns troops available for transfer, or 0 if attacker lost
    pub fn attack_all(&mut self) -> () {
        // TODO: Refactor to something like... There's a small diff, though
        // match self.selected_territory_index() {
        //     Some(attacker) => {
        //         let attack_troops = self.map.territories[attacker].troops - 1;
        //         self.attack_with(attack_troops as usize);
        //     }
        //     _ => ()
        // }
        let selected_idx = self.selected_territory_index();
        let targeted_idx = self.targeted_territory_index();
        match (selected_idx, targeted_idx) {
            (Some(attacker), Some(defender)) => {
                let attack_dice = self.map.territories[attacker].troops - 1;
                let defend_dice = self.map.territories[defender].troops;
                let survivors = self.roll_all(attack_dice, defend_dice);
                if survivors.defend_dice == 0 { // Attacker won;
                    let remaining_troops = if survivors.attack_dice > 3 { survivors.attack_dice - 2 } else { 1 };
                    self.map.territories[attacker].troops = remaining_troops;
                    self.map.territories[defender].troops = std::cmp::min(survivors.attack_dice, 3);
                    let player_idx = self.on_player_index().clone();
                    self.players.iter_mut().find(|p| p.territories.contains(&(defender as u32)))
                        .map(|p| p.territories.retain(|i| i != &(defender as u32)));
                    self.players[player_idx].capture_territory(defender as u32);
                    self.update_colors();
                    log!("remaining_troops - 1 = {}", (remaining_troops - 1));
                    if remaining_troops <= 1 { self.unselect_all(); } else { self.turn.phase = TurnPhase::PostAttackFortify }

                } else {
                    self.map.territories[attacker].troops = survivors.attack_dice + 1;
                    self.map.territories[defender].troops = survivors.defend_dice;
                }
            },
            _ => ()
        }
    }
    pub fn attack_with(&mut self, troops: usize) -> () {
        let selected_idx = self.selected_territory_index();
        let targeted_idx = self.targeted_territory_index();
        match (selected_idx, targeted_idx) {
            (Some(attacker), Some(defender)) => {
                let attack_dice = troops as u32;
                let reserves = self.map.territories[attacker].troops - attack_dice;
                let defend_dice = self.map.territories[defender].troops;
                let survivors = self.roll_all(attack_dice, defend_dice);
                if survivors.defend_dice == 0 { // Attacker won;
                    let remaining_troops = if survivors.attack_dice > 3 { survivors.defend_dice - 2 } else { 1 } + reserves;
                    self.map.territories[attacker].troops = remaining_troops;
                    self.map.territories[defender].troops = std::cmp::min(survivors.attack_dice, 3);
                    let player_idx = self.on_player_index().clone();
                    self.players.iter_mut().find(|p| p.territories.contains(&(defender as u32)))
                        .map(|p| p.territories.retain(|i| i != &(defender as u32)));
                    self.players[player_idx].capture_territory(defender as u32);
                    self.update_colors();
                    if remaining_troops <= 1 { self.unselect_all(); } else { self.turn.phase = TurnPhase::PostAttackFortify }
                } else {
                    self.map.territories[attacker].troops = reserves + survivors.attack_dice;
                    self.map.territories[defender].troops = survivors.defend_dice;
                }
            },
            _ => ()
        }
    }
    pub fn fortify_all(&mut self) -> () {
        let selected_idx = self.selected_territory_index();
        let targeted_idx = self.targeted_territory_index();
        match (selected_idx, targeted_idx) {
            (Some(source), Some(destination)) => {
                let troops = self.map.territories[source].troops.clone() - 1;
                self.map.territories[source].sub_troops(troops as u32);
                self.map.territories[destination].add_troops(troops as u32);
                self.unselect_all();
            }
            _ => ()
        }
    }
    pub fn fortify_troops(&mut self, troops: usize) -> () {
        let selected_idx = self.selected_territory_index();
        let targeted_idx = self.targeted_territory_index();
        match (selected_idx, targeted_idx) {
            (Some(source), Some(destination)) => {
                self.map.territories[source].sub_troops(troops as u32);
                self.map.territories[destination].add_troops(troops as u32);
                if self.turn.phase == TurnPhase::PostAttackFortify {
                    self.turn.phase = TurnPhase::Attack;
                }
                self.unselect_all();
            }
            _ => ()
        }
    }
    pub fn unselect_all(&mut self) -> () {
        self.map.territories.iter_mut().for_each(|t| t.state = TerritoryState::Dormant);
    }
}

impl Game {
    pub fn on_player(&self) -> &Player {
        &(self.players[self.on_player_index()])
    }
    pub fn add_troops(&mut self, target: &usize, troops: &usize) -> () {
        // if self.on_player().territories.contains(&(*target as u32)) {
            let current_troops = &self.map.territories[*target].troops;
            let new_troops = *current_troops + (*troops as u32);
            self.map.territories[*target].troops = new_troops;
        // }
    }
    pub fn sub_troops(&mut self, target: &usize, troops: &usize) -> () {
        let current_troops = &self.map.territories[*target].troops;
        let new_troops = *current_troops - (*troops as u32); // TODO: Handle negatives
        self.map.territories[*target].troops = new_troops;
    }
    pub fn set_troops(&mut self, target: &usize, troops: &usize) -> () {
       self.map.territories[*target].troops = *troops as u32;
    }
    pub fn calc_troop_bonus(&self) -> usize {
        5 as usize
    }
    pub fn selected_territory_with_index(&self) -> Option<(usize, &Territory)> {
        self.map.territories.iter().enumerate().find(|t| (*t).1.is_selected())
    }
    pub fn targeted_territory_with_index(&self) -> Option<(usize, &Territory)> {
        self.map.territories.iter().enumerate().find(|t| (*t).1.is_targeted())
    }
    pub fn selected_territory_index(&self) -> Option<usize> {
        self.selected_territory_with_index().map(|t| t.0)
    }
    pub fn targeted_territory_index(&self) -> Option<usize> {
        self.targeted_territory_with_index().map(|t| t.0)
    }

    // Returns how many troops left: (attack, defense)
    fn roll_all(&mut self, mut attack_dice: u32, mut defend_dice: u32) -> AttackResults {
        while attack_dice > 0 && defend_dice > 0 {
            let losses = self.roll_dice(std::cmp::min(attack_dice, 3), std::cmp::min(defend_dice, 2));
            attack_dice -= losses.attack_dice;
            defend_dice -= losses.defend_dice;
        }
        AttackResults { attack_dice, defend_dice }
    }

    // Returns losses (thiw is counter to roll_all, which returns survivors, todo: rework to be consistent
    fn roll_dice(&mut self, attack_dice: u32, defense_dice: u32) -> AttackResults {
        // Returns (attack losses, defences losses)
        let mut attacks: Vec<u8> = vec![0; attack_dice as usize].iter_mut().map(|_| self.rng.gen_range(1,7)).collect();
        let mut defenses: Vec<u8> = vec![0; defense_dice as usize].iter_mut().map(|_| self.rng.gen_range(1,7)).collect();
        attacks.sort();
        defenses.sort();
        // log!("Attacks = {:?}", attacks);
        // log!("Defenses = {:?}", defenses);
        if attack_dice == 1 && defense_dice == 1 { AttackResults{ attack_dice: 1, defend_dice: 0 } }
        else { AttackResults { attack_dice: 1, defend_dice: 1 } }
    }

}

struct AttackResults {
    attack_dice: u32,
    defend_dice: u32
}

