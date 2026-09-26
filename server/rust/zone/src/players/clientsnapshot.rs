//! Клиентские снимки игрока `AddToByteArray_ForClient` над частями Zone:
//! short-вариант `(false)` для area/query публикаций и точный полный
//! вариант `(true)`, который `OnLogMessage` вкладывает в успешный `0xBF401`.
//! GameSave здесь неприменим: client wire иначе упорядочивает навыки,
//! контейнеры, валюты, задания и завершающие country/CiQing поля.
//!
//! Hub `CPlayer` старого пакета бережёт делегаты прежних сигнатур и передаёт
//! заёмные проекции своих частей; shared-проекции поверх тех же hub-полей —
//! [`PlayerClientShapeParts`] и [`PlayerInitialClientParts`].
//!
//! Исходный owner: `server/gameserver/appserver/player.cpp/.h`, точная пара
//! GameServer/gameserver.exe + GameServer.pdb. Статусы перенесённых тел
//! сохранены рядом с кодом.

use std::collections::VecDeque;

use nebokrai_shared::protocol::LegacyWriter;
use nebokrai_shared::resources::CQuestSystem;
use nebokrai_shared::values::CGuid;

use crate::combat::PlayerCombatProperties;
use crate::content::goods::{GOODS_TYPE_EQUIPMENT, GAP_WEAPON_LEVEL};
use crate::content::goodsfactory::CGoodsFactory;
use crate::items::camountlimitgoodscontainer::CAmountLimitGoodsContainer;
use crate::items::cbattlefairycontainer::CBattleFairyContainer;
use crate::items::cequipmentcontainer::CEquipmentContainer;
use crate::items::cfairycontainer::CFairyContainer;
use crate::items::cgoods::CGoods;
use crate::items::cjifen::CJiFen;
use crate::items::cvolumelimitgoodscontainer::CVolumeLimitGoodsContainer;
use crate::items::cwallet::CWallet;
use crate::items::cyuanbao::CYuanBao;
use crate::quests::{append_client_quest_record, PlayerQuestProgress};
use crate::regions::moveshape::{is_died, MoveShapeState};
use crate::skills::skillfactory::CSkillFactory;

use super::gamesave::{
    encode_player_lei_ting, encode_player_organizing_snapshot, serializable_player_skills,
    synchronized_base_property_wire, PlayerBaseProperties, PlayerFriend, PlayerLeiTingThing,
    PlayerOrganizingSnapshot, PLAYER_BASE_PROPERTY_WIRE_SIZE, PLAYER_COMBAT_PROPERTY_WIRE_SIZE,
};

/// Shared-проекция hub `CPlayer` для short-варианта клиентского снимка формы
/// (area/query публикации).
pub struct PlayerClientShapeParts<'a> {
    pub move_shape: &'a MoveShapeState,
    pub base_properties: &'a PlayerBaseProperties,
    pub combat_properties: &'a PlayerCombatProperties,
    pub equipment: &'a CEquipmentContainer,
    pub organizing: PlayerOrganizingSnapshot<'a>,
    pub murderer_time_stamp_ms: u32,
    pub contend_state: bool,
    pub city_war_died_state: bool,
    pub emotion_index: i32,
    pub emotion_timestamp_ms: u32,
    pub country: u8,
    pub war_soul_state: u32,
}

