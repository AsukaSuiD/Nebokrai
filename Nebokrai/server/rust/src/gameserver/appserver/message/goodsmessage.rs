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
//! `0x8FC2E` читает два GUID, выполняет exact four-container lookup и только
//! для найденного goods вызывает обязательный virtual property runtime.
//! Парные `0x8FC2F/0x8FC30` публикуют ordered CiQing goods preview и global
//! setup; первый непустой preview сохраняет ранний return исходного EXE.
//! `0x8FC31` продолжает тот же owner полным make-проходом: оба feature gate-а
//! стоят до decode, ресурсы расходуются в audit/delete order, factory и packet
//! add сохраняют stacking/ownership effects, затем отправляется `0xBF932`.
//! `0x8FC32` замыкает compose slots `0/1/2`, exact fallback без оплаты,
//! recipe payment/RNG, source deletion, unlock/query и `0xBF81B` tail.
//! `0x8FC33` расходует `CQ0008`, удаляет одну единицу из основного CiQing
//! container и вызывает обязательный property runtime либо шлёт отказ.
//! `0x8FC34` читает amount до gate и ведёт hand goods через level/slot/improve
//! проверки, chance roll, failure destruction либо native clone/mount,
//! material consumption, property callback и `0xC0111`.
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
    CGame, CiQingComposeContext, CiQingComposeReport, CiQingDeleteReport, CiQingGoodsQueryReport,
    CiQingMakeContext, CiQingMakeReport, CiQingMountReport, CiQingSetupQueryReport,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::public::guid::CGuid;

const CHECK_BATTLE_FAIRY_COMBINE: u32 = 0x0008_fc26;
const COMBINE_BATTLE_FAIRY: u32 = 0x0008_fc27;
const UPGRADE_BATTLE_FAIRY: u32 = 0x0008_fc28;
const RESET_BATTLE_FAIRY_SKILLS: u32 = 0x0008_fc29;
const ALLOCATE_BATTLE_FAIRY_POTENTIAL: u32 = 0x0008_fc2a;
const RESET_BATTLE_FAIRY_POTENTIAL: u32 = 0x0008_fc2b;
const SUMMON_BATTLE_FAIRY: u32 = 0x0008_fc2c;
const RECALL_BATTLE_FAIRY: u32 = 0x0008_fc2d;
const REFRESH_BATTLE_FAIRY_PROPERTY: u32 = 0x0008_fc2e;
const QUERY_CI_QING_GOODS: u32 = 0x0008_fc2f;
const QUERY_CI_QING_SETUP: u32 = 0x0008_fc30;
const MAKE_CI_QING_NODE: u32 = 0x0008_fc31;
const COMPOSE_CI_QING_NODE: u32 = 0x0008_fc32;
const DELETE_CI_QING_GOODS: u32 = 0x0008_fc33;
const MOUNT_CI_QING_FROM_HAND: u32 = 0x0008_fc34;

