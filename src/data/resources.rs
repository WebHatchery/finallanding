//! Stockpiled resources and the fixed-size bag used to count them.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Every resource the colony can stockpile. Power is a flow and lives in the
/// colony power budget instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resource {
    Food,
    Meals,
    Salvage,
    Fibre,
    Stone,
    Metal,
    Components,
    Medicine,
    Relics,
}

pub const RESOURCE_COUNT: usize = 9;

impl Resource {
    pub const ALL: [Resource; RESOURCE_COUNT] = [
        Resource::Food,
        Resource::Meals,
        Resource::Salvage,
        Resource::Fibre,
        Resource::Stone,
        Resource::Metal,
        Resource::Components,
        Resource::Medicine,
        Resource::Relics,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn key(self) -> &'static str {
        match self {
            Resource::Food => "food",
            Resource::Meals => "meals",
            Resource::Salvage => "salvage",
            Resource::Fibre => "fibre",
            Resource::Stone => "stone",
            Resource::Metal => "metal",
            Resource::Components => "components",
            Resource::Medicine => "medicine",
            Resource::Relics => "relics",
        }
    }

    /// Building and crafting materials, as opposed to consumables.
    pub fn is_material(self) -> bool {
        !matches!(self, Resource::Food | Resource::Meals | Resource::Medicine)
    }
}

/// Fixed-size counts for every resource; cheap to copy and serialize.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResourceBag {
    pub amounts: [f32; RESOURCE_COUNT],
}

impl ResourceBag {
    pub fn from_map(map: &BTreeMap<Resource, f32>) -> Self {
        let mut bag = Self::default();
        for (resource, amount) in map {
            bag.amounts[resource.index()] = *amount;
        }
        bag
    }

    pub fn get(&self, resource: Resource) -> f32 {
        self.amounts[resource.index()]
    }

    pub fn set(&mut self, resource: Resource, amount: f32) {
        self.amounts[resource.index()] = amount.max(0.0);
    }

    pub fn add(&mut self, resource: Resource, amount: f32) {
        let slot = &mut self.amounts[resource.index()];
        *slot = (*slot + amount).max(0.0);
    }

    /// Remove up to `amount`, returning what was actually taken.
    pub fn take(&mut self, resource: Resource, amount: f32) -> f32 {
        let slot = &mut self.amounts[resource.index()];
        let taken = amount.min(*slot).max(0.0);
        *slot -= taken;
        taken
    }

    pub fn covers(&self, other: &ResourceBag) -> bool {
        Resource::ALL
            .iter()
            .all(|resource| self.get(*resource) + 0.001 >= other.get(*resource))
    }

    pub fn subtract(&mut self, other: &ResourceBag) {
        for resource in Resource::ALL {
            self.add(resource, -other.get(resource));
        }
    }

    pub fn add_bag(&mut self, other: &ResourceBag) {
        for resource in Resource::ALL {
            self.add(resource, other.get(resource));
        }
    }

    pub fn scaled(&self, factor: f32) -> ResourceBag {
        let mut bag = *self;
        for amount in bag.amounts.iter_mut() {
            *amount *= factor;
        }
        bag
    }

    pub fn total(&self) -> f32 {
        self.amounts.iter().sum()
    }

    pub fn is_empty(&self) -> bool {
        self.total() <= 0.001
    }

    /// Amount still missing before `self` reaches `target`, per resource.
    pub fn shortfall(&self, target: &ResourceBag) -> ResourceBag {
        let mut missing = ResourceBag::default();
        for resource in Resource::ALL {
            missing.set(resource, target.get(resource) - self.get(resource));
        }
        missing
    }

    pub fn nonzero(&self) -> impl Iterator<Item = (Resource, f32)> + '_ {
        Resource::ALL
            .into_iter()
            .map(|resource| (resource, self.get(resource)))
            .filter(|(_, amount)| *amount > 0.001)
    }
}
