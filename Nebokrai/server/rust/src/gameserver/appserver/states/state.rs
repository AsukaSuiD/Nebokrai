//! Достигнутая базовая client-проекция `CState` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходные owners
//! `appserver/states/state.h/.cpp`. Vtable-аудит exact EXE подтверждает, что
//! базовые `GetClientStateTime` и `GetAdditionalData` сведены линкером в одну
//! функцию RVA `0x00201200`, возвращающую ноль. Конкретные state-классы могут
//! переопределять каждый getter; поэтому здесь закреплены только базовые
//! значения, а динамический remaining-time и additional-data остаются у
//! конкретных владельцев. Стандартные визуальные пакеты состояния отправляются
//! через уже заимствованного владельца региона, чтобы owner-проход не зависел от
//! повторного поиска временно вынутого региона. Точные `GetUser/GetSufferer`
//! разрешают player глобально, остальные identity типов
//! `500/600/1100/1200` через регион, а координатная ветвь выбирает первый
//! `CMoveShape` клетки. Базовый wire-префикс сохраняет четыре little-endian
//! `long` в порядке user type/ID → sufferer type/ID. Остальной корпус сохранён
//! ниже как `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Живое заимствование GetUser использует те же lookup-gates для чтения
//! visual и изменения movement; исчезнувший источник не заменяется
//! текущим держателем навыка. Эти адаптеры работают с опубликованным регионом,
//! не создавая копий owning форм или их ресурсов.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::SkillLifecycle;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const STATE_IDENTITY_BYTES: usize = 16;

/// Факты объектного CState::Begin при временно извлечённом регионе.
/// Запоминаются только region/type/id, без нового GUID-фильтра. Для
/// неподвижных CMoveShape реестр региона подтверждает регистрацию;
/// этот снимок не заменяет последующее разрешение GetSufferer в живой объект.
pub(crate) fn resolve_owned_skill_begin_object(
    game: &CGame,
    region: &CServerRegion,
    identity: ShapeIdentity,
) -> Option<(i32, ShapeIdentity)> {
    let identity = ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..identity };
    let region_id = match identity.object_type {
        400 => game.find_player(identity.id)?.shape().get_region_id(),
        500 => region.find_npc_by_id(identity.id)?.move_shape().shape().get_region_id(),
        600 => region.find_monster_by_id(identity.id)?.move_shape().shape().get_region_id(),
        1_100 | 1_200 if region.has_registered_shape(identity) => region.id,
        _ => return None,
    };
    Some((region_id, identity))
}

pub(crate) fn send_owned_state_visual(
    game: &CGame,
    region: &CServerRegion,
    shape: &CShape,
    state_id: u32,
    begin: bool,
    client_time: i32,
    additional_data: u32,
) {
    let identity = shape.identity();
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state_id as i32);
    if begin {
        message.add_long(client_time);
        message.add_long(additional_data as i32);
    }
    let _ = game.send_game_shape_around(region, shape, None, &message);
}

/// Координатная ветвь точного `CState::GetSufferer`: `(0, 0)` означает
/// отсутствие цели, иначе выбирается первый `CMoveShape` в порядке региона.
pub(crate) fn resolve_coordinate_sufferer(
    game: &CGame,
    region_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> Option<ShapeIdentity> {
    if tile_x == 0 && tile_y == 0 {
        return None;
    }
    let region = game.find_region(region_id)?.base();
    let (area_width, area_height) = game.area_dimensions();
    let mut shapes = Vec::new();
    region
        .get_shapes(
            tile_x,
            tile_y,
            area_width,
            area_height,
            game,
            &mut shapes,
        )
        .ok()?;
    shapes
        .into_iter()
        .map(|shape| shape.identity)
        .find(|identity| matches!(identity.object_type, 400 | 500 | 600 | 1_100 | 1_200))
}

/// Identity-ветвь точного `CState::GetSufferer`: игрок разрешается глобально,
/// остальные поддержанные `CMoveShape` — через сохранённый регион состояния.
pub(crate) fn resolve_identity_sufferer(
    game: &CGame,
    region_id: i32,
    identity: ShapeIdentity,
) -> Option<ShapeIdentity> {
    match identity.object_type {
        400 => game.find_player(identity.id).map(|_| identity),
        500 | 600 | 1_100 | 1_200 => game
            .find_shape_in_region(region_id, identity)
            .map(|_| identity),
        _ => None,
    }
}

/// Точный `CState::GetUser`: player хранится в глобальном реестре, остальные
/// достигнутые `CMoveShape` разрешаются через регион владельца состояния.
pub(crate) fn resolve_state_user(
    game: &CGame,
    region_id: i32,
    identity: ShapeIdentity,
) -> Option<ShapeIdentity> {
    resolve_identity_sufferer(game, region_id, identity)
}

/// GetSufferer сначала пытается разрешить сохранённую identity, затем клетку
/// в сохранённом регионе источника. Наличие живого GetUser не является gate.
pub(crate) fn resolve_skill_sufferer(
    game: &CGame,
    lifecycle: &SkillLifecycle,
) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = lifecycle.sufferer();
    if let Some(identity) = resolve_identity_sufferer(game, region, identity) {
        return Some((region, identity));
    }
    let (region, _) = lifecycle.user();
    let (x, y) = lifecycle.destination();
    Some((region, resolve_coordinate_sufferer(game, region, x, y)?))
}

