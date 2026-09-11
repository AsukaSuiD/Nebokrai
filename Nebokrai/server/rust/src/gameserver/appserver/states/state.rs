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

//! Общий UpdateAbnormality ниже повторяет CMoveShape::UpdateAbnormality
//! (0x004CFD00): уплотнение в прологе, исходная длина после UpdateProperty,
//! новое чтение позиции до каждого AI и контроль длины после callback.
//! SlotMap удерживает идентичность, но не продлевает жизнь удалённого payload.
//! Часы принадлежат конкретному AI; общий проход их не читает. End не вызывает
//! уплотнения. GS0121/GS0122 используют исходные StringTable и 32-битные аргументы.
//! Для постоянных Agility/Natural/Rapture, TaiJi, Enlarge*, Origin, MeteorArrow,
//! EnergyHolding, SoulCollect, Swordship1–4 и WuXing exact vtable+0x0C указывает
//! на 0x00485540 (ret), а не базовый CState::AI 0x005DBC90, вызывающий End.
//! Прочие варианты перечислены явно: отсутствующий override не становится no-op.
//! Сейчас общий проход вызывают живые player/monster; property-проекция игрока
//! пересчитывается через CGame, monster читает модификаторы из живых состояний.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::moveshape::{CMoveShape, StateData, StateKey};
use crate::gameserver::appserver::skills;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::SkillLifecycle;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const STATE_IDENTITY_BYTES: usize = 16;

type StateAi<Runtime> = fn(&mut CGame, i32, ShapeIdentity, StateKey, &mut Runtime);

/// Исходный массив перечитывается после каждого callback: добавленное состояние
/// не расширяет границу этого прохода, а удалённое не оживает из снимка ключей.
pub(crate) fn update_move_shape_states<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    runtime: &mut Runtime,
) -> Option<()> {
    let changed = resolve_state_move_shape_mut(game, region_id, identity)?.compact_state_slots();
    if changed && identity.object_type == 400 {
        let _ = game.update_player_properties(identity.id);
    }
    let initial_len = resolve_state_move_shape(game, region_id, identity)?.state_slot_count();
    for index in 0..initial_len {
        let shape = resolve_state_move_shape(game, region_id, identity)?;
        let current_len = shape.state_slot_count();
        if index >= current_len {
            log_state_array_change(game, shape.shape(), b"GS0121", None, initial_len, current_len, index);
            return Some(());
        }
        let Some((key, state)) = shape.state_at(index) else { continue };
        let state_id = state.state_id();
        let ai = state_ai::<Runtime>(state);
        ai(game, region_id, identity, key, runtime);
        let shape = resolve_state_move_shape(game, region_id, identity)?;
        let current_len = shape.state_slot_count();
        if current_len != initial_len {
            log_state_array_change(game, shape.shape(), b"GS0122", Some(state_id), initial_len, current_len, index);
        }
    }
    Some(())
}

fn log_state_array_change(
    game: &CGame,
    shape: &CShape,
    string_id: &[u8],
    state_id: Option<u32>,
    initial_len: usize,
    current_len: usize,
    index: usize,
) {
    use crate::gameserver::gameserver::game::{format_legacy_mixed, LegacyFormatArgument};
    let mut args = vec![LegacyFormatArgument::Bytes(shape.base_object().get_name())];
    if let Some(state_id) = state_id {
        args.push(LegacyFormatArgument::Word32(state_id));
    }
    args.extend([initial_len, current_len, index].map(|value| LegacyFormatArgument::Word32(value as u32)));
    let text = format_legacy_mixed(game.get_string_by_id(string_id), &args, 0xff);
    crate::public::tools::put_debug_string(&text);
}

/// Только адаптер краткого заимствования уже существующих regional End.
/// Отправки, повторно ищущие регион в CGame, выполняются после возврата owner.
fn with_state_region<R>(
    game: &mut CGame,
    region_id: i32,
    update: impl FnOnce(&mut CGame, &mut CServerRegion) -> R,
) -> Option<R> {
    let mut owner = game.take_region_owner(region_id)?;
    let result = update(game, owner.base_mut());
    game.restore_region_owner(owner);
    Some(result)
}

