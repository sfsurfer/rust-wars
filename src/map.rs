pub mod country;
pub mod territory;

use std::collections::HashMap;
use wasm_bindgen::prelude::*;

pub use crate::player::Player;
pub use crate::map::territory::*;
pub use crate::map::country::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Map {
    pub width: u32,
    pub height: u32,
    pub(crate) territories: Vec<Territory>,
    pub background_color: usize,
    pub background_index: usize,
    pub(crate) troop_placement_cache: HashMap<usize, usize>
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

    pub fn color_for(&self, index: usize) -> u32 {
        self.territories[index].color.clone()
    }

    pub fn territory_with_color(&self, color: usize) -> usize {
        *self.territories.iter().enumerate().find(|(i, _)|
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
    // fn get_index(&self, row: u32, col: u32) -> usize {
    //     (row * self.width + col) as usize
    // }
    // fn find_selected(&self) -> Option<&Territory> {
    //     self.territories.iter().find(|&t| t.state == TerritoryState::Selected)
    // }

    // Index of map.territory + 1, to account for outside = 0
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

// TODO: Convert color to single u32. Needs lots of helper functions!!
// TODO After: each map.territory in country adds index -> to be used in selecting
//             map.territory associated by click using color on js canvas
// TODO After After: Add safeguards around which colors can be used (yagni)

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
