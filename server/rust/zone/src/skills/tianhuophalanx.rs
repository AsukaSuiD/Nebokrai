//! Небесный огонь CTianhuoPhalanx — живая область боевого духа навыка
//! CTianhuo (0x21A; вызывающий путь — `skills/tianhuo.rs`).
//!
//! Источник: точная пара `gameserver.exe` (SHA-256 `4F5C98E0…`) +
//! `GameServer.pdb` (RSDS match), `appserver/skills/tianhuophalanx.cpp`.
//! Адресная конвенция факт-листа волны: истинный RVA (pub off + 0x1000;
//! VA = RVA + 0x400000). Прежний переходный владелец —
//! `src/gameserver/appserver/skills/tianhuophalanx.rs`; живое тело области
//! перенесено буквально порцией T2 «BF-облака области»; run-делегации
//! (скан клетки, отправка `0xBF504`, свёртка старой области) остаются у
//! прежнего владельца.
//!
//! Машинный факт (MATCH по снятой доказательной базе порции T2):
//!
//! - ctor `0x1E5890` (VA `0x5E5890`): 6 аргументов без id и часов; полный
//!   маппинг полей — PARTIAL (досмотр только уже видимых записей).
//! - AI `0x1E5C00` (VA `0x5E5C00`): пока собственный срок жизни не истёк,
//!   область на каждом проходе просматривает свою клетку в исходном порядке
//!   региона; после каждой допустимой атаки она помечается на удаление и
//!   немедленно отправляет `0xBF504` (End→BF504). Один проход всё ещё
//!   обрабатывает уже полученный снимок клетки, поэтому пакет удаления
//!   может повториться.
//! - Replace/AddTo/Decord ICF `0x1F54A0`/`0x1F54D0`/`0x1FF7C0`
//!   (VA `0x5F54A0`/`0x5F54D0`/`0x5FF7C0`) — та же группа, что у
//!   weak/thunderblow семейства (`CWeakPhalanx`, `CGodPunishmentPhalanx`,
//!   `CThunderBlowPhalanx`, `CTianhuoPhalanx` — таблица 11 тел и её
//!   доказательная база конверта: `skills/summonshape.rs`): wire содержит
//!   ID/level/Master и живое оставшееся время перед CShape; совпавшая
//!   старая область завершается до регистрации новой (шов прежнего
//!   владельца). Свёрку с владельцем CWeakPhalanx см. там же: расхождений
//!   по группе не выявлено.
//! - Формула: при наличии предмета в слоте 10 делает один вызов legacy RNG
//!   до повторного чтения боевого духа и свойств навыка; поздний отказ
//!   сохраняет нулевую запись урона. Слагаемое боевого духа и случайная
//!   база складываются в x87 до единственного усечения в `i64`, после
//!   которого читаются младшие 32 бита.
//!
//! Объявленные швы переноса (не расхождения): hub `battlefairyskill::
//! BattleFairyGame` — разрешение игрока, слот-10 equipment
//! (`battle_fairy_equipment_addon` повторяет прежнюю цепочку
//! find_player → `equipment().get_goods(10)` → addon с тем же допуском
//! отсутствия предмета), WarSoul (`battle_fairy_war_soul_addon`),
//! таблица свойств и RNG (`skill_random_below`). Поиск целей и применение
//! результата к независимым владельцам остаются у прежнего `CGame`.
//! UNKNOWN списком: маппинг 6 аргументов ctor `0x1E5890` (PARTIAL, см.
//! выше).

use nebokrai_shared::values::CGuid;

use crate::combat::{
    AttackInformation, AttackPower, AttackPowerType, MasterInfo, PlayerCombatProperties,
    truncate_original_i64_low,
};
use crate::content::goods::GAP_BF_SPRITE;
use crate::regions::ShapeIdentity;
use crate::regions::shape::{CShape, SHAPE_CHANGE_DELETE};

use super::battlefairyskill::{BattleFairyGame, BattleFairyPlayer};
use super::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use super::tianhuo::{TIANHUO_SKILL_ID, TIANHUO_TARGET_DAMAGE_FACTOR_PROPERTY};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TianhuoPhalanxTick {
    Scan { sampled_at_ms: u32 },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CTianhuoPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    _element_modifier: i32,
}


pub fn calculate_owned_tianhuo_attack<Game: BattleFairyGame>(game: &mut Game, phalanx: &CTianhuoPhalanx) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let master = phalanx.master();
    let player = game.find_player(master.master_id)?;
    let _ = game.battle_fairy_equipment_addon(master.master_id, GAP_BF_SPRITE)?;
    let combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let mut attack = AttackInformation::for_master(master);
    attack.skill_id = TIANHUO_SKILL_ID;
    attack.skill_level = phalanx.skill_level as u8;
    attack.hit_modifier = 100;
    let width = phalanx.maximum_attack.wrapping_sub(phalanx.minimum_attack)
        .wrapping_abs().wrapping_add(1);
    let rolled_attack = game.skill_random_below(width).wrapping_add(phalanx.minimum_attack);
    let sprite = game.battle_fairy_war_soul_addon(master.master_id, GAP_BF_SPRITE);
    let properties = game.skill_base_properties(TIANHUO_SKILL_ID, phalanx.skill_level);
    let damage = if let (Some(sprite), Some(properties)) = (sprite, properties) {
        let target_damage_factor = properties.query_property(TIANHUO_TARGET_DAMAGE_FACTOR_PROPERTY);
        truncate_original_i64_low(
            f64::from(target_damage_factor) * f64::from(sprite) * 1.0e-6
                + f64::from(rolled_attack),
        ).max(0)
    } else {
        0
    };
    attack.damages.push(AttackPower {
        kind: AttackPowerType::Element,
        hp_damage: damage,
        mp_damage: 0,
    });
    Some((attack, combat, occupation, attacker_level))
}

impl CTianhuoPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        minimum_attack: i32,
        maximum_attack: i32,
        element_modifier: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE,
            id,
            ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape,
            master,
            started_at_ms,
            lifetime_ms,
            skill_level,
            minimum_attack,
            maximum_attack,
            _element_modifier: element_modifier,
        }
    }

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn master(&self) -> MasterInfo { self.master }
    pub const fn skill_level(&self) -> i32 { self.skill_level }

    pub fn set_center(&mut self, x: i32, y: i32) {
        self.shape.set_pos_xy_move_order(
            (f64::from(x) + 0.5) as f32, (f64::from(y) + 0.5) as f32,
        );
    }

    pub fn tick(&mut self, now_ms: u32) -> TianhuoPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            self.finish();
            TianhuoPhalanxTick::Expired
        } else {
            TianhuoPhalanxTick::Scan {
                sampled_at_ms: now_ms,
            }
        }
    }

    pub fn finish(&mut self) {
        self.shape.set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, TIANHUO_SKILL_ID as i32, self.skill_level,
            self.master.master_type, self.master.master_id,
            self.started_at_ms, self.lifetime_ms, now_milliseconds,
        )
    }
}