/// Проекция hub `CPlayer` для точного полного варианта снимка входа
/// (`0xBF401`): shared-чтение всех частей; побочные hub-поля
/// `ci_qing_open`/`battle_fairy_summoned` — изменяемые, как в оригинале.
pub struct PlayerInitialClientParts<'a> {
    pub move_shape: &'a MoveShapeState,
    pub base_property_wire: &'a [u8; PLAYER_BASE_PROPERTY_WIRE_SIZE],
    pub base_properties: &'a PlayerBaseProperties,
    pub combat_property_wire: &'a [u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
    pub account: &'a [u8],
    pub title: &'a [u8],
    pub friends: &'a [PlayerFriend],
    pub lei_ting_things: &'a VecDeque<PlayerLeiTingThing>,
    pub organizing: PlayerOrganizingSnapshot<'a>,
    pub contend_state: bool,
    pub city_war_died_state: bool,
    pub quest_progress: &'a PlayerQuestProgress,
    pub country: u8,
    pub contribution: i32,
    pub war_soul_state: u32,
    pub hand: &'a CAmountLimitGoodsContainer,
    pub equipment: &'a CEquipmentContainer,
    pub auction_goods: &'a CVolumeLimitGoodsContainer,
    pub packet: &'a CVolumeLimitGoodsContainer,
    pub auction_listing: &'a CVolumeLimitGoodsContainer,
    pub fairy_container: &'a CFairyContainer,
    pub wallet: &'a CWallet,
    pub auction_wallet: &'a CWallet,
    pub yuan_bao: &'a CYuanBao,
    pub ji_fen: &'a CJiFen,
    pub battle_fairy_container: &'a CBattleFairyContainer,
    pub ci_qing_compose: &'a CVolumeLimitGoodsContainer,
    pub ci_qing: &'a CVolumeLimitGoodsContainer,
    pub ci_qing_open: &'a mut bool,
    pub battle_fairy_summoned: &'a mut bool,
}

/// Точный short-вариант `CPlayer::AddToByteArray_ForClient(false)` для
/// area/query публикаций. Полный login/save вариант остаётся у отдельного
/// GameSave codec; здесь нет container payload-ов и account-данных.
pub fn encode_client_shape_snapshot(
    parts: &PlayerClientShapeParts<'_>,
    goods_factory: &CGoodsFactory,
    country_identity: u8,
    personal_shop: Option<(i32, i32, &[u8])>,
    team_member_count: usize,
    mut now_milliseconds: impl FnMut() -> u32,
) -> Option<Vec<u8>> {
    const VISIBLE_EQUIPMENT: [u32; 11] = [0, 1, 2, 3, 4, 9, 10, 12, 13, 14, 15];

    let mut payload = crate::skills::state::encode_client_snapshot_with_team_count(
        &parts.move_shape.shape,
        &parts.move_shape.state_storage,
        false,
        is_died(parts.base_properties.health),
        team_member_count,
        &mut now_milliseconds,
    )?;
    let mut writer = LegacyWriter::new(&mut payload);
    writer.write_u8(parts.base_properties.head_picture as u8);
    writer.write_u8(u8::from(parts.base_properties.display_head_piece));
    for position in VISIBLE_EQUIPMENT {
        writer.write_u32(
            parts
                .equipment
                .get_goods(position)
                .map_or(0, CGoods::base_properties_index),
        );
    }
    for position in VISIBLE_EQUIPMENT {
        writer.write_u8(parts.equipment.get_goods(position).map_or(0, |goods| {
            goods.addon_property_value(goods_factory, GAP_WEAPON_LEVEL, 1) as u8
        }));
    }
    writer.write_u32(parts.base_properties.health);
    writer.write_u32(parts.combat_properties.maximum_hp);
    writer.write_u16(parts.base_properties.pk_count);
    writer.write_u8(u8::from(parts.murderer_time_stamp_ms != 0));
    writer.write_bytes(&encode_player_organizing_snapshot(parts.organizing)?);
    writer.write_u8(u8::from(parts.contend_state));
    writer.write_u8(u8::from(parts.city_war_died_state));
    writer.write_u8(parts.base_properties.occupation as u8);
    writer.write_u8(parts.base_properties.sex as u8);
    writer.write_u32(parts.base_properties.mode);
    if let Some((session_id, plug_id, shop_name)) = personal_shop {
        writer.write_i32(session_id);
        writer.write_i32(plug_id);
        writer.write_c_string(shop_name);
    } else {
        writer.write_i32(0);
        writer.write_i32(0);
    }
    writer.write_i32(parts.emotion_index);
    writer.write_u32(now_milliseconds().wrapping_sub(parts.emotion_timestamp_ms));
    writer.write_u8(parts.country);
    writer.write_u8(parts.base_properties.face_picture as u8);
    writer.write_u8(parts.base_properties.level);
    writer.write_u32(parts.base_properties.credit);
    writer.write_u8(country_identity);
    writer.write_u32(parts.base_properties.appellation_id);
    writer.write_u32(parts.war_soul_state);
    writer.write_i32(parts.base_properties.gods_battle_faction);
    Some(payload)
}

