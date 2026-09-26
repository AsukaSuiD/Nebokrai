//! Observable traversal-контракты запросов `CServerRegion`. Исходный
//! владелец — `appserver/serverregion.h/.cpp`; точная пара `gameserver.exe`
//! + `GameServer.pdb`.
//!
//! `legacy_msvc_npc_hash_traversal` воспроизводит обход старого MSVC
//! hash-хранилища NPC-кэша. Семейство ids/find/registered читает area-grid
//! и registry владельца без смены порядка: per-area фильтр исходного
//! `CArea::FindShapes(400)` подтверждается registry, так как area хранит
//! inherited socket ID и не знает о снятии регистрации.

use nebokrai_shared::values::CGuid;

use super::areagrid::get_area;
use super::geometry::{GOODS_TYPE, MONSTER_TYPE, NPC_TYPE, PLAYER_TYPE};
use super::registry::ServerRegionRegistry;
use crate::regions::ShapeIdentity;
use crate::regions::area::CArea;
use crate::regions::baseobject::CBaseObject;
use crate::regions::shape::{CShape, ShapeResolver, ShapeView};
use crate::replication::recipients::{ServerRegionRecipientArea, ServerRegionRecipientsSnapshot};

/// Воспроизводит только observable traversal `m_mNpcs` из decoder RVA
/// `0x000858F0`. MSVC `_Hash::insert` RVA `0x00081A60` начинает с mask/bucket
/// `1/1`, растит один bucket на каждые четыре элемента, группирует linked
/// list по bucket и держит signed `long` keys по возрастанию внутри группы.
pub fn legacy_msvc_npc_hash_traversal(ids: impl IntoIterator<Item = i32>) -> Vec<i32> {
    let mut ids: Vec<_> = ids.into_iter().collect();
    let mut mask = 1_u32;
    let mut bucket_count = 1_u32;
    let mut bucket_vector_len = 9_u32;

    for inserted in 0..ids.len() as u32 {
        if bucket_count <= inserted >> 2 {
            if bucket_count < bucket_vector_len - 1 {
                if mask < bucket_count {
                    mask = mask.wrapping_mul(2).wrapping_add(1);
                }
            } else {
                mask = bucket_vector_len.wrapping_mul(2).wrapping_sub(3);
                bucket_vector_len = bucket_vector_len.wrapping_mul(2).wrapping_sub(1);
            }
            bucket_count = bucket_count.wrapping_add(1);
        }
    }

    ids.sort_by_key(|id| {
        let mut bucket = (*id as u32 ^ 0xdead_beef) & mask;
        if bucket_count <= bucket {
            bucket = bucket.wrapping_sub(1 + (mask >> 1));
        }
        (bucket, *id)
    });
    ids
}

/// Per-area ядро фильтра `FindShapes(400)`: area отдаёт inherited socket ID
/// по RTTI, registry owning region-а подтверждает регистрацию. Порядок
/// следует `CArea::append_player_ids`.
pub fn append_registered_player_ids(
    registry: &ServerRegionRegistry,
    area: &CArea,
    destination: &mut Vec<i32>,
) {
    let mut area_player_ids = Vec::new();
    if !area.append_player_ids(&mut area_player_ids) {
        return;
    }
    for player_id in area_player_ids {
        if registry.contains(ShapeIdentity {
            object_type: PLAYER_TYPE,
            id: player_id,
            ex_id: CGuid::GUID_INVALID,
        }) {
            destination.push(player_id);
        }
    }
}

/// Собирает player IDs одной area без чтения их координат: исходный
/// `CArea::FindShapes(400)` использовал только RTTI и inherited socket ID.
pub fn find_player_ids_in_area(
    areas: &[CArea],
    area_x: i32,
    area_y: i32,
    x: i32,
    y: i32,
    registry: &ServerRegionRegistry,
    destination: &mut Vec<i32>,
) {
    let Some(area) = get_area(areas, area_x, area_y, x, y) else {
        return;
    };
    append_registered_player_ids(registry, area, destination);
}

/// Обходит все `CArea` в физическом row-major storage order и сохраняет
/// exact `FindShapes(400)` filtering через registry owning region-а.
pub fn find_all_player_ids(
    areas: &[CArea],
    registry: &ServerRegionRegistry,
    destination: &mut Vec<i32>,
) {
    for area in areas {
        append_registered_player_ids(registry, area, destination);
    }
}

/// Безопасно заменяет исходный `CArea::m_pFather`: пара принимается только
/// если area действительно принадлежит этому server-region.
pub fn find_player_ids_in_area_object(
    areas: &[CArea],
    area: &CArea,
    registry: &ServerRegionRegistry,
    destination: &mut Vec<i32>,
) -> bool {
    let Some(owned_area) = areas
        .iter()
        .find(|owned_area| std::ptr::eq(*owned_area, area))
    else {
        return false;
    };
    append_registered_player_ids(registry, owned_area, destination);
    true
}

