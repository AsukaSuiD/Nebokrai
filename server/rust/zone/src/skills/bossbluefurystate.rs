//! Живые Begin/End/Restart/AI и пересчёт монстра ярости синего босса
//! `CBossBlueFuryState` (`0x1F7`), а также установочный Begin для навыка.
//! Источник: точная пара `gameserver.exe` (SHA-256 `4F5C98E0…`) +
//! `GameServer.pdb` (RSDS match), исходный владелец
//! `appserver/skills/bossbluefurystate.cpp/.h`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/bossbluefurystate.rs`; тела перенесены
//! кластером E3 полосы D/E (сверка — разведка `.local/recon-de/notes/
//! E4-bossbluefury.md`, тела `.local/recon-de/disasm/CBossBlueFuryState.txt`).
//! Данные, срок, формула процента и 12-байтный codec — Zone
//! `effects/bossbluefury.rs` (подтверждены побайтно; здесь не дублируются).
//!
//! Машинные якоря (VA = RVA + 0x400000): ctor `0x5E8A60` (arg1 factor →
//! `[+0x38]`, arg2 keep → `[+0x3C]`, arg3 weak → `[+0x40]`; ID `0x1F7`, без
//! чтения часов); Begin триадой `0x5E8B60`/`0x5E8C30`/`0x5E8ED0` (объектная с
//! NULL U → ret 0; base Begin, затем SetMoveable(0) и SetFightable(0) на S,
//! затем `new` loop=1 visual `CBossBlueFuryStateVisualEffect` без немедленного
//! пакета); AI `0x5E8D50` (часы №1 строго позже `[+0x40]+[+0x2C]` → unlock
//! обоих запретов на sufferer КАЖДЫЙ проход, без one-shot; часы №2 строго
//! позже `[+0x3C]+[+0x2C]` → vcall End); End `0x5E8D10` (visual(1)? →
//! GetSufferer → `CMoveShape::RemoveState(this)` `0x4CDAB0` → SetMoveable(1) →
//! SetFightable(1) на том же sufferer); Restart `0x5FD450` (только
//! `[+0x2C] = timeGetTime()`); OnUpdateProperties `0x5E8DC0` (GetSufferer →
//! visual(0) → sufferer type `0x258` + RTTI CMonster: max живым getter
//! `vcall+0xE8` → trunc-gain → `vcall+0x1AC`, затем min живым getter
//! `vcall+0xE4` → gain → `vcall+0x1A8`; иначе ret 1; порядок maximum →
//! minimum, каждый процент от живого getter после предыдущей прибавки);
//! GetRemainedTime `0x5D5F30` (ICF, keep `[+0x3C]`); Serialize `0x5E7330` /
//! Unserialize `0x5D6190` (кодек — effects; загруженный weak_time остаётся 0).
//!
//! Диспетчерский restart моделирует не timer-only Restart, а путь
//! `Begin(NULL, holder)` загрузки: base Begin сохраняет timestamp/user,
//! запреты и loop=1 visual повторяются, готовая запись и её ключ не заменяются.
//! Установка нового состояния машинно — Begin(U,U): участники хранятся
//! записью (ex_id сброшен), запреты идут до выделения visual; запись арены —
//! `encoded_for_install`; экземпляр добавляется в хвост. Узкая достижимость
//! нескольких записей `0x1F7` (дублированные записи БД, загруженных Restart-ом
//! без продува) снимается машинным продувом навыка — см. `skills/bossbluefury.rs`.
//!
//! Объявленные швы переноса (не расхождения): hub `statecast::StateCastGame`
//! (арена, property/end visual, удаление `RemoveState(pointer)`) реализован у
//! прежнего владельца; `fightable` открыт новым методом `StateCastMoveShape`
//! этой порцией (до неё hub открывал только `moveable`). Монстровый пересчёт
//! OnUpdateProperties — фасад `BossBlueFuryStateGame` (реализация у делегата
//! `appserver/skills/bossbluefurystate.rs`); live-getter порядок maximum →
//! minimum сохранён двум отдельными прибавками.

use crate::effects::{
    BOSS_BLUE_FURY_STATE_BYTES, BossBlueFuryState,
};
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::MONSTER_TYPE;

use super::state::StateKey;
use super::statecast::{
    StateCastGame, StateCastMoveShape, StateCastPropertyTarget, state_cast_storage_participant,
};

/// Переходные фасады прежнего владельца `CGame` для монстрового пересчёта
/// `CBossBlueFuryState::OnUpdateProperties` (`0x5E8DC0`): origin-name снимок,
/// живые границы после каждой прибавки и два wrapping-add в модификаторы.
pub trait BossBlueFuryStateGame: StateCastGame {
    /// `(minimum_attack, maximum_attack)` setup-строки живого монстра-цели
    /// (`find_monster_by_id` → `find_monster_property_by_origin_name`).
    fn boss_blue_fury_monster_attack_base(&self, region_id: i32, monster_id: i32)
        -> Option<(u32, u32)>;

    /// Живой `state_attack_bounds` монстра над тем же снимком; читается заново
    /// после каждой прибавки (машинный повторный виртуальный getter).
    fn boss_blue_fury_monster_attack_bounds(
        &self,
        region_id: i32,
        monster_id: i32,
        base: (u32, u32),
    ) -> Option<(u32, u32)>;

    /// Wrapping-прибавки к накопленным модификаторам живого монстра
    /// (`property_modifiers_mut`; машинные `vcall+0x1A8`/`vcall+0x1AC`).
    fn apply_boss_blue_fury_monster_gains(
        &mut self,
        region_id: i32,
        monster_id: i32,
        minimum_gain: i32,
        maximum_gain: i32,
    );
}