/// Точный полный вариант `CPlayer::AddToByteArray_ForClient(true)`,
/// который `OnLogMessage` вкладывает в успешный `0xBF401`. Persisted
/// GameSave здесь неприменим: client wire иначе упорядочивает навыки,
/// контейнеры, валюты, задания и завершающие country/CiQing поля.
#[allow(
    clippy::too_many_arguments,
    reason = "literal login snapshot сохраняет достигнутые runtime-входы hub-а"
)]
pub fn encode_initial_client_snapshot(
    parts: &mut PlayerInitialClientParts<'_>,
    goods_factory: &CGoodsFactory,
    skill_factory: &CSkillFactory,
    quest_system: &CQuestSystem,
    da_kong_enabled: bool,
    level_experience: u32,
    country_identity: u8,
    team_member_count: usize,
    loan_time_limit: u32,
    ci_qing_quest_id: u32,
    timed_state_now_milliseconds: impl FnMut() -> u32,
) -> Option<Vec<u8>> {
    const SKILL_USAGE_MP_COST: u32 = 2;
    const SKILL_USAGE_MIN_DISTANCE: u32 = 5_002;
    const SKILL_USAGE_MAX_DISTANCE: u32 = 5_003;
    const SKILL_USAGE_DELAY_TIME: u32 = 10_001;

    *parts.battle_fairy_summoned = parts.war_soul_state != 0;
    let mut payload = crate::skills::state::encode_client_snapshot_with_team_count(
        &parts.move_shape.shape,
        &parts.move_shape.state_storage,
        true,
        is_died(parts.base_properties.health),
        team_member_count,
        timed_state_now_milliseconds,
    )?;
    payload.extend_from_slice(&synchronized_base_property_wire(
        parts.base_property_wire,
        parts.base_properties,
        *parts.battle_fairy_summoned,
    ));
    {
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_c_string(parts.account);
        writer.write_c_string(parts.title);
        writer.write_bytes(parts.combat_property_wire);
        writer.write_u32(level_experience);

        let skills: Vec<_> = serializable_player_skills(parts.move_shape).collect();
        writer.write_i32(i32::try_from(skills.len()).ok()?);
        for skill in skills {
            let properties = skill_factory.query_skill_base_properties(skill.id(), skill.level())?;
            writer.write_u32(
                (skill.id() & 0xffff) | ((skill.level() as u32 & 0xffff) << 16),
            );
            writer.write_u32(properties.query_property(SKILL_USAGE_DELAY_TIME));
            let maximum = properties.query_property(SKILL_USAGE_MAX_DISTANCE);
            writer.write_u16(if maximum == 0 { 1 } else { maximum } as u16);
            writer.write_u16(properties.query_property(SKILL_USAGE_MP_COST) as u16);
            writer.write_u16(properties.query_property(SKILL_USAGE_MIN_DISTANCE) as u16);
        }

        writer.write_i32(i32::try_from(parts.friends.len()).ok()?);
        for friend in parts.friends {
            writer.write_c_string(&friend.name);
            writer.write_u8(u8::from(friend.online));
        }
    }
    payload.extend_from_slice(&encode_player_lei_ting(parts.base_properties, parts.lei_ting_things));

    if let Some(goods) = parts.hand.get_goods(0)
        && let Some(base) = goods_factory.query_goods_base_properties(goods.base_properties_index())
    {
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_u8(1);
        writer.write_u8(u8::from(base.goods_type() == GOODS_TYPE_EQUIPMENT));
        writer.write_u16(goods.amount() as u16);
        writer.write_u8(0);
        goods
            .serialize_for_old_client(&mut payload, goods_factory, da_kong_enabled)
            .then_some(())?;
    } else {
        payload.push(0);
    }

    let equipment = parts.equipment.traversing_goods();
    LegacyWriter::new(&mut payload).write_i32(i32::try_from(equipment.len()).ok()?);
    for (column, goods) in equipment {
        goods
            .serialize_for_old_client(&mut payload, goods_factory, da_kong_enabled)
            .then_some(())?;
        LegacyWriter::new(&mut payload).write_u32(column.position());
    }
    append_old_client_volume(&mut payload, parts.auction_goods, goods_factory, da_kong_enabled)?;
    append_old_client_volume(&mut payload, parts.packet, goods_factory, da_kong_enabled)?;
    append_old_client_volume(&mut payload, parts.auction_listing, goods_factory, da_kong_enabled)?;
    append_old_client_volume(
        &mut payload,
        parts.fairy_container.base(),
        goods_factory,
        da_kong_enabled,
    )?;

    for (amount, goods) in [
        (parts.wallet.currency_amount(), parts.wallet.goods()),
        (parts.auction_wallet.currency_amount(), parts.auction_wallet.goods()),
        (parts.yuan_bao.currency_amount(), parts.yuan_bao.goods()),
        (parts.ji_fen.currency_amount(), parts.ji_fen.goods()),
    ] {
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_u32(amount);
        let guid = goods.map_or(CGuid::GUID_INVALID, |goods| goods.identity().ex_id);
        if guid.is_invalid() {
            writer.write_u8(0);
        } else {
            writer.write_u8(16);
            writer.write_bytes(guid.as_legacy_bytes());
        }
    }

    payload.extend_from_slice(&encode_player_organizing_snapshot(parts.organizing)?);
    {
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_u8(u8::from(parts.contend_state));
        // Оригинал: VA 0x0044A980–0x0044A988; клиент читает byte
        // в 0x0045197C–0x00451989. Ширина не следует из имени m_l*.
        writer.write_u8(u8::from(parts.city_war_died_state));
    }
    append_client_quest_snapshot(&mut payload, parts.quest_progress, quest_system)?;
    {
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_u8(parts.country);
        writer.write_i32(parts.contribution);
        writer.write_u8(country_identity);
        writer.write_u32(loan_time_limit);
    }
    append_old_client_volume(
        &mut payload,
        parts.battle_fairy_container.base(),
        goods_factory,
        da_kong_enabled,
    )?;
    append_old_client_volume(&mut payload, parts.ci_qing_compose, goods_factory, da_kong_enabled)?;
    append_old_client_volume(&mut payload, parts.ci_qing, goods_factory, da_kong_enabled)?;
    {
        let quest_state = parts.quest_progress.raw_state(ci_qing_quest_id as u16);
        if quest_state == Some(1) {
            *parts.ci_qing_open = true;
        }
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_u32(parts.war_soul_state);
        writer.write_u32(quest_state.map_or(2, u32::from));
    }
    Some(payload)
}

