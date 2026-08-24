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
    BattleFairyPotentialResetReport, BattleFairyUpgradeReport,
};
use crate::gameserver::gameserver::game::{
    BattleFairyCombineContext, BattleFairyDeathContext, BattleFairyPotentialResetContext,
    BattleFairyUpgradeContext, CGame,
};
use crate::nets::netserver::message::CMessage;

const CHECK_BATTLE_FAIRY_COMBINE: u32 = 0x0008_fc26;
const COMBINE_BATTLE_FAIRY: u32 = 0x0008_fc27;
const UPGRADE_BATTLE_FAIRY: u32 = 0x0008_fc28;
const ALLOCATE_BATTLE_FAIRY_POTENTIAL: u32 = 0x0008_fc2a;
const RESET_BATTLE_FAIRY_POTENTIAL: u32 = 0x0008_fc2b;

pub(crate) trait GameGoodsMessageRuntime:
    BattleFairyCombineContext
    + BattleFairyUpgradeContext
    + BattleFairyDeathContext
    + BattleFairyPotentialResetContext
{
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameGoodsMessageError {
    MissingField(&'static str),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameGoodsMessageOutcome {
    MissingPlayer,
    BattleFairyCombineCheck(BattleFairyCombineCheck),
    BattleFairyCombine(BattleFairyCombineReport),
    BattleFairyUpgrade(BattleFairyUpgradeReport),
    BattleFairyPotentialAllocation(BattleFairyPotentialAllocationReport),
    BattleFairyPotentialReset(BattleFairyPotentialResetReport),
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
            | ALLOCATE_BATTLE_FAIRY_POTENTIAL
            | RESET_BATTLE_FAIRY_POTENTIAL
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