pub(crate) trait GameGoodsMessageRuntime:
    BattleFairyCombineContext
    + BattleFairyUpgradeContext
    + BattleFairyDeathContext
    + BattleFairyPotentialResetContext
    + BattleFairyRuntimeContext
    + CiQingMakeContext
    + CiQingComposeContext
{
    fn run_battle_fairy_reset_script(
        &mut self,
        game: &mut CGame,
        player_id: i32,
        region_id: Option<i32>,
        script_path: &[u8],
    );

    fn update_battle_fairy_player_property(&mut self, game: &mut CGame, player_id: i32);
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPropertyRefreshOutcome {
    MissingGoods,
    UpdateDispatched,
}

#[must_use = "property refresh report сохраняет оба GUID и virtual update result"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyPropertyRefreshReport {
    pub(crate) ignored_guid: CGuid,
    pub(crate) goods_guid: CGuid,
    pub(crate) outcome: BattleFairyPropertyRefreshOutcome,
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
    BattleFairyPropertyRefresh(BattleFairyPropertyRefreshReport),
    CiQingGoods(CiQingGoodsQueryReport),
    CiQingSetup(CiQingSetupQueryReport),
    CiQingUnavailable,
    CiQingMake(CiQingMakeReport),
    CiQingCompose(CiQingComposeReport),
    CiQingPositionOutOfRange { position: u32 },
    CiQingDelete(CiQingDeleteReport),
    CiQingMount(CiQingMountReport),
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
            | REFRESH_BATTLE_FAIRY_PROPERTY
            | QUERY_CI_QING_GOODS
            | QUERY_CI_QING_SETUP
            | MAKE_CI_QING_NODE
            | COMPOSE_CI_QING_NODE
            | DELETE_CI_QING_GOODS
            | MOUNT_CI_QING_FROM_HAND
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
        REFRESH_BATTLE_FAIRY_PROPERTY => {
            let ignored_guid = match message.base_mut().get_guid() {
                Some(guid) => guid,
                None => return Some(Err(GameGoodsMessageError::MissingField("ignored GUID"))),
            };
            let goods_guid = match message.base_mut().get_guid() {
                Some(guid) => guid,
                None => return Some(Err(GameGoodsMessageError::MissingField("goods GUID"))),
            };
            let goods_exists = game
                .find_player(player_id)
                .is_some_and(|player| player.get_goods_by_id(goods_guid).is_some());
            let outcome = if goods_exists {
                runtime.update_battle_fairy_player_property(game, player_id);
                BattleFairyPropertyRefreshOutcome::UpdateDispatched
            } else {
                BattleFairyPropertyRefreshOutcome::MissingGoods
            };
            GameGoodsMessageOutcome::BattleFairyPropertyRefresh(BattleFairyPropertyRefreshReport {
                ignored_guid,
                goods_guid,
                outcome,
            })
        }
        QUERY_CI_QING_GOODS => GameGoodsMessageOutcome::CiQingGoods(
            game.query_ci_qing_goods(player_id, runtime)
                .expect("resolved message player остаётся в CGame во время synchronous dispatch"),
        ),
        QUERY_CI_QING_SETUP => {
            GameGoodsMessageOutcome::CiQingSetup(game.query_ci_qing_setup(player_id))
        }
        MAKE_CI_QING_NODE => {
            if !game.ci_qing_message_enabled(player_id) {
                GameGoodsMessageOutcome::CiQingUnavailable
            } else {
                let base_index = match read_long(message, "CiQing make base index") {
                    Ok(value) => value as u32,
                    Err(error) => return Some(Err(error)),
                };
                let amount = match read_long(message, "CiQing make amount") {
                    Ok(value) => value as u32,
                    Err(error) => return Some(Err(error)),
                };
                GameGoodsMessageOutcome::CiQingMake(
                    game.make_ci_qing_node(player_id, base_index, amount, runtime)
                        .expect(
                            "resolved message player остаётся в CGame во время synchronous dispatch",
                        ),
                )
            }
        }
        COMPOSE_CI_QING_NODE => {
            if !game.ci_qing_message_enabled(player_id) {
                GameGoodsMessageOutcome::CiQingUnavailable
            } else {
                GameGoodsMessageOutcome::CiQingCompose(
                    game.compose_ci_qing_node(player_id, runtime).expect(
                        "resolved message player остаётся в CGame во время synchronous dispatch",
                    ),
                )
            }
        }
        DELETE_CI_QING_GOODS => {
            if !game.ci_qing_message_enabled(player_id) {
                GameGoodsMessageOutcome::CiQingUnavailable
            } else {
                let position = match read_long(message, "CiQing delete position") {
                    Ok(value) => value as u32,
                    Err(error) => return Some(Err(error)),
                };
                if position >= 8 {
                    GameGoodsMessageOutcome::CiQingPositionOutOfRange { position }
                } else {
                    GameGoodsMessageOutcome::CiQingDelete(
                        game.delete_goods_from_ci_qing(player_id, position, runtime)
                            .expect(
                                "resolved message player остаётся в CGame во время synchronous dispatch",
                            ),
                    )
                }
            }
        }
        MOUNT_CI_QING_FROM_HAND => {
            let amount = match read_long(message, "CiQing mount amount") {
                Ok(value) => value as u32,
                Err(error) => return Some(Err(error)),
            };
            if !game.ci_qing_message_enabled(player_id) {
                GameGoodsMessageOutcome::CiQingUnavailable
            } else {
                GameGoodsMessageOutcome::CiQingMount(
                    game.mount_ci_qing_from_hand(player_id, amount, runtime)
                        .expect(
                            "resolved message player остаётся в CGame во время synchronous dispatch",
                        ),
                )
            }
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