fn append_client_quest_snapshot(
    destination: &mut Vec<u8>,
    quest_progress: &PlayerQuestProgress,
    quest_system: &CQuestSystem,
) -> Option<()> {
    let active: Vec<_> = quest_progress
        .client_entries(|quest_id| quest_system.quest_data_by_id(quest_id))
        .collect();
    let mut writer = LegacyWriter::new(destination);
    writer.write_i32(quest_system.max_quest_count);
    writer.write_i32(i32::try_from(active.len()).ok()?);
    for (quest_id, quest) in active {
        append_client_quest_record(destination, quest_id, quest);
    }
    Some(())
}

fn append_old_client_volume(
    destination: &mut Vec<u8>,
    container: &CVolumeLimitGoodsContainer,
    goods_factory: &CGoodsFactory,
    da_kong_enabled: bool,
) -> Option<()> {
    // CPacketListener::OnTraversingContainer, VA 0x0042D35A–0x0042D3BF:
    // префикс содержит признак экипировки, количество и младший байт
    // позиции QueryGoodsPosition. Все семь контейнеров используют его;
    // клиент аукциона пропускает первые три байта, но читает четвёртый.
    let goods: Vec<_> = container.base().traversing_goods().collect();
    LegacyWriter::new(destination).write_i32(i32::try_from(goods.len()).ok()?);
    for goods in goods {
        let position = container.query_goods_position(goods.identity().ex_id)?;
        let base = goods_factory.query_goods_base_properties(goods.base_properties_index())?;
        let mut writer = LegacyWriter::new(destination);
        writer.write_u8(u8::from(base.goods_type() == GOODS_TYPE_EQUIPMENT));
        writer.write_u16(goods.amount() as u16);
        writer.write_u8(position as u8);
        goods
            .serialize_for_old_client(destination, goods_factory, da_kong_enabled)
            .then_some(())?;
    }
    Some(())
}
