//! Общая область CMeteorArrowPhalanx и CFallingStarPhalanx с wire-ID 0xCD.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/meteorarrowphalanx.cpp
//! и fallingstarphalanx.cpp. Два владельца отличаются только маской Initialize.
//!
//! Форма хранит неизменный снимок атаки и срок frequency*count+10 с DWORD
//! переполнением. Конструктор выделяет клетки; Initialize после SetTileXY
//! читает фактические X/Y и выбирает две координаты MSVCRT RNG для каждой
//! стрелы. Все три уровня и запасной первый уровень имеют полную маску:
//! 3×3 у MeteorArrow, 1×1 у FallingStar. Даже маска 1×1 потребляет оба RNG.
//! Маска после Initialize больше не нужна; хранится только полученный массив.
//!
//! AI читает часы срока, затем частоты. При наступлении частоты третьи часы
//! записываются до проверки числа стрел и разрешения фактического региона.
//! GetShapes сохраняет список одной клетки; свежий CMoveShape и player-допуск
//! проверяются перед каждым контактом. Счётчик увеличивается после всей
//! клетки даже при NULL регионе; после последней клетки немедленного End нет.
//! Общий региональный runtime держит форму опубликованной во время callbacks.
//!
//! Attack проверяет IsDied и сохраняет PK-снимок только для master типа 400.
//! Calculate не читает живого стрелка, таблицу навыка или weapon modifier:
//! единичный коэффициент, abs(MAX-MIN)+1, три неотрицательные компонента,
//! второй RNG критического удара и усечение расширенного произведения к нулю.
//! Контакт не начисляет RP, не накладывает яд и не устраняет повторные цели.
//!
//! Wire: skill/level/master type/id/remained, полный lifetime/frequency/count,
//! затем пары клеток и CShape. Vec заменяет массив и освобождает всю память;
//! статическая маска не требует отдельного выделенного CScope. Серверный decode
//! не имеет достигнутого caller-а и сохраняется адресно ниже.
//! Число стрел сохраняет DWORD-биты без знакового ограничения; нехватка памяти
//! остаётся фатальной границей, а переполнение размера native-массива не переносится.

use super::fightdefense::truncate_original;
use super::meteorarrow::METEOR_ARROW_SKILL_ID;
use nebokrai_shared::protocol::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_prefix};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_shared::values::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MeteorArrowScope {
    Meteor,
    FallingStar,
}

