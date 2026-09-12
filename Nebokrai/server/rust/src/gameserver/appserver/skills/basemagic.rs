//! Базовая магическая атака CBaseMagic (ID3) и общие идентификаторы таблицы.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/basemagic.cpp.
//! Begin 0x005B3DC0, Check 0x005B4700, AI 0x005B4330 и Summon 0x005B49A0
//! связаны через baseprojectilecast/baseprojectilecheck с CArchery:
//! единый зарегистрированный lifecycle игрока/монстра, без player-only копии.
//! CAN перед смертью/самоцелью, два свежих GetS при повороте, точный MAX
//! без +1 и дополнительное чтение EM остаются явными различиями магии.
//! End 0x005AE7A0 сбрасывает фазу, но сохраняет время полёта экземпляра.
//! Отдельный basemagicphalanx хранит MIN/MAX/EM и владеет формулой удара;
//! поиск игрока по attacker ID не ограничен типом master.

use crate::gameserver::appserver::build::BUILD_OBJECT_TYPE;
use crate::gameserver::appserver::citygate::CITY_GATE_OBJECT_TYPE;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, ServerRegionOwner};
use crate::nets::netserver::message::CMessage;

const PLAYER_TYPE: i32 = 400;
const NPC_TYPE: i32 = 500;
const MONSTER_TYPE: i32 = 600;

pub(crate) const BASE_MAGIC_SKILL_ID: u32 = 3;
pub(crate) const BASE_MAGIC_EFFECT_MESSAGE: i32 = 0x000b_fe01;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
pub(crate) const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
pub(crate) const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
pub(crate) const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;
pub(crate) const SKILL_USAGE_ELEMENT_MODIFIER: u32 = 20_015;
pub(crate) const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;
pub(crate) const SKILL_USAGE_SUMMONED_SPEED: u32 = 30_002;

/// Типы, которые исходный `CState::GetSufferer` разрешает в object-target
/// перегрузках семейства базовой магии. NPC доходит до owner-а и уже там
/// отклоняется как мёртвый; постройки продолжают region-owned combat path.
pub(crate) const fn is_base_magic_object_target_type(object_type: i32) -> bool {
    matches!(object_type, PLAYER_TYPE | NPC_TYPE | MONSTER_TYPE)
        || object_type == BUILD_OBJECT_TYPE as i32
        || object_type == CITY_GATE_OBJECT_TYPE as i32
}

impl CGame {
    /// Общий legacy failure-пакет базовой магии и стрельбы остаётся рядом с
    /// семейством навыков; `CGame` предоставляет только фактическую сетевую доставку.
    pub(crate) fn send_base_magic_failure(&self, player_id: i32, action: u8) {
        let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
        message.add_byte(0);
        message.add_byte(action);
        let _ = message.send_to_player(self.net_server(), player_id);
    }
}

pub(crate) fn execute_owned_monster_base_magic<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    super::baseprojectilecast::execute_owned_monster_base_projectile::<BASE_MAGIC_SKILL_ID, Runtime>(
        game, owner, monster_id, target, skill_level, runtime)
}