/// Объектный Begin нового `CBossBlueFuryState` (`0x5E8ED0` в связке с ctor
/// `0x5E8A60`): держатель, U и S обязаны разрешиться; часы ставятся при U;
/// участники перечитываются живыми (ex_id сброшен); запреты движения и боя
/// идут до выделения loop=1 visual каталога арены; запись — технический
/// `encoded_for_install` владельца; экземпляр добавляется в хвост.
#[allow(clippy::too_many_arguments, reason = "User, Sufferer, держатель арены и три свойства независимы, как в исходном Begin")]
pub fn begin_primary_boss_blue_fury_state<Game: StateCastGame>(
    game: &mut Game,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    keep_time_ms: u32,
    attack_factor_percent: i32,
    weak_time_ms: u32,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let user = state_cast_storage_participant(game, user?)?;
    let sufferer = state_cast_storage_participant(game, sufferer?)?;
    // Конструктор часов не читает; timestamp при наличии U задаётся Begin-ом.
    let state = BossBlueFuryState::new(now(), keep_time_ms, attack_factor_percent, weak_time_ms);
    let record = state.encoded_for_install();
    let shape = game.resolve_state_move_shape_mut(holder_region, holder)?;
    // Оба запрета добавляются до выделения loop=1 visual (Begin владельца).
    shape.set_moveable(false);
    shape.set_fightable(false);
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(user));
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}

/// Полный End конкретного ключа (`0x5E8D10`): visual(1) по sufferer, повторное
/// разрешение фактического sufferer и `RemoveState(this)` у него (dtor записи
/// и UpdateProperty внутри шва), затем снятие обоих запретов на sufferer.
pub fn end_boss_blue_fury_state_key<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<BossBlueFuryState>(key)).is_none()
    { return false; }
    game.update_applied_state_end_visual(region_id, holder, key, StateCastPropertyTarget::Sufferer);
    let Some((sufferer_region, sufferer)) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    let removed = game.remove_applied_state_from(
        region_id, holder, key, (sufferer_region, sufferer), BOSS_BLUE_FURY_STATE_BYTES,
    );
    if let Some(shape) = game.resolve_state_move_shape_mut(sufferer_region, sufferer) {
        shape.set_moveable(true);
        shape.set_fightable(true);
    }
    removed
}

/// Диспетчерский restart загруженного экземпляра (путь `Begin(NULL, holder)`,
/// не timer-only Restart `0x5FD450`): base Begin помечает слот, сохраняя часы
/// и user; оба запрета и loop=1 visual повторяются без немедленного пакета.
pub fn restart_boss_blue_fury_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<BossBlueFuryState>(key)).is_none()
    { return false; }
    if !game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.mark_applied_state_begun(key))
    { return false; }
    if let Some(shape) = game.resolve_state_move_shape_mut(region_id, holder) {
        shape.set_moveable(false);
        shape.set_fightable(false);
    }
    let _ = game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.begin_applied_state_visual(key, 1));
    true
}

/// AI конкретного ключа (`0x5E8D50`): два отдельных чтения часов; после слабой
/// границы оба запрета снимаются на каждом проходе (без one-shot флага), по
/// строгому общему сроку — тот же End, что и у владельца.
pub fn update_boss_blue_fury_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    mut now_milliseconds: impl FnMut() -> u32,
) -> bool {
    let Some(state) = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<BossBlueFuryState>(key)).copied()
    else { return false; };
    if state.weak_elapsed(now_milliseconds()) {
        if let Some((sufferer_region, sufferer)) =
            game.resolve_applied_state_sufferer(region_id, holder, key)
            && let Some(shape) = game.resolve_state_move_shape_mut(sufferer_region, sufferer)
        {
            shape.set_moveable(true);
            shape.set_fightable(true);
        }
    }
    if state.expired(now_milliseconds()) {
        let _ = end_boss_blue_fury_state_key(game, region_id, holder, key);
    }
    true
}

/// OnUpdateProperties конкретного ключа (`0x5E8DC0`): visual по sufferer, затем
/// только для sufferer-монстра пересчёт maximum и minimum атаки — каждый
/// процент от живого getter после предыдущей прибавки, порядок maximum →
/// minimum, signed-delta в накопленные модификаторы.
pub fn update_boss_blue_fury_state_properties<Game: BossBlueFuryStateGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    let _ = game.update_property_state_visual::<BossBlueFuryState>(
        region_id, holder, key, StateCastPropertyTarget::Sufferer, now,
        |state, now| state.client_state_time(now),
    );
    let Some(state) = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<BossBlueFuryState>(key)).copied()
    else { return false; };
    if target.object_type != MONSTER_TYPE {
        return true;
    }
    let Some(base) = game.boss_blue_fury_monster_attack_base(target_region, target.id)
    else { return true; };
    // Машинный порядок: live getter maximum (vcall+0xE8) → vcall+0x1AC,
    // затем live getter minimum (vcall+0xE4) → vcall+0x1A8.
    if let Some(bounds) = game.boss_blue_fury_monster_attack_bounds(target_region, target.id, base) {
        let maximum_gain = state.attack_modifier(bounds.1);
        game.apply_boss_blue_fury_monster_gains(target_region, target.id, 0, maximum_gain);
    }
    if let Some(bounds) = game.boss_blue_fury_monster_attack_bounds(target_region, target.id, base) {
        let minimum_gain = state.attack_modifier(bounds.0);
        game.apply_boss_blue_fury_monster_gains(target_region, target.id, minimum_gain, 0);
    }
    true
}
