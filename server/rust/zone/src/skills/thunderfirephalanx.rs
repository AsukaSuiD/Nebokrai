//! Движущийся громовой огонь `CThunderFirePhalanx` (`0x322`): форма
//! предметного навыка `CItemSkill_2`. Источник: точная пара `gameserver.exe`
//! (SHA-256 `4F5C98E0…`) + `GameServer.pdb` (RSDS match), исходный владелец
//! `appserver/skills/thunderfirephalanx.cpp/.h`.
//!
//! Машинные якоря (VA = RVA + 0x400000): ctor `0x5E11E0` (0xFC байта, 10
//! аргументов; статические `CScope` 1×1 — `g_dwLength/g_dwHeight` = 1,
//! `g_bScope` из 9 байт с маской только (0,0) — конструктор области
//! `0x5E98B0` + маска `0x5E9960`), vtable `0x65EF2C`; AI `0x5E1970`;
//! обход области `0x5E1670` (scope-ячейки вокруг клетки, список
//! уже-атакованных, self-пропуск, допуск vcall+0x134 для player-целей);
//! Attack по цели `0x5E1570` (IsDied → пропуск; Calculate; OnBeenAttacked
//! vcall+0x15C; без IncreaseRp); Calculate `0x5E1310` (`GetWeaponModifier`
//! `0x42D980` через vcall+0x184 без blast-клампа; x87 soul-формула
//! `trunc((soul_variable * soul_count * 0.01 + 1) * damage)`; hit = 0x64;
//! два вызова MSVCRT RNG `0x41CBA0`: диапазон, затем крит с FISTP);
//! AddToByteArray `0x5FBD20` / DecordFromByteArray `0x5FBFA0` (префикс 5
//! dword: id 0x322, уровень [+0xD4], master type/id [+0x84]/[+0x88],
//! remained `0x5E9870`, затем `CShape`); ReplaceAffectRegion `0x5FECA0`
//! (`ret 0xC` — собственное пустое тело, ICF с CWeakPhalanx отсутствует);
//! End `0x5E9DC0`, ForceMove vcall+0xA0 `0x5E9AF0`.
//!
//! Активация только предметная: единственный вызов ctor — из
//! `CItemSkill_2::Summon` (xref `0x515D6F`); в `QuerySkill` форма не
//! регистрируется. Item-владелец передаёт путь, `speed` и soul-пару
//! (снимок SoulCollect); регистрация и доставка снимка остаются
//! межвладельческой координацией старого пакета.
//!
//! Машинный факт (FIX, якорь AI `0x5E1970`):
//!
//! ```text
//! 005e19e8: cmp  eax, ecx          ; now ? started + idx*speed
//! 005e19ea: jb   0x5e1a80          ; не due — пропуск без End
//! 005e1a0e: test ebx, ebx          ; RTTI-регион из [U+0x40]
//! 005e1a10: je   0x5e19f0.../0x5e1a80 ; region NULL — пропуск без End
//! 005e1a1d: cmp  edi, eax          ; idx < count ?
//! 005e1a1f: jb   0x5e1a2d
//! ...      call [edx + 0xac]        ; End ТОЛЬКО в ветке due+region
//! ```
//!
//! Верхний Expired-гейт машинно: `exp(started+lifetime < now)` /
//! scope-патч [+0xBC] == 0 / данные пути [+0xE0] == NULL / `count == 0` —
//! и НЕ `idx >= count`: исчерпание пути завершает форму только в due-ветке
//! с разрешённым регионом (тот же vcall+0xAC, что и у expired-веток).
//! Прежняя реконструкция выносила `current_position >= path.len()` в верхний
//! гейт и завершала форму досрочно даже вне due; условие снято отсюда,
//! исчерпание проверяется в точке due (см. `tick`). После End
//! по idx-исчерпанию машинный хвост выполняет разовый ForceMove — блок уже
//! закрыт первым проходом, различия нет. Разовый ForceMove в последнюю
//! ячейку с длительностью `count * speed` остаётся разовым (`[+0xF8]`).
//!
//! Объявленные швы переноса (не расхождения): hub-трейты
//! `ThunderFirePhalanxGame`/`ThunderFirePhalanxPlayer` — переходные фасады
//! прежнего владельца `CGame`/`CPlayer` (реализация у делегата старого
//! пакета): свойства игрока, weapon-modifier и RNG; применение клетки и
//! End формы — региональный runtime старого пакета. Клиентский снимок —
//! общий пятипольный префикс `summonshape`-конверта (`Id`=0x322).
//!
//! UNKNOWN/объявленная неполнота: нормализация blast/knock-back scale из
//! прежней реконструкции в теле Calculate НЕ обнаружена машинным дампом
//! `0x5E1310`; блок не добавляется и не удаляется, вопрос остаётся открытым
//! (см. тело `calculate_owned_thunder_fire_attack`).