fn state_ai<Runtime: GameMainLoopRuntime>(state: &StateData) -> StateAi<Runtime> {
    match state {
        StateData::PersistentAgility(_) | StateData::TaiJi(_)
        | StateData::EnlargeFullMiss(_) | StateData::EnlargeMaxHp(_)
        | StateData::EnlargeMaxMp(_) | StateData::Origin(_)
        | StateData::MeteorArrow(_) | StateData::EnergyHolding(_)
        | StateData::SoulCollect(_) | StateData::Swordship(_)
        | StateData::WuXing(_) => |_, _, _, _, _| {},
        StateData::Agility2(_) => |game, region, target, key, runtime| {
            skills::agilitystate2::update_agility_state_2(game, region, target, key, runtime.now_milliseconds());
        },
        StateData::Callosity(_) => |game, region, target, key, runtime| {
            skills::callositystate::update_callosity_state(game, region, target, key, runtime.now_milliseconds());
        },
        StateData::Hearten(_) => |game, region, target, key, runtime| {
            skills::heartenstate::update_hearten_state(game, region, target, key, runtime.now_milliseconds());
        },
        StateData::RageBreak(_) => |game, region, target, key, runtime| {
            skills::ragebreakstate::update_rage_break_state(game, region, target, key, runtime.now_milliseconds());
        },
        StateData::Pillar(_) => |game, region, target, key, runtime| {
            skills::pillarstate::update_pillar_state(game, region, target, key, runtime.now_milliseconds());
        },
        StateData::TianShenXiaFan(_) => |game, region, target, key, runtime| {
            skills::tianshenxiafanstate::update_tian_shen_xia_fan_state(game, region, target, key, runtime.now_milliseconds());
        },
        StateData::Wangsheng(_) => |game, region, target, key, runtime| {
            skills::wangshengstate::update_wangsheng_state(game, region, target, key, runtime.now_milliseconds());
        },
        StateData::Blind(_) | StateData::KnockOut(_) | StateData::SpiderWeb(_)
        | StateData::Seal(_) | StateData::Strike(_) => |game, region, target, key, runtime| {
            skills::blindstate::update_blind_state(game, region, target, key, runtime.now_milliseconds());
        },
        StateData::Heal(_) => |game, region, target, key, runtime| {
            skills::healstate::update_stored_heal_state(game, region, target, key, || runtime.now_milliseconds());
        },
        StateData::PoisonArrow(_) => |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::poisonarrowstate::update_player_poison_arrow_state(game, target.id, key, runtime); }
                600 => { skills::poisonarrowstate::update_monster_poison_arrow_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        StateData::SpiderPoison(_) => |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::spiderpoisonstate::update_player_spider_poison_state(game, target.id, key, runtime); }
                600 => { skills::spiderpoisonstate::update_monster_spider_poison_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        StateData::SpriteBurn(_) => |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::spriteburnstate::update_player_sprite_burn_state(game, target.id, key, runtime); }
                600 => { skills::spriteburnstate::update_monster_sprite_burn_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        StateData::BloodLoss(_) => |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::bloodlossstate::update_player_blood_loss_state(game, target.id, key, runtime); }
                600 => { skills::bloodlossstate::update_monster_blood_loss_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        StateData::LeafCut(_) => |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::leafcutstate::update_player_leaf_cut_state(game, target.id, key, runtime); }
                600 => { skills::leafcutstate::update_monster_leaf_cut_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        StateData::LeafCut2(_) => |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::leafcutstate2::update_player_leaf_cut_2_state(game, target.id, key, runtime); }
                600 => { skills::leafcutstate2::update_monster_leaf_cut_2_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        StateData::LeafCut3(_) => |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::leafcutstate3::update_player_leaf_cut_3_state(game, target.id, key, runtime); }
                600 => { skills::leafcutstate3::update_monster_leaf_cut_3_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        StateData::Kerosene(_) => |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::kerosenestate::update_player_kerosene_state(game, target.id, key, runtime); }
                600 => { skills::kerosenestate::update_monster_kerosene_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        StateData::Cure(_) => |game, region, target, key, runtime| {
            skills::curestate::update_cure_state(game, region, target, key, runtime.now_milliseconds());
        },
        StateData::BossBlueQuake(_) => |game, region, target, key, runtime| {
            let now = runtime.now_milliseconds();
            match target.object_type {
                400 => { skills::bossbluequakestate::expire_player_boss_blue_quake_state(game, target.id, key, now); }
                600 => { with_state_region(game, region, |game, region| skills::bossbluequakestate::expire_monster_boss_blue_quake_state(game, region, target.id, key, now)); }
                _ => {}
            }
        },
        StateData::BoaLock(_) => |game, region, target, key, runtime| {
            let now = runtime.now_milliseconds();
            match target.object_type {
                400 => { skills::boalockstate::expire_player_boa_lock_state(game, target.id, key, now); }
                600 => { with_state_region(game, region, |game, region| skills::boalockstate::expire_monster_boa_lock_state(game, region, target.id, key, now)); }
                _ => {}
            }
        },
        StateData::Rush(_) => |game, region, target, key, runtime| {
            let now = runtime.now_milliseconds();
            match target.object_type {
                400 => { skills::rushstate::expire_player_rush_state(game, target.id, key, now); }
                600 => { with_state_region(game, region, |game, region| skills::rushstate::expire_monster_rush_state(game, region, target.id, key, now)); }
                _ => {}
            }
        },
        StateData::Rush2(_) => |game, region, target, key, runtime| {
            let now = runtime.now_milliseconds();
            match target.object_type {
                400 => { skills::rushstate2::expire_player_rush_2_state(game, target.id, key, now); }
                600 => { with_state_region(game, region, |game, region| skills::rushstate2::expire_monster_rush_2_state(game, region, target.id, key, now)); }
                _ => {}
            }
        },
        StateData::KnightCut(_) => |game, region, target, key, runtime| {
            let now = runtime.now_milliseconds();
            match target.object_type {
                400 => { skills::knightcutstate::expire_player_knight_cut_state(game, target.id, key, now); }
                600 => { with_state_region(game, region, |game, region| skills::knightcutstate::expire_monster_knight_cut_state(game, region, target.id, key, now)); }
                _ => {}
            }
        },
        StateData::GodBless(_) => |game, region, target, key, runtime| {
            let now = runtime.now_milliseconds();
            match target.object_type {
                400 => { skills::godblessstate::finish_player_god_bless(game, target.id, key, now, runtime); }
                600 => { skills::godblessstate::finish_monster_god_bless(game, region, target.id, key, now); }
                _ => {}
            }
        },
        StateData::Roar(_) => |game, region, target, key, runtime| {
            let now = runtime.now_milliseconds();
            match target.object_type {
                400 => { skills::roarstate::finish_player_roar(game, target.id, key, now, runtime); }
                600 => { skills::roarstate::finish_monster_roar(game, region, target.id, key, now); }
                _ => {}
            }
        },
        StateData::Weak(_) => |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::weakstate::finish_player_weak_outside(game, target.id, key, runtime); }
                600 => { skills::weakstate::finish_monster_weak_outside(game, region, target.id, key); }
                _ => {}
            }
        },
        StateData::Fury(_) => |game, region, target, key, runtime| {
            let now = runtime.now_milliseconds();
            match target.object_type {
                400 => { skills::furystate::expire_player_fury_state(game, target.id, key, now, runtime); }
                600 => { with_state_region(game, region, |game, region| skills::furystate::expire_monster_fury_state(game, region, target.id, key, now)); }
                _ => {}
            }
        },
        StateData::BossBlueFury(_) => |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::bossbluefurystate::expire_player_boss_blue_fury_state(game, target.id, key, || runtime.now_milliseconds()); }
                600 => { with_state_region(game, region, |game, region| skills::bossbluefurystate::expire_monster_boss_blue_fury_state(game, region, target.id, key, || runtime.now_milliseconds())); }
                _ => {}
            }
        },
        StateData::PoisonFog(_) => |game, region, target, key, runtime| {
            let now = runtime.now_milliseconds();
            match target.object_type {
                400 => { skills::poisonfogstate::expire_player_poison_fog_state(game, target.id, key, now, runtime); }
                600 => {
                    if let Some(Some(end)) = with_state_region(game, region, |_, region| skills::poisonfogstate::take_expired_monster_poison_fog_state(region, target.id, key, now)) {
                        end.deliver(game, region, now);
                    }
                }
                _ => {}
            }
        },
        StateData::BattleFairyAttribute(_) => |game, region, target, key, runtime| {
            let now = runtime.now_milliseconds();
            match target.object_type {
                400 => { skills::battlefairyattributestate::expire_player_battle_fairy_attribute_state(game, target.id, key, now); }
                600 => {
                    if let Some(Some(end)) = with_state_region(game, region, |_, region| skills::battlefairyattributestate::take_expired_monster_battle_fairy_attribute_state(region, target.id, key, now)) {
                        end.deliver(game, region);
                    }
                }
                _ => {}
            }
        },
        StateData::DefenseShield(_) => |game, region, target, key, runtime| {
            skills::shieldstate::update_defense_shield(game, region, target, key, runtime.now_milliseconds());
        },
        StateData::DaubPoison(_) => |game, region, target, key, runtime| {
            skills::daubpoisonstate::update_daub_poison_state(game, region, target, key, runtime.now_milliseconds());
        },
        StateData::AutomaticRestore(_) => |game, region, target, key, runtime| {
            game.update_move_shape_automatic_restore_state(region, target, key, runtime);
        },
        StateData::ConsumableRestore(_) => |game, region, target, key, runtime| {
            game.update_move_shape_consumable_restore_state(region, target, key, runtime);
        },
        StateData::Particular(_) => |game, region, target, key, runtime| {
            game.update_move_shape_particular_state(region, target, key, runtime);
        },
        StateData::Team(_) => |game, region, target, key, runtime| {
            game.update_move_shape_team_recruitment_state(region, target, key, runtime);
        },
        StateData::Script(_) => |game, region, target, key, runtime| {
            game.update_move_shape_script_move_state(region, target, key, runtime);
        },
        StateData::ChangeBody(_) => |game, region, target, key, runtime| {
            game.update_move_shape_change_body_state(region, target, key, runtime);
        },
        StateData::Extended(_) => |game, region, target, key, runtime| {
            game.update_move_shape_extended_state(region, target, key, runtime);
        },
        StateData::Undead(_) => |game, region, target, key, runtime| {
            game.update_move_shape_appellation_state(region, target, key, runtime);
        },
        StateData::Ride(_) => |game, region, target, key, runtime| {
            game.update_move_shape_ride_state(region, target, key, runtime);
        },
    }
}



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
