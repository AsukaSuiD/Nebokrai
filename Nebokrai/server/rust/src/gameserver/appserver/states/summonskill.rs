//! Общий достигнутый хвост `CSummonSkill::End`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/states/summonskill.cpp`. При `End(1)` конкретный навык сначала
//! выполняет свой виртуальный `Summon`, после чего базовый `CSkill::End`
//! очищает текущий навык и фиксирует время восстановления. В достигнутых
//! владельцах виртуальный `Summon` является синхронным обновлением свойств
//! игрока; skill-specific состояние и движение остаются у конкретного owner-а.
//! При `End(0)` обновление свойств и cooldown не выполняются: общий хвост
//! только очищает текущий навык, а конкретный owner завершает свои флаги.

use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) fn abort_skill(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_current_skill_id(None);
    }
}

pub(crate) fn finish_summon_skill<Runtime, MarkUsed>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
    mark_used: MarkUsed,
) where
    Runtime: GameMainLoopRuntime,
    MarkUsed: FnOnce(&mut CPlayerAI, u32),
{
    let _ = game.update_player_properties(player_id, runtime);
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_current_skill_id(None);
    }
    mark_used(player_ai, runtime.now_milliseconds());
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\summonskill.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\summonskill.cpp

// ============================================================================
// FUNCTION: CSummonSkill::CSummonSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\summonskill.cpp:22
// RVA: 0x001E0EC0
// ADDRESS: 005e0ec0
// PROTOTYPE: undefined __thiscall CSummonSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSummonSkill::~CSummonSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\summonskill.cpp:30
// RVA: 0x001E0F20
// ADDRESS: 005e0f20
// PROTOTYPE: void __thiscall ~CSummonSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
