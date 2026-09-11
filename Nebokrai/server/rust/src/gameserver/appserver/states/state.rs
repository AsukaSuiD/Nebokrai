//! Базовые контракты и общий virtual AI/End применённых состояний GameServer.
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
//! ClearAllStates (0x004CF090) сохраняет три death-прохода, исходные исключения
//! ID, прямой GetCell и строгий байт Undead==1. End и destructor-only хвост
//! разделены; по окончании End перечитывается текущая позиция. Общий хвост
//! использует также CastCure, без второго UpdateProperty и уплотнения.
//! Данные Begin для базового End различают загруженный null GetUser и
//! подтверждённого runtime user-holder; полный общий Begin ещё не введён.

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

/// Базовый End 0x005DBCE0 для owners, у которых runtime GetUser подтверждён
/// как holder, а StartAllStates вызывает Begin(null, holder). Sufferer у
/// отдельных BFAttribute может отличаться и не подставляется вместо user.
/// Загруженный экземпляр помечается завершённым, но GetUser остаётся null:
/// его удаляет внешний ClearAllStates, а не выдуманный GetSufferer в base End.
/// Владельцы с отдельной сохранённой identity источника сюда не направляются.
pub(crate) fn end_base_applied_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    bytes: usize,
) -> bool {
    let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) else { return false };
    let Some(from_save) = shape.applied_state_was_loaded(key) else { return false };
    shape.mark_applied_state_ended(key);
    if from_save {
        return false;
    }
    let removed = shape.remove_applied_state_data(key, bytes).is_some();
    if removed && holder.object_type == 400 {
        let _ = game.update_player_properties(holder.id);
    }
    removed
}

/// Прямой virtual +0x1C, без искусственного deadline и без удержания payload
/// на стеке во время callback. bool сообщает удаление, не значение IsEnded.
pub(crate) fn end_move_shape_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(end) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state_data(key)).map(state_end)
    else { return false };
    end(game, region_id, holder, key)
}

/// Общий хвост ClearAllStates/CastCure: вызов выбранного End и затем
/// destructor оставшегося в этой же позиции объекта. Внешний UpdateProperty
/// сюда не входит; RemoveState внутри конкретного End выполняет свой сам.
pub(crate) fn end_and_destroy_state_at(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    index: usize,
) -> Option<()> {
    let key = resolve_state_move_shape(game, region_id, holder)?
        .state_at(index).map(|(key, _)| key);
    if let Some(key) = key {
        end_move_shape_state(game, region_id, holder, key);
        let shape = resolve_state_move_shape_mut(game, region_id, holder)?;
        if let Some((remaining, _)) = shape.state_at(index) {
            shape.remove_applied_state(remaining);
        }
    }
    Some(())
}

/// CMoveShape::ClearAllStates (0x004CF090). Три death-прохода сохраняют
/// native исключения и самостоятельные UpdateProperty первых двух проходов.
/// После End перечитывается именно позиция, не прежний ключ: callback мог
/// удалить или заменить её. Destructor-only хвост не публикует второй End.
/// Без death-фильтра длина тоже живая; в конце безопасно освобождается массив
/// вместо оставленных native operator_delete висячих границ vector.
pub(crate) fn clear_move_shape_states(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    after_death: bool,
) -> Option<()> {
    let passes = if after_death { 3 } else { 1 };
    for pass in 0..passes {
        let mut index = 0;
        loop {
            let shape = resolve_state_move_shape(game, region_id, holder)?;
            if index >= shape.state_slot_count() { break; }
            let selected = if let Some((key, state)) = shape.state_at(index) {
                let should_end = if !after_death {
                    true
                } else if pass == 0 {
                    matches!(state, StateData::ChangeBody(state) if !state.continue_after_death)
                } else if pass == 1 {
                    matches!(state, StateData::Undead(state) if state.disappear_after_dead == 1)
                } else {
                    match state.state_id() {
                        100_007..=100_012 | 0x37 | 0x38 => false,
                        0x32 | 0x33 => {
                            if let Some(region) = game.find_region(shape.shape().get_region_id()) {
                                // Невиртуальный CRegion::GetCell, не war GetSecurity.
                                let x = shape.shape().get_tile_x().ok()?;
                                let y = shape.shape().get_tile_y().ok()?;
                                region.base().region.get_cell(x, y).ok()?
                                    .is_some_and(|cell| cell.security().value() == 0)
                            } else {
                                true
                            }
                        }
                        _ => true,
                    }
                };
                should_end.then_some(key)
            } else { None };
            if selected.is_some() {
                end_and_destroy_state_at(game, region_id, holder, index)?;
                if after_death && pass < 2 && holder.object_type == 400 {
                    let _ = game.update_player_properties(holder.id);
                }
            }
            index += 1;
        }
    }
    if !after_death {
        resolve_state_move_shape_mut(game, region_id, holder)?.release_state_slots();
    }
    Some(())
}

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