use nebokrai_shared::values::CGuid;

use crate::combat::{
    AttackInformation, AttackPower, AttackPowerType, MasterInfo, PlayerCombatProperties,
    truncate_original,
};
use crate::regions::ShapeIdentity;
use crate::regions::shape::{CShape, SHAPE_CHANGE_DELETE};

use super::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};

/// ID предметного навыка-владельца (ctor пишет [+0xB8] = 0x322).
pub const ITEM_SKILL_2_ID: u32 = 0x322;

/// Снимок игрока-источника формулы: боевые свойства, круг и уровень
/// (`GetWeaponModifier` с фабрикой предметов — шов владельца).
pub trait ThunderFirePhalanxPlayer {
    fn combat_properties(&self) -> PlayerCombatProperties;

    fn occupation(&self) -> u8;

    fn level(&self) -> u8;
}

/// Переходные фасады прежнего владельца `CGame` для формулы формы;
/// имена сохраняют исходную операцию.
pub trait ThunderFirePhalanxGame {
    type Player: ThunderFirePhalanxPlayer;

    /// `FindPlayer` по master_id (отсутствие игрока — пустая атака).
    fn find_player(&self, player_id: i32) -> Option<&Self::Player>;

    /// `weapon_modifier` найденного игрока против уровня цели
    /// (`GetWeaponModifier 0x42D980`); чтение фабрики — у владельца.
    fn thunder_fire_weapon_modifier(
        &self,
        player: &Self::Player,
        target_level: i32,
        divisor: f32,
        minimum_factor: f32,
    ) -> f32;

    /// Базовые делитель/пол оружейного урона (`CGlobeSetup`).
    fn weapon_damage_factors(&self) -> (f32, f32);

    /// Нормализация пяти scale (см. UNKNOWN в шапке).
    fn base_combat_scales(&self) -> [f32; 5];

    fn skill_random_below(&mut self, maximum: i32) -> i32;
}

/// Исход одного тика AI `0x5E1970` для регионального runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThunderFirePhalanxTick {
    Pending,
    Active {
        force_move: Option<(i32, i32, u32)>,
        scan: Option<(i32, i32, u32)>,
    },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CThunderFirePhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_modifier: i32,
    path: Vec<(i32, i32)>,
    speed_ms: u32,
    soul_count: i32,
    soul_variable: u32,
    current_position: usize,
    force_moved: bool,
}

impl CThunderFirePhalanx {
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
        path: Vec<(i32, i32)>,
        speed_ms: u32,
        soul_count: i32,
        soul_variable: u32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE,
            id,
            ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape, master, started_at_ms, lifetime_ms, skill_level, minimum_attack,
            maximum_attack, element_modifier, path, speed_ms, soul_count, soul_variable,
            current_position: 0, force_moved: false,
        }
    }

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn master(&self) -> MasterInfo { self.master }

    pub fn finish(&mut self) { self.shape.set_change_state(SHAPE_CHANGE_DELETE); }

    /// Тик AI `0x5E1970`. Верхний гейт: exp/пустой путь (машинные
    /// `started+lifetime < now`, данные/count пути — FIX #2: БЕЗ
    /// исчерпания). Исчерпание пути завершает форму только в due-точке
    /// (тот же machine-End), до неё — разовый ForceMove в последнюю клетку.
    pub fn tick(&mut self, now: u32) -> ThunderFirePhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now || self.path.is_empty() {
            self.finish();
            return ThunderFirePhalanxTick::Expired;
        }
        let force_move = if self.force_moved {
            None
        } else {
            self.force_moved = true;
            let &(x, y) = self.path.last().expect("путь непуст");
            Some((x, y, (self.path.len() as u32).wrapping_mul(self.speed_ms)))
        };
        if self.started_at_ms.wrapping_add((self.current_position as u32).wrapping_mul(self.speed_ms)) > now {
            return if force_move.is_some() {
                ThunderFirePhalanxTick::Active { force_move, scan: None }
            } else {
                ThunderFirePhalanxTick::Pending
            };
        }
        // FIX #2: idx >= count → End только здесь (due-ветка после гейта
        // региона у владельца); в верхний гейт условие не поднимается.
        if self.current_position >= self.path.len() {
            self.finish();
            return ThunderFirePhalanxTick::Expired;
        }
        let (x, y) = self.path[self.current_position];
        self.current_position = self.current_position.wrapping_add(1);
        ThunderFirePhalanxTick::Active { force_move, scan: Some((x, y, now)) }
    }

    /// Клиентский снимок входа 0xBF502: префикс 5 dword (`0x5FBD20`) +
    /// базовый `CShape`.
    pub fn encode_client_snapshot(&self, now_milliseconds: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, ITEM_SKILL_2_ID as i32, self.skill_level,
            self.master.master_type, self.master.master_id,
            self.started_at_ms, self.lifetime_ms, now_milliseconds,
        )
    }
}