/// Живой GetUser по сохранённым region/type/id, без подстановки держателя
/// состояния. Региональные объекты здесь принадлежат опубликованному CGame.
pub(crate) fn resolve_state_move_shape(
    game: &CGame,
    region_id: i32,
    identity: ShapeIdentity,
) -> Option<&CMoveShape> {
    let identity = resolve_state_user(game, region_id, identity)?;
    match identity.object_type {
        400 => Some(game.find_player(identity.id)?.move_shape()),
        500 => Some(game.find_region(region_id)?.base().find_npc_by_id(identity.id)?.move_shape()),
        600 => Some(game.find_region(region_id)?.base().find_monster_by_id(identity.id)?.move_shape()),
        1_100 | 1_200 => Some(game.find_region(region_id)?.stationary_build(identity)?.move_shape()),
        _ => None,
    }
}

pub(crate) fn resolve_state_move_shape_mut(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
) -> Option<&mut CMoveShape> {
    let identity = resolve_state_user(game, region_id, identity)?;
    match identity.object_type {
        400 => Some(game.find_player_mut(identity.id)?.move_shape_mut()),
        500 => Some(game.find_region_mut(region_id)?.base_mut().find_npc_by_id_mut(identity.id)?.move_shape_mut()),
        600 => Some(game.find_region_mut(region_id)?.base_mut().find_monster_by_id_mut(identity.id)?.move_shape_mut()),
        1_100 | 1_200 => Some(game.find_region_mut(region_id)?.stationary_build_mut(identity)?.move_shape_mut()),
        _ => None,
    }
}

/// Byte-exact базовый `CState::Serialize`: user type/ID, затем sufferer type/ID.
pub(crate) fn encode_state_identities(
    user: ShapeIdentity,
    sufferer: ShapeIdentity,
) -> [u8; STATE_IDENTITY_BYTES] {
    let mut bytes = Vec::with_capacity(STATE_IDENTITY_BYTES);
    let mut writer = LegacyWriter::new(&mut bytes);
    writer.write_i32(user.object_type);
    writer.write_i32(user.id);
    writer.write_i32(sufferer.object_type);
    writer.write_i32(sufferer.id);
    bytes.try_into().expect("размер identity-префикса CState фиксирован")
}

/// Byte-exact базовый `CState::Unserialize` без C++ cursor side effects.
pub(crate) fn decode_state_identities(
    payload: &[u8],
    offset: usize,
) -> Result<(ShapeIdentity, ShapeIdentity), LegacyReadBlock> {
    let mut reader = LegacyReader::at(payload, offset)?;
    let read_identity = |reader: &mut LegacyReader<'_>| -> Result<ShapeIdentity, LegacyReadBlock> {
        Ok(ShapeIdentity {
            object_type: reader.read_i32()?,
            id: reader.read_i32()?,
            ex_id: CGuid::GUID_INVALID,
        })
    };
    Ok((read_identity(&mut reader)?, read_identity(&mut reader)?))
}

/// Exact базовый `CState::GetClientStateTime` для классов без override-а.
pub(crate) const fn default_client_state_time() -> i32 {
    0
}

/// Exact базовый `CState::GetAdditionalData` для классов без override-а.
pub(crate) const fn default_additional_data() -> u32 {
    0
}

/// Exact общее тело `CBlindState::GetRemainedTime` по адресу `0x005F2CD0`.
/// Проверка deadline и вычисление положительного остатка независимо читают
/// wrapping clock; второе чтение не выполняется на уже истёкшем состоянии.
pub(crate) fn timed_client_state_time(
    started_at_ms: u32,
    keep_time_ms: u32,
    mut now_milliseconds: impl FnMut() -> u32,
) -> u32 {
    let deadline = started_at_ms.wrapping_add(keep_time_ms);
    if deadline <= now_milliseconds() {
        0
    } else {
        deadline.wrapping_sub(now_milliseconds())
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.h

// ============================================================================
// FUNCTION: CState::IsEnded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.h:61
// RVA: 0x000D8380
// ADDRESS: 004d8380
// PROTOTYPE: int __thiscall IsEnded(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:21
// RVA: 0x001DBC90
// ADDRESS: 005dbc90
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::CState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:29
// RVA: 0x001DBCA0
// ADDRESS: 005dbca0
// PROTOTYPE: undefined __thiscall CState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::End
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:149
// RVA: 0x001DBCE0
// ADDRESS: 005dbce0
// PROTOTYPE: void __thiscall End(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::~CState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:48
// RVA: 0x001DBD40
// ADDRESS: 005dbd40
// PROTOTYPE: void __thiscall ~CState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:175
// RVA: 0x001DBD70
// ADDRESS: 005dbd70
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:208
// RVA: 0x001DBDD0
// ADDRESS: 005dbdd0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:227
// RVA: 0x001DBE20
// ADDRESS: 005dbe20
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