impl MeteorArrowScope {
    const fn side(self) -> i32 {
        match self { Self::Meteor => 3, Self::FallingStar => 1 }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CMeteorArrowPhalanx {
    shape: CShape,
    started_at_ms: u32,
    lifetime_ms: u32,
    frequency_ms: u32,
    attack: MeteorArrowAttack,
    cells: Vec<(i32, i32)>,
    last_attack_at_ms: u32,
    attack_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MeteorArrowAttack {
    master: MasterInfo,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_attack: i32,
    soul_attack: i32,
    critical_chance: i32,
    hit_modifier: i32,
}

impl CMeteorArrowPhalanx {
    #[allow(clippy::too_many_arguments, reason = "снимки соответствуют аргументам конструктора EXE")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, frequency_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32, element_attack: i32,
        soul_attack: i32, critical_chance: i32, hit_modifier: i32, arrow_count: u32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape, started_at_ms,
            lifetime_ms: frequency_ms.wrapping_mul(arrow_count).wrapping_add(10),
            frequency_ms,
            attack: MeteorArrowAttack {
                master, skill_level, minimum_attack, maximum_attack, element_attack,
                soul_attack, critical_chance, hit_modifier,
            },
            cells: vec![(0, 0); arrow_count as usize],
            last_attack_at_ms: 0, attack_count: 0,
        }
    }

    pub(super) fn initialize_cells(
        &mut self, scope: MeteorArrowScope, mut random_below: impl FnMut(i32) -> i32,
    ) {
        let side = scope.side();
        // Native FISTP при непредставимой координате даёт integer indefinite.
        let center_x = self.shape.get_tile_x().unwrap_or(i32::MIN);
        let center_y = self.shape.get_tile_y().unwrap_or(i32::MIN);
        let start_x = center_x.wrapping_sub(side >> 1);
        let start_y = center_y.wrapping_sub(side >> 1);
        for cell in &mut self.cells {
            let x = random_below(side);
            let y = random_below(side);
            *cell = (start_x.wrapping_add(x), start_y.wrapping_add(y));
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master }
    pub(crate) const fn attack_snapshot(&self) -> MeteorArrowAttack { self.attack }

    pub(crate) const fn expired_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms
    }

    pub(crate) const fn attack_due_at(&self, now_ms: u32) -> bool {
        self.last_attack_at_ms.wrapping_add(self.frequency_ms) < now_ms
    }

    pub(crate) fn mark_attack_at(&mut self, now_ms: u32) { self.last_attack_at_ms = now_ms; }

    pub(crate) fn attack_cells_finished(&self) -> bool {
        self.cells.len() as u32 <= self.attack_count
    }

    pub(crate) fn current_cell(&self) -> Option<(i32, i32)> {
        self.cells.get(self.attack_count as usize).copied()
    }

    pub(crate) fn advance_attack_cell(&mut self) {
        self.attack_count = self.attack_count.wrapping_add(1);
    }

    pub(crate) fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        let mut payload = encode_related_phalanx_prefix(
            METEOR_ARROW_SKILL_ID as i32, self.attack.skill_level,
            self.attack.master.master_type, self.attack.master.master_id,
            self.started_at_ms, self.lifetime_ms, now_milliseconds,
        );
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_u32(self.lifetime_ms);
        writer.write_u32(self.frequency_ms);
        writer.write_u32(self.cells.len() as u32);
        for &(x, y) in &self.cells {
            writer.write_i32(x);
            writer.write_i32(y);
        }
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }

}

impl MeteorArrowAttack {
    fn attack_master(self) -> MasterInfo {
        if self.master.master_type == 400 { return self.master; }
        MasterInfo {
            master_type: self.master.master_type, master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }
}

fn calculate_meteor_arrow_attack(
    game: &mut CGame, phalanx: MeteorArrowAttack,
) -> AttackInformation {
    let mut attack = AttackInformation::for_master(phalanx.attack_master());
    attack.skill_id = METEOR_ARROW_SKILL_ID;
    attack.skill_level = phalanx.skill_level as u8;
    attack.damage_modifier = 0;
    attack.damage_factor = 1.0;
    attack.hit_modifier = phalanx.hit_modifier;
    let width = phalanx.maximum_attack.wrapping_sub(phalanx.minimum_attack)
        .wrapping_abs().wrapping_add(1);
    let physical = phalanx.minimum_attack.wrapping_add(game.skill_random_below(width)).max(0);
    attack.damages = vec![
        AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
        AttackPower { kind: AttackPowerType::Element, hp_damage: phalanx.element_attack.max(0), mp_damage: 0 },
        AttackPower { kind: AttackPowerType::Soul, hp_damage: phalanx.soul_attack.max(0), mp_damage: 0 },
    ];
    if game.skill_random_below(100) < phalanx.critical_chance {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
    attack
}

pub(crate) fn apply_meteor_arrow_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, attack: MeteorArrowAttack,
    region_id: i32, target: ShapeIdentity, runtime: &mut Runtime,
) {
    if game.move_shape_health(region_id, target).is_none_or(|hp| hp == 0) { return; }
    let information = calculate_meteor_arrow_attack(game, attack);
    game.apply_owned_skill_contact(attack.attack_master(), target, region_id, information, runtime);
}

// Неиспользуемый server decode: Meteor VT+14 и CFallingStarPhalanx разделяют
// функцию 0x005F7010. Нет достигнутого server caller-а; клиентский encoder
// не заменяет её. RAW не задаёт корректный Rust-контракт владения старым массивом.
//
// FUNCTION: CFallingStarPhalanx::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// SOURCE: appserver/skills/fallingstarphalanx.cpp:316
// RVA: 0x001F7010
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