// Один список виртуального поведения порождает оба selector-а: End не
// дублирует каталог AI и не требует фиктивного Runtime/чтения часов.
macro_rules! state_callbacks {
    ($($pattern:pat => ($ai:expr, $end:path)),+ $(,)?) => {
        fn state_ai<Runtime: GameMainLoopRuntime>(state: &StateData) -> StateAi<Runtime> {
            match state { $($pattern => $ai),+ }
        }

        fn state_end(state: &StateData) -> fn(&mut CGame, i32, ShapeIdentity, StateKey) -> bool {
            match state { $($pattern => $end),+ }
        }
    };
}

state_callbacks! {
    StateData::PersistentAgility(_) => (
        |_, _, _, _, _| {},
        skills::agilitystate::end_persistent_agility_state
    ),
    StateData::TaiJi(_) => (
        |_, _, _, _, _| {},
        skills::taijistate::end_tai_ji_state
    ),
    StateData::EnlargeFullMiss(_) => (
        |_, _, _, _, _| {},
        skills::enlargefullmissstate::end_enlarge_full_miss_state
    ),
    StateData::EnlargeMaxHp(_) => (
        |_, _, _, _, _| {},
        skills::enlargemaxhpstate::end_enlarge_max_hp_state
    ),
    StateData::EnlargeMaxMp(_) => (
        |_, _, _, _, _| {},
        skills::enlargemaxmpstate::end_enlarge_max_mp_state
    ),
    StateData::Origin(_) => (
        |_, _, _, _, _| {},
        skills::originstate::end_origin_state
    ),
    StateData::MeteorArrow(_) => (
        |_, _, _, _, _| {},
        skills::meteorarrowstate::end_meteor_arrow_state
    ),
    StateData::EnergyHolding(_) => (
        |_, _, _, _, _| {},
        skills::energyholdingstate::end_energy_holding_state
    ),
    StateData::SoulCollect(_) => (
        |_, _, _, _, _| {},
        skills::soulcollectstate::end_soul_collect_state
    ),
    StateData::Swordship(_) => (
        |_, _, _, _, _| {},
        skills::swordshipstate::end_swordship_state
    ),
    StateData::WuXing(_) => (
        |_, _, _, _, _| {},
        skills::wuxingstate::end_wuxing_state
    ),
    StateData::Agility2(_) => (
        |game, region, target, key, runtime| {
            skills::agilitystate2::update_agility_state_2(game, region, target, key, runtime.now_milliseconds());
        },
        skills::agilitystate2::end_agility_state_2
    ),
    StateData::Callosity(_) => (
        |game, region, target, key, runtime| {
            skills::callositystate::update_callosity_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::callositystate::end_callosity_state_key
    ),
    StateData::Hearten(_) => (
        |game, region, target, key, runtime| {
            skills::heartenstate::update_hearten_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::heartenstate::end_hearten_state
    ),
    StateData::RageBreak(_) => (
        |game, region, target, key, runtime| {
            skills::ragebreakstate::update_rage_break_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::ragebreakstate::end_rage_break_state_key
    ),
    StateData::Pillar(_) => (
        |game, region, target, key, runtime| {
            skills::pillarstate::update_pillar_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::pillarstate::end_pillar_state
    ),
    StateData::TianShenXiaFan(_) => (
        |game, region, target, key, runtime| {
            skills::tianshenxiafanstate::update_tian_shen_xia_fan_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::tianshenxiafanstate::end_tian_shen_xia_fan_state
    ),
    StateData::Wangsheng(_) => (
        |game, region, target, key, runtime| {
            skills::wangshengstate::update_wangsheng_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::wangshengstate::end_wangsheng_state
    ),
    StateData::Blind(_) | StateData::KnockOut(_) | StateData::SpiderWeb(_)
    | StateData::Seal(_) | StateData::Strike(_) => (
        |game, region, target, key, runtime| {
            skills::blindstate::update_blind_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::blindstate::end_blind_state
    ),
    StateData::Heal(_) => (
        |game, region, target, key, runtime| {
            skills::healstate::update_stored_heal_state(game, region, target, key, || runtime.now_milliseconds());
        },
        skills::healstate::end_heal_state
    ),
    StateData::PoisonArrow(_) => (
        |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::poisonarrowstate::update_player_poison_arrow_state(game, target.id, key, runtime); }
                600 => { skills::poisonarrowstate::update_monster_poison_arrow_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        skills::poisonarrowstate::end_poison_arrow_state
    ),
    StateData::SpiderPoison(_) => (
        |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::spiderpoisonstate::update_player_spider_poison_state(game, target.id, key, runtime); }
                600 => { skills::spiderpoisonstate::update_monster_spider_poison_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        skills::spiderpoisonstate::end_spider_poison_state
    ),
    StateData::SpriteBurn(_) => (
        |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::spriteburnstate::update_player_sprite_burn_state(game, target.id, key, runtime); }
                600 => { skills::spriteburnstate::update_monster_sprite_burn_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        skills::spriteburnstate::end_sprite_burn_state
    ),
    StateData::BloodLoss(_) => (
        |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::bloodlossstate::update_player_blood_loss_state(game, target.id, key, runtime); }
                600 => { skills::bloodlossstate::update_monster_blood_loss_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        skills::bloodlossstate::end_blood_loss_state
    ),
    StateData::LeafCut(_) => (
        |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::leafcutstate::update_player_leaf_cut_state(game, target.id, key, runtime); }
                600 => { skills::leafcutstate::update_monster_leaf_cut_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        skills::leafcutstate::end_leaf_cut_state
    ),
    StateData::LeafCut2(_) => (
        |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::leafcutstate2::update_player_leaf_cut_2_state(game, target.id, key, runtime); }
                600 => { skills::leafcutstate2::update_monster_leaf_cut_2_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        skills::leafcutstate2::end_leaf_cut_2_state
    ),
    StateData::LeafCut3(_) => (
        |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::leafcutstate3::update_player_leaf_cut_3_state(game, target.id, key, runtime); }
                600 => { skills::leafcutstate3::update_monster_leaf_cut_3_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        skills::leafcutstate3::end_leaf_cut_3_state
    ),
    StateData::Kerosene(_) => (
        |game, region, target, key, runtime| {
            match target.object_type {
                400 => { skills::kerosenestate::update_player_kerosene_state(game, target.id, key, runtime); }
                600 => { skills::kerosenestate::update_monster_kerosene_state(game, region, target.id, key, runtime); }
                _ => {}
            }
        },
        skills::kerosenestate::end_kerosene_state
    ),
    StateData::Cure(_) => (
        |game, region, target, key, runtime| {
            skills::curestate::update_cure_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::curestate::end_cure_state_key
    ),
    StateData::BossBlueQuake(_) => (
        |game, region, target, key, runtime| {
            skills::bossbluequakestate::update_boss_blue_quake_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::bossbluequakestate::end_boss_blue_quake_state
    ),
    StateData::BoaLock(_) => (
        |game, region, target, key, runtime| {
            skills::boalockstate::update_boa_lock_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::boalockstate::end_boa_lock_state
    ),
    StateData::Rush(_) => (
        |game, region, target, key, runtime| {
            skills::rushstate::update_rush_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::rushstate::end_rush_state
    ),
    StateData::Rush2(_) => (
        |game, region, target, key, runtime| {
            skills::rushstate2::update_rush_2_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::rushstate2::end_rush_2_state
    ),
    StateData::KnightCut(_) => (
        |game, region, target, key, runtime| {
            skills::knightcutstate::update_knight_cut_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::knightcutstate::end_knight_cut_state
    ),
    StateData::GodBless(_) => (
        |game, region, target, key, runtime| {
            skills::godblessstate::update_god_bless_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::godblessstate::end_god_bless_state
    ),
    StateData::Roar(_) => (
        |game, region, target, key, runtime| {
            skills::roarstate::update_roar_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::roarstate::end_roar_state
    ),
    StateData::Weak(_) => (
        |game, region, target, key, _runtime| {
            skills::weakstate::update_weak_state(game, region, target, key);
        },
        skills::weakstate::end_weak_state
    ),
    StateData::Fury(_) => (
        |game, region, target, key, runtime| {
            skills::furystate::update_fury_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::furystate::end_fury_state
    ),
    StateData::BossBlueFury(_) => (
        |game, region, target, key, runtime| {
            skills::bossbluefurystate::update_boss_blue_fury_state(game, region, target, key, || runtime.now_milliseconds());
        },
        skills::bossbluefurystate::end_boss_blue_fury_state
    ),
    StateData::PoisonFog(_) => (
        |game, region, target, key, runtime| {
            skills::poisonfogstate::update_poison_fog_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::poisonfogstate::end_poison_fog_state
    ),
    StateData::BattleFairyAttribute(_) => (
        |game, region, target, key, runtime| {
            skills::battlefairyattributestate::update_battle_fairy_attribute_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::battlefairyattributestate::end_battle_fairy_attribute_state
    ),
    StateData::DefenseShield(_) => (
        |game, region, target, key, runtime| {
            skills::shieldstate::update_defense_shield(game, region, target, key, runtime.now_milliseconds());
        },
        skills::shieldstate::end_defense_shield
    ),
    StateData::DaubPoison(_) => (
        |game, region, target, key, runtime| {
            skills::daubpoisonstate::update_daub_poison_state(game, region, target, key, runtime.now_milliseconds());
        },
        skills::daubpoisonstate::end_daub_poison_state
    ),
    StateData::AutomaticRestore(_) => (
        |game, region, target, key, runtime| {
            game.update_move_shape_automatic_restore_state(region, target, key, runtime);
        },
        CGame::end_move_shape_automatic_restore_state
    ),
    StateData::ConsumableRestore(_) => (
        |game, region, target, key, runtime| {
            game.update_move_shape_consumable_restore_state(region, target, key, runtime);
        },
        CGame::end_move_shape_consumable_restore_state
    ),
    StateData::Particular(_) => (
        |game, region, target, key, runtime| {
            game.update_move_shape_particular_state(region, target, key, runtime);
        },
        CGame::end_move_shape_particular_state
    ),
    StateData::Team(_) => (
        |game, region, target, key, runtime| {
            game.update_move_shape_team_recruitment_state(region, target, key, runtime);
        },
        CGame::end_move_shape_team_recruitment_state
    ),
    StateData::Script(_) => (
        |game, region, target, key, runtime| {
            game.update_move_shape_script_move_state(region, target, key, runtime);
        },
        CGame::end_move_shape_script_move_state
    ),
    StateData::ChangeBody(_) => (
        |game, region, target, key, runtime| {
            game.update_move_shape_change_body_state(region, target, key, runtime);
        },
        CGame::end_move_shape_change_body_state
    ),
    StateData::Extended(_) => (
        |game, region, target, key, runtime| {
            game.update_move_shape_extended_state(region, target, key, runtime);
        },
        CGame::end_move_shape_extended_state
    ),
    StateData::Undead(_) => (
        |game, region, target, key, runtime| {
            game.update_move_shape_appellation_state(region, target, key, runtime);
        },
        CGame::end_move_shape_appellation_state
    ),
    StateData::Ride(_) => (
        |game, region, target, key, runtime| {
            game.update_move_shape_ride_state(region, target, key, runtime);
        },
        CGame::end_move_shape_ride_state
    ),
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