/// Материализует per-area списки player identities для spatial snapshot:
/// порядок area — физический storage order, фильтр — общее per-area ядро.
pub fn recipients_snapshot(
    region_id: i32,
    area_x: i32,
    area_y: i32,
    areas: &[CArea],
    registry: &ServerRegionRegistry,
) -> ServerRegionRecipientsSnapshot {
    let recipient_areas = areas
        .iter()
        .map(|area| {
            let mut player_ids = Vec::new();
            append_registered_player_ids(registry, area, &mut player_ids);
            ServerRegionRecipientArea {
                x: area.x(),
                y: area.y(),
                player_ids,
            }
        })
        .collect();
    ServerRegionRecipientsSnapshot::from_parts(region_id, area_x, area_y, recipient_areas)
}

/// Exact `m_vPlayers` storage order, который Nation kick обходит
/// напрямую, не через area scan `FindAllPlayer`.
pub fn registered_player_ids(registry: &ServerRegionRegistry) -> Vec<i32> {
    registry.players.clone()
}

/// Owned identity snapshot для проверки полноты resolver-а перед
/// pointer-sensitive `OnGMMessage 0x7FC07` scan.
pub fn registered_shape_identities(registry: &ServerRegionRegistry) -> Vec<ShapeIdentity> {
    let mut identities = Vec::with_capacity(
        registry.monsters.len()
            + registry.players.len()
            + registry.npcs.len()
            + registry.goods.len()
            + registry.other_shapes.len(),
    );
    identities.extend(registry.monsters.iter().map(|id| ShapeIdentity {
        object_type: MONSTER_TYPE,
        id: *id,
        ex_id: CGuid::GUID_INVALID,
    }));
    identities.extend(registry.players.iter().map(|id| ShapeIdentity {
        object_type: PLAYER_TYPE,
        id: *id,
        ex_id: CGuid::GUID_INVALID,
    }));
    identities.extend(registry.npcs.iter().map(|id| ShapeIdentity {
        object_type: NPC_TYPE,
        id: *id,
        ex_id: CGuid::GUID_INVALID,
    }));
    identities.extend(registry.goods.iter().map(|ex_id| ShapeIdentity {
        object_type: GOODS_TYPE,
        id: 0,
        ex_id: *ex_id,
    }));
    identities.extend(registry.other_shapes.iter().map(|hash| ShapeIdentity {
        object_type: CBaseObject::calculate_type(*hash),
        id: CBaseObject::calculate_id(*hash),
        ex_id: CGuid::GUID_INVALID,
    }));
    identities
}

pub fn has_registered_shape(registry: &ServerRegionRegistry, identity: ShapeIdentity) -> bool {
    registry.contains(identity)
}

/// Identity-свёртка virtual `FindChildObject(type, id, ex_id)`
/// (`?FindChildObject@CServerRegion@@UAEPAVCBaseObject@@JJABVCGUID@@@Z`):
/// goods адресуются по `ex_id`, остальные типы по numeric `id`.
pub fn find_child_object<Resolver: ShapeResolver>(
    registry: &ServerRegionRegistry,
    object_type: i32,
    id: i32,
    ex_id: CGuid,
    resolver: &Resolver,
) -> Option<ShapeView> {
    let identity = ShapeIdentity {
        object_type,
        id: if object_type == GOODS_TYPE { 0 } else { id },
        ex_id: if object_type == GOODS_TYPE {
            ex_id
        } else {
            CGuid::GUID_INVALID
        },
    };
    if !registry.contains(identity) {
        return None;
    }
    resolver.resolve_shape(identity)
}

/// Bool-перегрузка virtual `FindChildObject(CBaseObject*)`
/// (`?FindChildObject@CServerRegion@@UAE_NPAVCBaseObject@@@Z`).
pub fn contains_child_object<Resolver: ShapeResolver>(
    registry: &ServerRegionRegistry,
    shape: &CShape,
    resolver: &Resolver,
) -> bool {
    let identity = shape.identity();
    find_child_object(
        registry,
        identity.object_type,
        identity.id,
        identity.ex_id,
        resolver,
    )
    .is_some()
}

/// Размер `m_vPlayers`: исходный `GetPlayerAmout@CServerRegion` возвращал
/// `(end - begin) / 4` того же vector storage.
pub fn get_player_amount(registry: &ServerRegionRegistry) -> u32 {
    registry.players.len() as u32
}
