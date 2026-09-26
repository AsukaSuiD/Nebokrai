//! Identity-регистр фигур и монотонные счётчики ID `CServerRegion`. Исходный
//! владелец — `appserver/serverregion.h/.cpp`; точная пара `gameserver.exe`
//! + `GameServer.pdb`.
//!
//! Старые pointer-valued hash maps выражены registry identity: сами `CShape`
//! остаются у runtime owner-а и разрешаются через `ShapeResolver`. Это
//! сознательная смена формы API без копии shared/derived семантики. Player
//! registry сохраняет vector и first-erase, остальные map assignment —
//! уникальные ключи. `BTreeSet` используется только для identity lookup:
//! observable обход старого MSVC `stdext::hash_map` для startup name-cache
//! отдельно сортируется вызывающей стороной и не зависит от traversal
//! этого хранилища.

use std::collections::BTreeSet;

use nebokrai_shared::values::CGuid;

use super::geometry::{GOODS_TYPE, MONSTER_TYPE, NPC_TYPE, PLAYER_TYPE};
use crate::regions::ShapeIdentity;
use crate::regions::baseobject::CBaseObject;
use crate::regions::shape::{ShapeResolver, ShapeRuntimeFacts, ShapeView};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NextNpcId(i32);

impl NextNpcId {
    pub fn take(&mut self) -> i32 {
        let id = self.0;
        self.0 = self.0.wrapping_add(1);
        id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NextMonsterId(i32);

impl NextMonsterId {
    pub fn take(&mut self) -> i32 {
        let id = self.0;
        self.0 = self.0.wrapping_add(1);
        id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NextChildId(i32);

impl NextChildId {
    pub fn take(&mut self) -> i32 {
        let id = self.0;
        self.0 = self.0.wrapping_add(1);
        id
    }
}

impl Default for NextMonsterId {
    fn default() -> Self {
        Self(1)
    }
}

impl Default for NextNpcId {
    fn default() -> Self {
        Self(1)
    }
}

impl Default for NextChildId {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServerRegionRegistry {
    pub monsters: BTreeSet<i32>,
    pub players: Vec<i32>,
    pub npcs: BTreeSet<i32>,
    pub goods: BTreeSet<CGuid>,
    pub other_shapes: BTreeSet<i64>,
}

impl ServerRegionRegistry {
    pub fn add(&mut self, identity: ShapeIdentity, facts: ShapeRuntimeFacts) {
        match identity.object_type {
            MONSTER_TYPE if facts.monster.is_some() => {
                self.monsters.insert(identity.id);
            }
            PLAYER_TYPE if facts.is_player => self.players.push(identity.id),
            NPC_TYPE if facts.is_npc => {
                self.npcs.insert(identity.id);
            }
            GOODS_TYPE if facts.goods.is_some() => {
                self.goods.insert(identity.ex_id);
            }
            PLAYER_TYPE | NPC_TYPE | MONSTER_TYPE | GOODS_TYPE => {}
            _ => {
                self.other_shapes.insert(CBaseObject::get_hash_value(
                    identity.object_type,
                    identity.id,
                ));
            }
        }
    }

    pub fn remove(&mut self, identity: ShapeIdentity) {
        match identity.object_type {
            MONSTER_TYPE => {
                self.monsters.remove(&identity.id);
            }
            PLAYER_TYPE => {
                if let Some(index) = self.players.iter().position(|id| *id == identity.id) {
                    self.players.remove(index);
                }
            }
            NPC_TYPE => {
                self.npcs.remove(&identity.id);
            }
            GOODS_TYPE => {
                self.goods.remove(&identity.ex_id);
            }
            _ => {
                self.other_shapes.remove(&CBaseObject::get_hash_value(
                    identity.object_type,
                    identity.id,
                ));
            }
        }
    }

    pub fn contains(&self, identity: ShapeIdentity) -> bool {
        match identity.object_type {
            MONSTER_TYPE => self.monsters.contains(&identity.id),
            PLAYER_TYPE => self.players.contains(&identity.id),
            NPC_TYPE => self.npcs.contains(&identity.id),
            GOODS_TYPE => self.goods.contains(&identity.ex_id),
            _ => self.other_shapes.contains(&CBaseObject::get_hash_value(
                identity.object_type,
                identity.id,
            )),
        }
    }
}

pub struct RegisteredShapeResolver<'a, Resolver> {
    pub registry: &'a ServerRegionRegistry,
    pub resolver: &'a Resolver,
}

impl<Resolver: ShapeResolver> ShapeResolver for RegisteredShapeResolver<'_, Resolver> {
    fn resolve_shape(&self, identity: ShapeIdentity) -> Option<ShapeView> {
        if !self.registry.contains(identity) {
            return None;
        }
        self.resolver.resolve_shape(identity)
    }
}
