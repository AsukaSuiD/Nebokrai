//! Призыв, следование и гибель/воскрешение боевого духа надетой боевой феи
//! (`CPlayer`-часть war-soul): `SetWarSoulStaus`, player-хвост
//! `CBattleFairyContainer::SummonBF` (mode ±1), `ComputeWarSoulXY` и его
//! мёртвая половина, player-tail spatial-входа в регион, periodic HP-death
//! префикс `CPlayer::AI`, `ReviveBattleFairy`, сброс после смерти хозяина.
//!
//! Quirk следования: dead-zone dist < 0.5, коэффициенты 0.265/0.065/0.045 по
//! порогам дистанции, оба fistp под fnstcw (x87 RC=truncate) — формула
//! воспроизведена дословно.
//!
//! Швы: живые поля прежнего `CPlayer` — view `BattleFairyWarSoul` либо
//! scalar-параметры; equipment/headgear — closure-швы прежнего владельца;
//! отложенные эффекты — упорядоченный `Vec` zone-типов (конверт/план — hub
//! `appserver/player.rs`, `gameeffectjournal.rs` старого пакета); spatial map
//! mutation — Zone `regions/serverregion/areagrid.rs`.
//!
//! VERIFIED: refresh — первый блок тела `CPlayer::AI` (RVA `0x59FF0`, полное
//! тело сверено, `.local/verify-bfai/`): GetGoods(10) → GAP 172==1 → GAP
//! 153==0 → четыре записи summoned/state/recall/died в этом порядке →
//! virtual `+0x9C`; до lost-timeout, region-gate и остального AI.
//! UNKNOWN: pub-имя `ReviveBattleFairy` не резолвится (inline); tail
//! `SummonBF` — у владельца контейнера.
//!
//! Исходный владелец PDB: `appserver/player.cpp/.h`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#battlefairysummon--призывследованиегибель-боевого-духа-cplayer

use crate::content::goods::{GAP_BF_BATTLE_FAIRY, GAP_BF_HP, GAP_BF_MAX_HP, GAP_BF_MAX_MP, GAP_BF_MP};
use crate::regions::area::WarSoulPoint;
use crate::regions::shape::ShapeCoordinateBlock;