/// Calculate `0x5E1310`: weapon-modifier без blast-клампа, элементный
/// урон из снимка source, x87 soul-формула, два RNG. Отсутствие игрока
/// оставляет пустую атаку, не отменяя получение целью.
pub fn calculate_owned_thunder_fire_attack<Game: ThunderFirePhalanxGame>(
    game: &mut Game,
    phalanx: &CThunderFirePhalanx,
    target_level: u8,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let player = game.find_player(phalanx.master.master_id)?;
    let mut combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let (divisor, minimum) = game.weapon_damage_factors();
    let factor = game.thunder_fire_weapon_modifier(
        player, i32::from(target_level), divisor, minimum,
    );
    let width_delta = phalanx.maximum_attack.wrapping_sub(phalanx.minimum_attack);
    let width = if width_delta < 0 { width_delta.wrapping_neg() } else { width_delta }.wrapping_add(1);
    let mut damage = phalanx.element_modifier
        .wrapping_mul(combat.element_modify).wrapping_div(100)
        .wrapping_add(combat.add_element_attack as i32)
        .wrapping_add(game.skill_random_below(width))
        .wrapping_add(phalanx.minimum_attack);
    if phalanx.soul_count != 0 && phalanx.soul_variable != 0 {
        damage = truncate_original(
            (f64::from(phalanx.soul_variable as i32) * f64::from(phalanx.soul_count)
                * f64::from(0.01_f32) + f64::from(1.0_f32))
                * f64::from(damage),
        );
    }
    damage = damage.max(0);
    let mut attack = AttackInformation {
        skill_id: ITEM_SKILL_2_ID,
        skill_level: phalanx.skill_level as u8,
        attacker_type: phalanx.master.master_type,
        attacker_id: phalanx.master.master_id,
        attacker_team_id: phalanx.master.master_team_id,
        attacker_faction_id: phalanx.master.master_guild_id,
        attacker_union_id: phalanx.master.master_union_id,
        hit_modifier: 100,
        damage_factor: factor,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 }],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let rate = combat.critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
    // UNKNOWN: нормализация scale из прежней реконструкции; в машинном
    // дампе Calculate 0x5E1310 не обнаружена (см. шапку).
    let [blast_attack, blast_defense, element_blast_attack, element_blast_defense, full_miss] =
        game.base_combat_scales();
    if combat.blast_attack_scale() < 1.0 { combat.blast_attack_scale_bits = blast_attack.max(1.0).to_bits(); }
    if combat.blast_defense_scale() < 0.01 { combat.blast_defense_scale_bits = blast_defense.max(0.01).to_bits(); }
    if combat.element_blast_attack_scale() < 1.0 { combat.element_blast_attack_scale_bits = element_blast_attack.max(1.0).to_bits(); }
    if combat.element_blast_defense_scale() < 0.01 { combat.element_blast_defense_scale_bits = element_blast_defense.max(0.01).to_bits(); }
    if combat.full_miss_scale() < 0.01 { combat.full_miss_scale_bits = full_miss.max(0.01).to_bits(); }
    Some((attack, combat, occupation, attacker_level))
}
