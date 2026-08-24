//! Входные goods-сообщения GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/message/goodsmessage.cpp`. Материализован
//! полный combine-проход боевой феи: `0x8FC26` проверяет состав и публикует
//! notification либо `0xBF92C`, а `0x8FC27` исполняет player/container/game
//! mutations, old-client codec, сетевые результаты и аудит. `0x8FC28`
//! продолжает тот же транспортный owner полным upgrade-проходом через RNG,
//! player wallet, gems, equipment update и audit gates. Decoder `0x8FC2A`
//! сохраняет count/reserved/pairs wire-формат и доводит распределение potential
//! до player properties и повторных old-client goods updates. `0x8FC2B`
//! замыкает расход reset-item, сброс potential/player state и клиентский update.
//! Парные `0x8FC2C/0x8FC2D` ведут summon/recall через один WarSoul lifecycle,
//! region spatial map, around packets и пересчёт player properties. `0x8FC29`
//! сохраняет два `long`, detach → live script → attach и World ack; сам script
//! остаётся явной runtime-границей, а не подменяется упрощённым reset helper-ом.
//!
//! Остальные opcodes owner-а остаются RAW ниже и продолжают проходить через
//! прежнюю общую handler-границу.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\goodsmessage.cpp

use crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyCombineCheck;
use crate::gameserver::appserver::player::{
    BattleFairyCombineReport, BattleFairyPotentialAllocationReport,
    BattleFairyPotentialResetReport, BattleFairySummonReport, BattleFairyUpgradeReport,
};
use crate::gameserver::gameserver::game::{
    BattleFairyCombineContext, BattleFairyDeathContext, BattleFairyPotentialResetContext,
    BattleFairyRuntimeContext, BattleFairyScriptSkillAttachReport, BattleFairyUpgradeContext,
    CGame,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};

const CHECK_BATTLE_FAIRY_COMBINE: u32 = 0x0008_fc26;
const COMBINE_BATTLE_FAIRY: u32 = 0x0008_fc27;
const UPGRADE_BATTLE_FAIRY: u32 = 0x0008_fc28;
const RESET_BATTLE_FAIRY_SKILLS: u32 = 0x0008_fc29;
const ALLOCATE_BATTLE_FAIRY_POTENTIAL: u32 = 0x0008_fc2a;
const RESET_BATTLE_FAIRY_POTENTIAL: u32 = 0x0008_fc2b;
const SUMMON_BATTLE_FAIRY: u32 = 0x0008_fc2c;
const RECALL_BATTLE_FAIRY: u32 = 0x0008_fc2d;