/// Тайл-around кадр движения боевого духа (wire `0xBF605`).
pub const BATTLE_FAIRY_MOVE_MESSAGE_TYPE: u32 = 0x0b_f605;
/// Around кадр статуса боевого духа (wire `0xBF930`).
pub const BATTLE_FAIRY_STATUS_MESSAGE_TYPE: u32 = 0x0b_f930;
/// Around кадр призыва боевого духа (wire `0xBF92E`).
pub const BATTLE_FAIRY_SUMMON_MESSAGE_TYPE: u32 = 0x0b_f92e;
/// Активный `SKILL_MONSTER_TAMING` (0xD4) запрещает призыв SummonBF.
const MONSTER_TAMING_SKILL_ID: u32 = 0xd4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairyWarSoulAction {
    SetPosition {
        previous: WarSoulPoint,
        target: WarSoulPoint,
    },
    Delete {
        previous: WarSoulPoint,
        player_position: WarSoulPoint,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairySummonOutcome {
    FeatureDisabled,
    AlreadySummoned,
    AlreadyRecalled,
    MissingHeadgear,
    InvalidHeadgear,
    NoHitPoints,
    ActivePet,
    MonsterTamingActive,
    CoordinateBlocked(ShapeCoordinateBlock),
    Summoned,
    Recalled,
    IgnoredMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BattleFairySummonEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    AroundMessage {
        message_type: u32,
        player_id: i32,
        values: Vec<i32>,
    },
    PropertiesChanged {
        player_id: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairyFollowOutcome {
    ActiveSkill,
    NotSummoned,
    Dead,
    CoordinateBlocked(ShapeCoordinateBlock),
    NonFiniteVisualState,
    InsideDeadZone,
    Moved,
    Snapped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BattleFairyFollowEffect {
    AroundMove {
        message_type: u32,
        player_id: i32,
        object_type: i32,
        x: u32,
        y: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairyDeathOutcome {
    MissingHeadgear,
    NotBattleFairy,
    Alive,
    Died,
}

/// Живые war-soul поля прежнего `CPlayer`, которыми владеют правила этого
/// файла. Делегаты старого владельца строят view по disjoint-полям. Само
/// пространственное действие областной карты применяет `CGame`-координатор.
pub struct BattleFairyWarSoul<'a> {
    pub summoned: &'a mut bool,
    pub state: &'a mut u32,
    pub recall: &'a mut bool,
    pub died: &'a mut bool,
    pub visual_x_bits: &'a mut u32,
    pub visual_y_bits: &'a mut u32,
    pub point: &'a mut WarSoulPoint,
}

/// Операция над GAP_BF-свойствами живого головного предмета для
/// `revive_battle_fairy`: presence чтения и сама запись
/// `set_addon_property_value_core` остаются у прежнего владельца equipment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairyHeadgearOperation {
    Read(i32),
    Write(i32, i32),
}

/// Результат player-части SummonBF до конверта в прежний журнал эффектов.
#[must_use = "resolution хранит точный порядок адресных broadcast и property effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleFairySummonResolution {
    pub outcome: BattleFairySummonOutcome,
    pub spatial_action: Option<BattleFairyWarSoulAction>,
    pub effects: Vec<BattleFairySummonEffect>,
}

/// Результат одного прохода следования/очистки до конверта в прежний журнал.
#[must_use = "resolution следования содержит пространственное действие и рассылку движения"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleFairyFollowResolution {
    pub outcome: BattleFairyFollowOutcome,
    pub spatial_action: Option<BattleFairyWarSoulAction>,
    pub effects: Vec<BattleFairyFollowEffect>,
}

fn push_battle_fairy_summon_notification(
    effects: &mut Vec<BattleFairySummonEffect>,
    player_id: i32,
    string_id: &'static str,
    color: u32,
) {
    effects.push(BattleFairySummonEffect::Notification {
        player_id,
        string_id,
        color,
    });
}

/// Exact `SetWarSoulStaus`: around status публикуется по прежнему state,
/// затем любое значение кроме единицы нормализуется к нулю.
pub const fn set_war_soul_status(summoned: &mut bool, state: &mut u32, value: u32) -> bool {
    let broadcast_previous = *state == 1;
    if value == 1 {
        *summoned = true;
        *state = 1;
    } else {
        *summoned = false;
        *state = 0;
    }
    broadcast_previous
}

/// Исполняет player-часть `CBattleFairyContainer::SummonBF`. Spatial map
/// принадлежит `CServerRegion`, поэтому действие возвращается явным
/// tail-ом для `CGame`; ordered notify/broadcast/property effects там
/// сериализуются concrete wire после spatial mutation.
pub fn summon_battle_fairy(
    war_soul: &mut BattleFairyWarSoul<'_>,
    player_id: i32,
    battle_fairy_enabled: bool,
    mode: i32,
    has_pet: bool,
    active_skill_id: Option<u32>,
    mut read_headgear_property: impl FnMut(i32) -> Option<i32>,
    tile: impl FnOnce() -> (
        Result<i32, ShapeCoordinateBlock>,
        Result<i32, ShapeCoordinateBlock>,
    ),
) -> BattleFairySummonResolution {
    let mut resolution = BattleFairySummonResolution {
        outcome: BattleFairySummonOutcome::IgnoredMode,
        spatial_action: None,
        effects: Vec::new(),
    };
    if !battle_fairy_enabled {
        resolution.outcome = BattleFairySummonOutcome::FeatureDisabled;
        push_battle_fairy_summon_notification(&mut resolution.effects, player_id, "ZHGS0023", 0xffff_ffff);
        return resolution;
    }
    if mode == 1 && *war_soul.state == 1 {
        resolution.outcome = BattleFairySummonOutcome::AlreadySummoned;
        push_battle_fairy_summon_notification(&mut resolution.effects, player_id, "ZHGS0024", 0xffff_ffff);
        return resolution;
    }
    if mode == -1 && *war_soul.recall {
        resolution.outcome = BattleFairySummonOutcome::AlreadyRecalled;
        push_battle_fairy_summon_notification(&mut resolution.effects, player_id, "ZHGS0025", 0xffff_ffff);
        return resolution;
    }
    let Some(battle_fairy_flag) = read_headgear_property(GAP_BF_BATTLE_FAIRY) else {
        resolution.outcome = BattleFairySummonOutcome::MissingHeadgear;
        return resolution;
    };
    if battle_fairy_flag != 1 {
        resolution.outcome = BattleFairySummonOutcome::InvalidHeadgear;
        push_battle_fairy_summon_notification(&mut resolution.effects, player_id, "ZHGS0009", 0xffff_ffff);
        return resolution;
    }
    if read_headgear_property(GAP_BF_HP).unwrap_or_default() < 1 {
        resolution.outcome = BattleFairySummonOutcome::NoHitPoints;
        push_battle_fairy_summon_notification(&mut resolution.effects, player_id, "ZHGS0026", 0xffff_0000);
        return resolution;
    }
    if has_pet {
        resolution.outcome = BattleFairySummonOutcome::ActivePet;
        push_battle_fairy_summon_notification(&mut resolution.effects, player_id, "ZHGS0027", 0xffff_ffff);
        return resolution;
    }
    if active_skill_id == Some(MONSTER_TAMING_SKILL_ID) {
        resolution.outcome = BattleFairySummonOutcome::MonsterTamingActive;
        push_battle_fairy_summon_notification(&mut resolution.effects, player_id, "ZHGS0028", 0xffff_ffff);
        return resolution;
    }
    let player_position = match tile() {
        (Ok(x), Ok(y)) => WarSoulPoint { x, y },
        (Err(error), _) | (_, Err(error)) => {
            resolution.outcome = BattleFairySummonOutcome::CoordinateBlocked(error);
            return resolution;
        }
    };

    match mode {
        1 => {
            *war_soul.summoned = true;
            *war_soul.state = 1;
            *war_soul.recall = false;
            *war_soul.died = false;
            *war_soul.visual_x_bits = (player_position.x as f32).to_bits();
            *war_soul.visual_y_bits = (player_position.y as f32).to_bits();
            resolution.outcome = BattleFairySummonOutcome::Summoned;
            resolution.spatial_action = Some(BattleFairyWarSoulAction::SetPosition {
                previous: *war_soul.point,
                target: player_position,
            });
            resolution.effects.push(BattleFairySummonEffect::AroundMessage {
                message_type: BATTLE_FAIRY_MOVE_MESSAGE_TYPE,
                player_id,
                values: vec![player_id, 700, player_position.x, player_position.y],
            });
            // `SetWarSoulStaus(1)` наблюдает уже записанный state `1` и
            // поэтому публикует exact `0xbf930 {400, player_id}`.
            let _broadcast_previous = set_war_soul_status(war_soul.summoned, war_soul.state, 1);
            resolution.effects.push(BattleFairySummonEffect::AroundMessage {
                message_type: BATTLE_FAIRY_STATUS_MESSAGE_TYPE,
                player_id,
                values: vec![400, player_id],
            });
            resolution.effects.push(BattleFairySummonEffect::AroundMessage {
                message_type: BATTLE_FAIRY_SUMMON_MESSAGE_TYPE,
                player_id,
                values: vec![400, 1],
            });
        }
        -1 => {
            *war_soul.summoned = false;
            *war_soul.state = 0;
            *war_soul.recall = true;
            *war_soul.died = false;
            *war_soul.visual_x_bits = (-1.0f32).to_bits();
            *war_soul.visual_y_bits = (-1.0f32).to_bits();
            resolution.outcome = BattleFairySummonOutcome::Recalled;
            resolution.spatial_action = Some(BattleFairyWarSoulAction::Delete {
                previous: *war_soul.point,
                player_position,
            });
            resolution.effects.push(BattleFairySummonEffect::AroundMessage {
                message_type: BATTLE_FAIRY_STATUS_MESSAGE_TYPE,
                player_id,
                values: vec![400, -1],
            });
        }
        _ => {}
    }
    resolution
        .effects
        .push(BattleFairySummonEffect::PropertiesChanged { player_id });
    resolution
}

/// Завершает принадлежащий `CGame` хвост области: `spatial_applied`
/// означает найденную нужную area, а не изменение её map entry.
pub const fn apply_war_soul_action(
    point: &mut WarSoulPoint,
    action: BattleFairyWarSoulAction,
    spatial_applied: bool,
) {
    match action {
        BattleFairyWarSoulAction::SetPosition { target, .. } if spatial_applied => {
            *point = target;
        }
        BattleFairyWarSoulAction::Delete {
            player_position, ..
        } if spatial_applied => {
            *point = player_position;
        }
        BattleFairyWarSoulAction::SetPosition { .. }
        | BattleFairyWarSoulAction::Delete { .. } => {}
    }
}

/// Active WarSoul tail `CPlayer::OnEnterRegion`: visual float координаты
/// возвращаются к клетке хозяина; spatial point применяет координатор
/// после target-area gate и End(int,0) выбранного навыка.
pub fn prepare_war_soul_region_entry(
    state: u32,
    visual_x_bits: &mut u32,
    visual_y_bits: &mut u32,
    point: WarSoulPoint,
    tile: impl FnOnce() -> Option<(i32, i32)>,
) -> Option<(BattleFairyWarSoulAction, u32, u32)> {
    if state != 1 {
        return None;
    }
    let (x, y) = tile()?;
    let target = WarSoulPoint { x, y };
    *visual_x_bits = (target.x as f32).to_bits();
    *visual_y_bits = (target.y as f32).to_bits();
    Some((
        BattleFairyWarSoulAction::SetPosition {
            previous: point,
            target,
        },
        *visual_x_bits,
        *visual_y_bits,
    ))
}

/// Один проход живой ветви `ComputeWarSoulXY`. `Some(false)` означает
/// найденный текущий навык боевой феи с `IsRestored()==0`; `None` точно
/// соответствует отсутствующему навыку и не блокирует следование.
pub fn compute_war_soul_xy(
    war_soul: &mut BattleFairyWarSoul<'_>,
    player_id: i32,
    current_war_soul_skill_restored: Option<bool>,
    tile: impl FnOnce() -> (
        Result<i32, ShapeCoordinateBlock>,
        Result<i32, ShapeCoordinateBlock>,
    ),
) -> BattleFairyFollowResolution {
    let mut resolution = BattleFairyFollowResolution {
        outcome: BattleFairyFollowOutcome::NotSummoned,
        spatial_action: None,
        effects: Vec::new(),
    };
    if current_war_soul_skill_restored == Some(false) {
        resolution.outcome = BattleFairyFollowOutcome::ActiveSkill;
        return resolution;
    }
    if *war_soul.state != 1 {
        return resolution;
    }
    let (tile_x, tile_y) = match tile() {
        (Ok(x), Ok(y)) => (x, y),
        (Err(error), _) | (_, Err(error)) => {
            resolution.outcome = BattleFairyFollowOutcome::CoordinateBlocked(error);
            return resolution;
        }
    };
    let current_x = tile_x as f32;
    let current_y = tile_y as f32;
    let mut visual_x = f32::from_bits(*war_soul.visual_x_bits);
    let mut visual_y = f32::from_bits(*war_soul.visual_y_bits);
    let delta_x = current_x - visual_x;
    let delta_y = current_y - visual_y;
    let distance = (delta_x * delta_x + delta_y * delta_y).sqrt().abs();
    if !distance.is_finite() {
        resolution.outcome = BattleFairyFollowOutcome::NonFiniteVisualState;
        return resolution;
    }
    if distance < 0.5 {
        resolution.outcome = BattleFairyFollowOutcome::InsideDeadZone;
        return resolution;
    }

    let (target, outcome) = if distance <= 5.0 {
        let coefficient = if distance > 3.75 {
            0.265f32
        } else if distance > 0.75 {
            0.065f32
        } else {
            0.045f32
        };
        let step = distance * (coefficient + coefficient);
        if (current_x - visual_x).abs() > 0.1 {
            visual_x = if current_x <= visual_x {
                visual_x - step
            } else {
                visual_x + step
            };
        }
        if (current_y - visual_y).abs() > 0.1 {
            visual_y = if current_y <= visual_y {
                visual_y - step
            } else {
                visual_y + step
            };
        }
        (
            WarSoulPoint {
                // EXE временно ставит x87 RC=truncate перед обоими fistp.
                x: visual_x.trunc() as i32,
                y: visual_y.trunc() as i32,
            },
            BattleFairyFollowOutcome::Moved,
        )
    } else {
        visual_x = current_x;
        visual_y = current_y;
        (
            WarSoulPoint {
                x: tile_x,
                y: tile_y,
            },
            BattleFairyFollowOutcome::Snapped,
        )
    };
    *war_soul.visual_x_bits = visual_x.to_bits();
    *war_soul.visual_y_bits = visual_y.to_bits();
    resolution.outcome = outcome;
    resolution.spatial_action = Some(BattleFairyWarSoulAction::SetPosition {
        previous: *war_soul.point,
        target,
    });
    resolution.effects.push(BattleFairyFollowEffect::AroundMove {
        message_type: BATTLE_FAIRY_MOVE_MESSAGE_TYPE,
        player_id,
        object_type: 700,
        x: visual_x.to_bits(),
        y: visual_y.to_bits(),
    });
    resolution
}

/// Мёртвая ветвь сразу после `CMoveShape::AI`: пространственная позиция
/// получает точное `(-1,-1)`, обе визуальные координаты `float` становятся
/// `-1.0`, но исходник не публикует пакет движения вокруг.
pub fn clear_dead_war_soul_xy(war_soul: &mut BattleFairyWarSoul<'_>) -> BattleFairyFollowResolution {
    let target = WarSoulPoint { x: -1, y: -1 };
    *war_soul.visual_x_bits = (-1.0f32).to_bits();
    *war_soul.visual_y_bits = (-1.0f32).to_bits();
    BattleFairyFollowResolution {
        outcome: BattleFairyFollowOutcome::Dead,
        spatial_action: Some(BattleFairyWarSoulAction::SetPosition {
            previous: *war_soul.point,
            target,
        }),
        effects: Vec::new(),
    }
}

/// Периодический префикс `CPlayer::AI`: нулевой HP надетой боевой феи при
/// каждом проходе повторно нормализует четыре поля состояния и вызывает
/// `PropertiesChanged`. Исходник не удаляет устаревшую запись карты области
/// и не рассылает состояние.
pub fn refresh_battle_fairy_death(
    war_soul: &mut BattleFairyWarSoul<'_>,
    mut read_headgear_property: impl FnMut(i32) -> Option<i32>,
) -> BattleFairyDeathOutcome {
    let Some(battle_fairy_flag) = read_headgear_property(GAP_BF_BATTLE_FAIRY) else {
        return BattleFairyDeathOutcome::MissingHeadgear;
    };
    if battle_fairy_flag != 1 {
        return BattleFairyDeathOutcome::NotBattleFairy;
    }
    if read_headgear_property(GAP_BF_HP).unwrap_or_default() != 0 {
        return BattleFairyDeathOutcome::Alive;
    }
    *war_soul.summoned = false;
    *war_soul.state = 0;
    set_battle_fairy_recall(war_soul.recall, true);
    set_battle_fairy_died(war_soul.died, true);
    BattleFairyDeathOutcome::Died
}

/// Player-owned mutation `ReviveBattleFairy`; client goods/state wire
/// остаётся у вызывающего `CGame`, уже после изменения всех полей. Машинное
/// вхождение (inline в CGame-handler) — UNKNOWN, поведение — снятая модель.
pub fn revive_battle_fairy(
    war_soul: &mut BattleFairyWarSoul<'_>,
    mut headgear: impl FnMut(BattleFairyHeadgearOperation) -> Option<i32>,
) -> bool {
    let Some(hit_points) = headgear(BattleFairyHeadgearOperation::Read(GAP_BF_HP)) else {
        return false;
    };
    if hit_points > 0 {
        return false;
    }
    let maximum_hp = headgear(BattleFairyHeadgearOperation::Read(GAP_BF_MAX_HP)).unwrap_or_default();
    let maximum_mp = headgear(BattleFairyHeadgearOperation::Read(GAP_BF_MAX_MP)).unwrap_or_default();
    let _ = headgear(BattleFairyHeadgearOperation::Write(GAP_BF_HP, maximum_hp));
    let _ = headgear(BattleFairyHeadgearOperation::Write(GAP_BF_MP, maximum_mp));
    *war_soul.recall = true;
    *war_soul.died = false;
    *war_soul.summoned = false;
    *war_soul.state = 0;
    true
}

pub const fn set_battle_fairy_recall(recall: &mut bool, value: bool) {
    *recall = value;
}

pub const fn set_battle_fairy_died(died: &mut bool, value: bool) {
    *died = value;
}

/// Scalar tail `ApplyDeathFinalWarSoulReset`. В отличие от гибели самой
/// боевой феи смерть хозяина снимает summon/state, разрешает recall и
/// очищает `bBFDied`.
pub const fn reset_war_soul_after_player_death(
    summoned: &mut bool,
    state: &mut u32,
    recall: &mut bool,
    died: &mut bool,
) {
    *summoned = false;
    *state = 0;
    *recall = true;
    *died = false;
}

/// Прямое чтение флага `m_BaseProperty.bFairyContainerEnabled`; само поле
/// остаётся у hub `CPlayer` до его переноса (объявленный шов значением).
pub const fn fairy_container_enabled(container_enabled: bool) -> bool {
    container_enabled
}

/// Граница восстановления `m_BaseProperty.bFairyContainerEnabled` из
/// persisted player snapshot; default остаётся выключенным до decode.
pub const fn set_fairy_container_enabled(container_enabled: &mut bool, value: bool) {
    *container_enabled = value;
}