pub(crate) trait GameGoodsMessageRuntime:
    BattleFairyCombineContext
    + BattleFairyUpgradeContext
    + BattleFairyDeathContext
    + BattleFairyPotentialResetContext
    + BattleFairyRuntimeContext
{
    fn run_battle_fairy_reset_script(
        &mut self,
        game: &mut CGame,
        player_id: i32,
        region_id: Option<i32>,
        script_path: &[u8],
    );
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameGoodsMessageError {
    MissingField(&'static str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyScriptResetOutcome {
    MissingBattleFairyEquipment,
    Dispatched,
}

#[must_use = "script reset report сохраняет detach, script, attach и World ack"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyScriptResetReport {
    pub(crate) ignored_value: i32,
    pub(crate) script_index: i32,
    pub(crate) script_path: Vec<u8>,
    pub(crate) outcome: BattleFairyScriptResetOutcome,
    pub(crate) detached_skill_ids: Vec<u32>,
    pub(crate) attached: Option<BattleFairyScriptSkillAttachReport>,
    pub(crate) world_ack: Option<Result<i32, SendMessageError>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameGoodsMessageOutcome {
    MissingPlayer,
    BattleFairyCombineCheck(BattleFairyCombineCheck),
    BattleFairyCombine(BattleFairyCombineReport),
    BattleFairyUpgrade(BattleFairyUpgradeReport),
    BattleFairyScriptReset(BattleFairyScriptResetReport),
    BattleFairyPotentialAllocation(BattleFairyPotentialAllocationReport),
    BattleFairyPotentialReset(BattleFairyPotentialResetReport),
    BattleFairySummon(BattleFairySummonReport),
}

#[must_use = "goods-message report содержит routing и полный gameplay result"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameGoodsMessageReport {
    pub(crate) message_type: u32,
    pub(crate) socket_id: i32,
    pub(crate) player_id: Option<i32>,
    pub(crate) region_id: Option<i32>,
    pub(crate) outcome: GameGoodsMessageOutcome,
}

pub(crate) fn dispatch_game_goods_message<Runtime: GameGoodsMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<GameGoodsMessageReport, GameGoodsMessageError>> {
    let message_type = message.message_type() as u32;
    if !matches!(
        message_type,
        CHECK_BATTLE_FAIRY_COMBINE
            | COMBINE_BATTLE_FAIRY
            | UPGRADE_BATTLE_FAIRY
            | RESET_BATTLE_FAIRY_SKILLS
            | ALLOCATE_BATTLE_FAIRY_POTENTIAL
            | RESET_BATTLE_FAIRY_POTENTIAL
            | SUMMON_BATTLE_FAIRY
            | RECALL_BATTLE_FAIRY
    ) {
        return None;
    }

    message.resolve_player_context(game);
    let socket_id = message.socket_id();
    let player_id = message.player_id();
    let region_id = message.region_id();
    let Some(player_id) = player_id else {
        return Some(Ok(GameGoodsMessageReport {
            message_type,
            socket_id,
            player_id: None,
            region_id,
            outcome: GameGoodsMessageOutcome::MissingPlayer,
        }));
    };
    let read_long = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long()
            .ok_or(GameGoodsMessageError::MissingField(field))
    };
    let outcome = match message_type {
        CHECK_BATTLE_FAIRY_COMBINE => GameGoodsMessageOutcome::BattleFairyCombineCheck(
            game.check_battle_fairy_combine(player_id),
        ),
        COMBINE_BATTLE_FAIRY => GameGoodsMessageOutcome::BattleFairyCombine(
            game.combine_battle_fairy(player_id, runtime)
                .expect("resolved message player остаётся в CGame во время synchronous dispatch"),
        ),
        UPGRADE_BATTLE_FAIRY => GameGoodsMessageOutcome::BattleFairyUpgrade(
            game.upgrade_battle_fairy_equipment(player_id, runtime)
                .expect("resolved message player остаётся в CGame во время synchronous dispatch"),
        ),
        RESET_BATTLE_FAIRY_SKILLS => {
            let ignored_value = match read_long(message, "skill reset ignored value") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let script_index = match read_long(message, "skill reset script index") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let script_path =
                format!("scripts/skills/restskills_0{script_index}.script").into_bytes();
            let Some(detached_skill_ids) = game.detach_battle_fairy_script_skills(player_id) else {
                return Some(Ok(GameGoodsMessageReport {
                    message_type,
                    socket_id,
                    player_id: Some(player_id),
                    region_id,
                    outcome: GameGoodsMessageOutcome::BattleFairyScriptReset(
                        BattleFairyScriptResetReport {
                            ignored_value,
                            script_index,
                            script_path,
                            outcome: BattleFairyScriptResetOutcome::MissingBattleFairyEquipment,
                            detached_skill_ids: Vec::new(),
                            attached: None,
                            world_ack: None,
                        },
                    ),
                }));
            };
            runtime.run_battle_fairy_reset_script(game, player_id, region_id, &script_path);
            let attached = game.attach_battle_fairy_script_skills(player_id);
            let world_ack = CMessage::new(0x0b_f931).send(game, player_id != 0);
            GameGoodsMessageOutcome::BattleFairyScriptReset(BattleFairyScriptResetReport {
                ignored_value,
                script_index,
                script_path,
                outcome: BattleFairyScriptResetOutcome::Dispatched,
                detached_skill_ids,
                attached: Some(attached),
                world_ack: Some(world_ack),
            })
        }
        ALLOCATE_BATTLE_FAIRY_POTENTIAL => {
            let count = match read_long(message, "allocation count") {
                Ok(count) => count,
                Err(error) => return Some(Err(error)),
            };
            if let Err(error) = read_long(message, "allocation reserved value") {
                return Some(Err(error));
            }
            let mut allocations = Vec::new();
            for _ in 0..count.max(0) {
                let property = match read_long(message, "allocation property") {
                    Ok(property) => property,
                    Err(error) => return Some(Err(error)),
                };
                let points = match read_long(message, "allocation points") {
                    Ok(points) => points,
                    Err(error) => return Some(Err(error)),
                };
                allocations.push((property, points));
            }
            GameGoodsMessageOutcome::BattleFairyPotentialAllocation(
                game.allocate_battle_fairy_potential(player_id, &allocations, runtime)
                    .expect(
                        "resolved message player остаётся в CGame во время synchronous dispatch",
                    ),
            )
        }
        RESET_BATTLE_FAIRY_POTENTIAL => GameGoodsMessageOutcome::BattleFairyPotentialReset(
            game.reset_battle_fairy_potential(player_id, runtime)
                .expect("resolved message player остаётся в CGame во время synchronous dispatch"),
        ),
        SUMMON_BATTLE_FAIRY | RECALL_BATTLE_FAIRY => {
            let mode = if message_type == SUMMON_BATTLE_FAIRY {
                1
            } else {
                -1
            };
            GameGoodsMessageOutcome::BattleFairySummon(
                game.summon_battle_fairy(player_id, mode, runtime).expect(
                    "resolved message player остаётся в CGame во время synchronous dispatch",
                ),
            )
        }
        _ => unreachable!("opcode отфильтрован перед dispatch"),
    };
    Some(Ok(GameGoodsMessageReport {
        message_type,
        socket_id,
        player_id: Some(player_id),
        region_id,
        outcome,
    }))
}

// ============================================================================
// FUNCTION: OnGoodsMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\goodsmessage.cpp:35
// RVA: 0x00093BF0
// ADDRESS: 00493bf0
// PROTOTYPE: void __cdecl OnGoodsMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
