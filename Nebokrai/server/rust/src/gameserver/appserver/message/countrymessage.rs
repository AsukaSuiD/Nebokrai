//! Диспетчер сообщений войны и управления странами.
//!
//! Источник: `gameserver.exe`, `GameServer.pdb` и исходный владелец
//! `message/countrymessage.cpp`. Реализованные коды сохраняют исходное чтение
//! полей, порядок мутаций стран и игроков, смены региона, вызовов генератора
//! случайных чисел и синхронных client/World-отправок. Узкий
//! `CountryWarDispatchEffects` передаёт только ещё не применённые решения:
//! замену кода сообщения и выбор адресатов после запуска фазы войны.
//! Результаты уже выполненных мутаций и отправок публикуются через `tracing` в
//! месте возникновения и не образуют дерево отчётов.
//!
//! Сохранена неоднозначность кода `0x7FF15`: восьмибайтовый устаревший ответ
//! распознаётся отдельно, а неопределённое чтение за концом исходного буфера не
//! воспроизводится. Все country-селекторы материализованы; `0x7FF20/21`
//! намеренно принадлежат соседнему reached owner-у `CGoodsWarMember`.

use super::super::country::countrywarsys::{
    CountryWarPhaseContext, CountryWarRegionContext, CountryWarSys, CountryWarVictoryContext,
};
use super::super::region::RegionRandomContext;
use super::super::servercountryregion::{CountryBattleStateBlock, CountryRegionRuntimeContext};
use crate::gameserver::gameserver::game::{
    CGame, RealmAppellationScriptContext, ScriptRegionChangeContext, ServerRegionOwner,
};
use crate::gameserver::appserver::legacycodec::LegacyReader;
use crate::nets::netserver::message::CMessage;
use std::mem::size_of;

pub(crate) trait GameCountryWarRuntime:
    CountryRegionRuntimeContext
    + RegionRandomContext
    + ScriptRegionChangeContext
    + RealmAppellationScriptContext
{
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarMessageDispatchError<SideError> {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    Side(SideError),
}

pub(crate) trait CountryWarMessageContext {
    type Region: Copy;
    type SideError;

    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region>;
    fn set_country_sides(&mut self, region: Self::Region, defend_country: i32, attack_country: i32);
    fn update_contend_player(&mut self, region: Self::Region);

    fn on_declare_begin(&mut self, region: Self::Region, region_id: i32);
    fn on_declare_end(&mut self, region: Self::Region, region_id: i32);
    fn on_prepare_begin(&mut self, region: Self::Region, region_id: i32);
    fn on_prepare_end(&mut self, region: Self::Region, region_id: i32);
    fn on_war_start(&mut self, region: Self::Region, region_id: i32);
    fn on_war_timeout(&mut self, region: Self::Region, region_id: i32);
    fn on_war_end(&mut self, region: Self::Region, region_id: i32);
    fn clear_country_region(&mut self, region: Self::Region);
    fn country_region_side_bytes(
        &mut self,
        region: Self::Region,
    ) -> Result<(u8, u8), Self::SideError>;
    fn reset_country_war_result(&mut self, country: u8);
    fn on_country_flag_destroy(&mut self, region: Self::Region, region_id: i32, country: i32);
    fn set_country_war_result(&mut self, country: u8, result: i32);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarBroadcastIntent {
    All,
    Countries([u8; 2]),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryWarDispatchEffects {
    pub(crate) message_type_effect: Option<u32>,
    pub(crate) broadcast: Option<CountryWarBroadcastIntent>,
}


pub(crate) fn dispatch_country_war_message<Context: CountryWarMessageContext>(
    opcode: u32,
    payload: &[u8],
    cursor: &mut usize,
    country_war_sys: &mut CountryWarSys,
    context: &mut Context,
) -> Option<
    Result<CountryWarDispatchEffects, CountryWarMessageDispatchError<Context::SideError>>,
> {
    let mut message_type_effect = None;
    let mut broadcast = None;
    match opcode {
        0x7ff17 => country_war_sys.on_declare_begin(&mut CountryPhaseAdapter(context)),
        0x7ff18 => country_war_sys.on_declare_end(&mut CountryPhaseAdapter(context)),
        0x7ff19 => country_war_sys.on_prepare_begin(&mut CountryPhaseAdapter(context)),
        0x7ff1a => country_war_sys.on_prepare_end(&mut CountryPhaseAdapter(context)),
        0x7ff1b => {
            country_war_sys.on_war_start(&mut CountryPhaseAdapter(context));
            message_type_effect = Some(0xc0312);
            let mut countries = [0_u8; 4];
            let mut country_count = 0;
            for country in 1..5 {
                if country_war_sys.is_already_declar(i32::from(country)) {
                    countries[country_count] = country;
                    country_count += 1;
                }
            }
            match country_count {
                4 => broadcast = Some(CountryWarBroadcastIntent::All),
                2 => broadcast = Some(CountryWarBroadcastIntent::Countries([countries[0], countries[1]])),
                _ => {}
            }
        }
        0x7ff1c => {
            if let Err(error) = country_war_sys.on_war_timeout(&mut CountryPhaseAdapter(context)) {
                return Some(Err(CountryWarMessageDispatchError::Side(error)));
            }
        }
        0x7ff1d => country_war_sys.on_war_end(&mut CountryPhaseAdapter(context)),
        0x7ff1e => country_war_sys.on_war_clear(&mut CountryPhaseAdapter(context)),
        0x7ff1f => {
            let region_id = match read_country_war_long(payload, cursor) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let defend_country = match read_country_war_long(payload, cursor) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let attack_country = match read_country_war_long(payload, cursor) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let update_found = country_war_sys.update_apply_war(
                region_id,
                defend_country,
                attack_country,
                &mut CountryRegionAdapter(context),
            );
            tracing::trace!(region_id, defend_country, attack_country, update_found, "заявка войны стран обновлена");
        }
        0x7ff22 => {
            let country = match read_country_war_byte(payload, cursor) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            country_war_sys
                .on_flag_destroy(i32::from(country), &mut CountryVictoryAdapter(context));
        }
        _ => return None,
    }
    Some(Ok(CountryWarDispatchEffects {
        message_type_effect,
        broadcast,
    }))
}

/// Подключает достигнутую country-war family к owned `CGame` state и тому же
/// входному `CMessage`, который исходный dispatcher переиспользует для send.
pub(crate) fn dispatch_game_country_war_message<Runtime: GameCountryWarRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<
    Result<(), CountryWarMessageDispatchError<CountryBattleStateBlock>>,
> {
    let opcode = message.message_type() as u32;
    if opcode == 0x7ff16 {
        dispatch_country_war_declaration_response(message, game, opcode);
        return Some(Ok(()));
    }
    if opcode == 0x7ff14 {
        dispatch_country_quest_reset_message(game, opcode);
        return Some(Ok(()));
    }
    if matches!(opcode, 0x7ff11..=0x7ff13) {
        dispatch_country_notice_message(message, game, opcode);
        return Some(Ok(()));
    }
    if matches!(opcode, 0x7ff0e | 0x7ff15) {
        dispatch_country_exile_message(message, game, runtime, opcode);
        return Some(Ok(()));
    }
    if matches!(opcode, 0x7ff0c | 0x7ff0d | 0x7ff10) {
        dispatch_country_governance_effect_message(message, game, runtime, opcode);
        return Some(Ok(()));
    }
    if matches!(opcode, 0x7ff05 | 0x7ff07 | 0x7ff08) {
        dispatch_country_direct_response_message(message, game, opcode);
        return Some(Ok(()));
    }
    if opcode == 0x7ff04 {
        dispatch_country_information_change_message(message, game);
        return Some(Ok(()));
    }
    if opcode == 0x7ff01 {
        dispatch_player_country_change_message(message, game);
        return Some(Ok(()));
    }
    if opcode == 0x9050b {
        dispatch_country_war_entry_message(message, game, runtime);
        return Some(Ok(()));
    }
    if matches!(opcode, 0x90502..=0x9050a) {
        dispatch_country_governance_message(message, game, opcode);
        return Some(Ok(()));
    }
    if !matches!(opcode, 0x7ff17..=0x7ff1f | 0x7ff22) {
        return None;
    }

    let mut owners = game.take_war_startup_owners();
    let dispatched = {
        let mut context = GameCountryWarContext { game, runtime };
        let (payload, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        dispatch_country_war_message(opcode, payload, cursor, &mut owners.country, &mut context)
            .expect("country-war opcode проверен перед dispatcher-ом")
    };
    game.restore_war_startup_owners(owners);

    let dispatched = match dispatched {
        Ok(dispatched) => dispatched,
        Err(error) => return Some(Err(error)),
    };
    if let Some(message_type) = dispatched.message_type_effect {
        message.set_message_type(message_type as i32);
    }
    if let Some(intent) = dispatched.broadcast {
        if let Some(net_server) = game.current_net_server() {
            match intent {
                CountryWarBroadcastIntent::All => {
                    let delivery = message.send_all(Some(net_server));
                    tracing::trace!(?delivery, "сообщение войны стран отправлено всем игрокам");
                }
                CountryWarBroadcastIntent::Countries(countries) => {
                    for player_id in game.player_ids_in_countries(countries) {
                        let delivery = message.send_to_player(net_server, player_id);
                        tracing::trace!(?countries, player_id, delivery, "сообщение войны стран отправлено игроку");
                    }
                }
            }
        } else {
            tracing::warn!(?intent, "сообщение войны стран не отправлено: сетевой владелец отсутствует");
        }
    }
    tracing::trace!(?dispatched, "сообщение войны стран обработано");
    Some(Ok(()))
}

fn dispatch_country_war_declaration_response(
    message: &mut CMessage,
    game: &CGame,
    opcode: u32,
) {
    let decoded_accepted = message.base_mut().get_char();
    let accepted = decoded_accepted.unwrap_or(0);
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let decoded_target_country = message.base_mut().get_long();
    let target_country = decoded_target_country.unwrap_or(0);
    tracing::trace!(
        opcode,
        accepted,
        accepted_complete = decoded_accepted.is_some(),
        player_id,
        player_id_complete = decoded_player_id.is_some(),
        target_country,
        target_country_complete = decoded_target_country.is_some(),
        player_found = game.find_player(player_id).is_some(),
        "ответ на объявление войны стран разобран"
    );
}

fn dispatch_country_quest_reset_message(game: &mut CGame, opcode: u32) {
    for country in 1..5 {
        if game.country_handler().country(country).is_none() {
            continue;
        }
        for job in 1..8 {
            let message = game
                .country_handler()
                .country(country)
                .expect("country owner проверен перед quest reset")
                .quest_switch_message(job, false);
            let delivery = message.send(game, false);
            game
                .country_handler_mut()
                .country_mut(country)
                .expect("country owner жив после synchronous World enqueue")
                .apply_quest_switch(job, false);
            tracing::trace!(opcode, country, job, ?delivery, "переключатель задания страны сброшен");
        }
    }
}
fn dispatch_country_notice_message(
    message: &mut CMessage,
    game: &CGame,
    opcode: u32,
) {
    match opcode {
        0x7ff11 => {
            let country = message.base_mut().get_char().unwrap_or(0) as u8;
            let text = message.base_mut().get_str_bytes(0x100).unwrap_or_default();
            let mut response = CMessage::new(0x000b_f806);
            response.add_ulong(0xffff_00aa);
            response.add_ulong(0xaaff_ffff);
            response.base_mut().add(&text);
            response.add_byte(0);
            for player_id in game.player_ids_in_country(u32::from(country)) {
                let delivery = response.send_to_player(game.net_server(), player_id);
                tracing::trace!(opcode, country, player_id, delivery, text_bytes = text.len(), "уведомление страны отправлено игроку");
            }
        }
        0x7ff12 => {
            message.set_message_type(0x000b_f806);
            let delivery = message.send_all(game.current_net_server());
            tracing::trace!(opcode, response_type = 0x000b_f806, ?delivery, "уведомление страны отправлено всем игрокам");
        }
        0x7ff13 => {
            let decoded_player_id = message.base_mut().get_long();
            let player_id = decoded_player_id.unwrap_or(0);
            let text = message.base_mut().get_str_bytes(0x100).unwrap_or_default();
            let extra_legacy_long = message.base_mut().get_long();
            let mut response = CMessage::new(0x000c_030d);
            response.base_mut().add(&text);
            response.add_byte(0);
            let delivery = response.send_to_player(game.net_server(), player_id);
            tracing::trace!(
                opcode,
                player_id,
                player_id_complete = decoded_player_id.is_some(),
                text_bytes = text.len(),
                ?extra_legacy_long,
                delivery,
                "личное уведомление страны отправлено"
            );
        }
        _ => unreachable!("country notice opcode проверен перед dispatcher-ом"),
    }
}

fn dispatch_country_exile_message<Runtime: GameCountryWarRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
    opcode: u32,
) {
    let payload_bytes = message
        .as_wire_bytes()
        .len()
        .saturating_sub(message.base_mut().cursor());
    if opcode == 0x7ff15 && payload_bytes == 2 * size_of::<i32>() {
        let decoded_remaining_seconds = message.base_mut().get_long();
        let remaining_seconds = decoded_remaining_seconds.unwrap_or(0);
        let decoded_player_id = message.base_mut().get_long();
        let player_id = decoded_player_id.unwrap_or(0);
        tracing::debug!(
            opcode,
            remaining_seconds,
            remaining_seconds_complete = decoded_remaining_seconds.is_some(),
            player_id,
            player_id_complete = decoded_player_id.is_some(),
            "получено устаревшее сообщение времени изгнания"
        );
        return;
    }
    let decoded_country = message.base_mut().get_char();
    let country = decoded_country.unwrap_or(0) as u8;
    if opcode == 0x7ff15 {
        let advertised_count = message.base_mut().get_long().unwrap_or(0);
        let Some(_) = game.country_handler().country(country) else {
            tracing::warn!(opcode, country, advertised_count, "список изгнанных не синхронизирован: страна отсутствует");
            return;
        };
        for _ in 0..advertised_count.max(0) {
            let player_id = message.base_mut().get_long().unwrap_or(0);
            let sampled_at_ms = runtime.now_milliseconds();
            game
                .country_handler_mut()
                .country_mut(country)
                .expect("country owner проверен перед синхронизацией exile-list")
                .add_to_exile_list(player_id, sampled_at_ms);
            tracing::trace!(opcode, country, player_id, sampled_at_ms, "запись списка изгнанных синхронизирована");
        }
        tracing::trace!(opcode, country, country_complete = decoded_country.is_some(), advertised_count, "список изгнанных синхронизирован");
        return;
    }

    if game.country_handler().country(country).is_none() {
        tracing::warn!(opcode, country, "изгнание не обработано: страна отсутствует");
        return;
    }
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let Some(player) = game.find_player(player_id) else {
        tracing::warn!(opcode, country, player_id, "изгнание не обработано: игрок отсутствует");
        return;
    };
    let current_region_id = player.server_region_id();
    let direction = player.shape().get_direction();
    let can_exile = !player.in_changing_region() && !player.in_changing_server();
    let guard_country = current_region_id
        .and_then(|region_id| game.find_region(region_id))
        .map(|region| region.base().country);
    let can_exile = can_exile && guard_country == Some(country);

    let mut extra_legacy_long = None;
    let mut mutation_applied = false;
    if can_exile {
        extra_legacy_long = message.base_mut().get_long();
        let Some(destination) = game.country_param().exile_point(country) else {
            tracing::error!(opcode, country, player_id, "изгнание не обработано: точка назначения отсутствует");
            return;
        };
        if current_region_id != Some(destination.region_id) {
            let relocation = game.change_player_region(
                player_id,
                destination.region_id,
                destination.x,
                destination.y,
                direction,
                0,
                0,
                0,
                runtime,
            );
            tracing::trace!(opcode, country, player_id, ?relocation, "игрок перемещён при изгнании");
        }
        let sampled_at_ms = runtime.now_milliseconds();
        game
            .country_handler_mut()
            .country_mut(country)
            .expect("country owner проверен перед AddToExileList")
            .add_to_exile_list(player_id, sampled_at_ms);
        mutation_applied = true;
        tracing::trace!(opcode, country, player_id, sampled_at_ms, "игрок добавлен в список изгнанных");
    }

    let player_ids = game
        .country_handler()
        .country(country)
        .expect("country owner жив до exile response")
        .exile_player_ids();
    let mut response = CMessage::new(0x0006_030e);
    response.base_mut().add_long(player_id);
    response.base_mut().add_byte(u8::from(mutation_applied));
    response.base_mut().add_byte(country);
    response.base_mut().add_long(player_ids.len() as i32);
    for &exiled_player_id in &player_ids {
        response.base_mut().add_long(exiled_player_id);
    }
    let delivery = response.send(game, false);
    tracing::trace!(
        opcode,
        country,
        country_complete = decoded_country.is_some(),
        player_id,
        player_id_complete = decoded_player_id.is_some(),
        ?guard_country,
        ?extra_legacy_long,
        mutation_applied,
        exile_count = player_ids.len(),
        ?delivery,
        "ответ об изгнании отправлен World"
    );
}

fn dispatch_country_governance_effect_message<Runtime: GameCountryWarRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
    opcode: u32,
) {
    if opcode == 0x7ff0c {
        let country = message.base_mut().get_char().unwrap_or(0) as u8;
        let player_id = message.base_mut().get_long().unwrap_or(0);
        let Some(player) = game.find_player_mut(player_id) else {
            tracing::warn!(opcode, country, player_id, "эффект управления страной не применён: игрок отсутствует");
            return;
        };
        let player_name = player.shape().base_object().get_name().to_vec();
        player.reset_murder_counters();
        let text =
            format_country_player_name_notice(game.get_string_by_id(b"GS0023"), &player_name);
        for recipient_id in game.player_ids_in_country(u32::from(country)) {
            let mut notice = CMessage::new(0x000b_f815);
            notice.add_byte(1);
            notice.add_byte(8);
            add_country_legacy_c_string(&mut notice, b"");
            add_country_legacy_c_string(&mut notice, &text);
            let delivery = notice.send_to_player(game.net_server(), recipient_id);
            tracing::trace!(opcode, country, player_id, recipient_id, delivery, "уведомление о прощении отправлено игроку");
        }
        let mut counters = CMessage::new(0x000b_f70e);
        counters.add_long(player_id);
        counters.add_long(0);
        counters.add_long(0);
        let around_delivery = game.send_player_shape_around(player_id, None, &counters);
        tracing::trace!(
            opcode,
            country,
            player_id,
            ?around_delivery,
            "прощение игрока применено"
        );
        return;
    }

    if opcode == 0x7ff0d {
        let country = message.base_mut().get_char().unwrap_or(0) as u8;
        if game.country_handler().country(country).is_none() {
            tracing::warn!(opcode, country, "молчание не применено: страна отсутствует");
            return;
        }
        let player_id = message.base_mut().get_long().unwrap_or(0);
        if game.find_player(player_id).is_none() {
            tracing::warn!(opcode, country, player_id, "молчание не применено: игрок отсутствует");
            return;
        }
        let Some(minutes) = game.country_param().silence_time() else {
            tracing::error!(opcode, country, player_id, field = "_silence_time", "молчание не применено: параметр страны отсутствует");
            return;
        };
        let sampled_at_ms = runtime.now_milliseconds();
        game.find_player_mut(player_id)
            .expect("0x7FF0D player проверен до clock sample")
            .set_silence(minutes, sampled_at_ms);
        tracing::trace!(opcode, country, player_id, minutes, sampled_at_ms, "молчание игрока применено");
        return;
    }

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let country = message.base_mut().get_char().unwrap_or(0) as u8;
    if game.find_player(player_id).is_none() {
        tracing::warn!(opcode, country, player_id, "управляющая точка не опубликована: игрок отсутствует");
        return;
    }
    let Some(country_owner) = game.country_handler_mut().country_mut(country) else {
        tracing::warn!(opcode, country, player_id, "управляющая точка не опубликована: страна отсутствует");
        return;
    };
    let recorded_king_id = country_owner.country_information(1);
    if recorded_king_id != player_id {
        tracing::warn!(opcode, country, player_id, recorded_king_id, "управляющая точка не опубликована: игрок не является правителем");
        return;
    }
    let control_point = message.base_mut().get_long().unwrap_or(0);
    let mut response = CMessage::new(0x000c_030e);
    response.add_long(control_point);
    let delivery = response.send_to_player(game.net_server(), player_id);
    tracing::trace!(opcode, country, player_id, control_point, delivery, "управляющая точка опубликована");
}

fn format_country_player_name_notice(template: &[u8], player_name: &[u8]) -> Vec<u8> {
    let template = &template[..template
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(template.len())];
    let player_name = &player_name[..player_name
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(player_name.len())];
    let mut text = Vec::with_capacity(template.len().saturating_add(player_name.len()));
    if let Some(marker) = template.windows(2).position(|window| window == b"%s") {
        text.extend_from_slice(&template[..marker]);
        text.extend_from_slice(player_name);
        text.extend_from_slice(&template[marker + 2..]);
    } else {
        text.extend_from_slice(template);
    }
    text.truncate(255);
    text
}

fn dispatch_country_direct_response_message(
    message: &mut CMessage,
    game: &mut CGame,
    opcode: u32,
) {
    if opcode == 0x7ff05 {
        let decoded_country = message.base_mut().get_char();
        let country = decoded_country.unwrap_or(0) as u8;
        let decoded_king_id = message.base_mut().get_long();
        let king_id = decoded_king_id.unwrap_or(0);
        let Some(country_owner) = game.country_handler_mut().country_mut(country) else {
            tracing::warn!(
                opcode,
                country,
                country_complete = decoded_country.is_some(),
                king_id,
                king_id_complete = decoded_king_id.is_some(),
                "ответ страны не применён: страна отсутствует"
            );
            return;
        };
        country_owner.set_king_id(king_id);
        tracing::trace!(
            opcode,
            country,
            country_complete = decoded_country.is_some(),
            king_id,
            king_id_complete = decoded_king_id.is_some(),
            "правитель страны обновлён"
        );
        return;
    }

    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let response_type = match opcode {
        0x7ff07 => 0x000c_0304,
        0x7ff08 => 0x000c_0305,
        _ => unreachable!("direct country response opcode проверен caller-ом"),
    };
    message.set_message_type(response_type);
    let delivery = message.send_to_player(game.net_server(), player_id);
    tracing::trace!(
        opcode,
        player_id,
        player_id_complete = decoded_player_id.is_some(),
        response_type,
        delivery,
        "прямой ответ страны передан игроку"
    );
}

fn dispatch_country_information_change_message(
    message: &mut CMessage,
    game: &mut CGame,
) {
    let decoded_country = message.base_mut().get_char();
    let country = decoded_country.unwrap_or(0) as u8;
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let decoded_job = message.base_mut().get_char();
    let job = decoded_job.unwrap_or(0) as u8;
    let decoded_active = message.base_mut().get_char();
    let active = decoded_active.unwrap_or(0) as u8;
    let Some(country_owner) = game.country_handler_mut().country_mut(country) else {
        tracing::warn!(country, player_id, job, active, "сведения о стране не изменены: страна отсутствует");
        return;
    };
    country_owner.set_country_information(job, player_id, active);
    if game.find_player(player_id).is_none() {
        tracing::debug!(country, player_id, job, active, "сведения о стране изменены без публикации: игрок отсутствует");
        return;
    }
    let client_job = if active == 2 { 0 } else { job };
    let mut publication = CMessage::new(0x000c_0302);
    publication.add_long(player_id);
    publication.add_byte(client_job);
    let around_delivery = game.send_player_shape_around(player_id, None, &publication);
    tracing::trace!(
        country,
        country_complete = decoded_country.is_some(),
        player_id,
        player_id_complete = decoded_player_id.is_some(),
        job,
        job_complete = decoded_job.is_some(),
        active,
        active_complete = decoded_active.is_some(),
        client_job,
        ?around_delivery,
        "изменение сведений о стране опубликовано"
    );
}

fn dispatch_player_country_change_message(
    message: &mut CMessage,
    game: &mut CGame,
) {
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let decoded_country = message.base_mut().get_long();
    let country = decoded_country.unwrap_or(0);
    let Some(player) = game.find_player_mut(player_id) else {
        tracing::warn!(
            player_id,
            player_id_complete = decoded_player_id.is_some(),
            country,
            country_complete = decoded_country.is_some(),
            "страна игрока не изменена: игрок отсутствует"
        );
        return;
    };
    player.apply_world_country(country);
    let mut publication = CMessage::new(0x000c_0301);
    publication.add_long(country);
    publication.add_long(player_id);
    let around_delivery = game.send_player_shape_around(player_id, None, &publication);
    tracing::trace!(
        player_id,
        player_id_complete = decoded_player_id.is_some(),
        country,
        country_complete = decoded_country.is_some(),
        ?around_delivery,
        "изменение страны игрока опубликовано"
    );
}

fn dispatch_country_war_entry_message<Runtime: GameCountryWarRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) {
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let Some(player) = game.find_player(player_id) else {
        tracing::warn!(player_id, player_id_complete = decoded_player_id.is_some(), "вход в войну стран отклонён: игрок отсутствует");
        return;
    };
    let country = player.country();
    let war_region_id = game
        .country_war_sys()
        .get_war_region_for_country(i32::from(country));
    let camp = game.country_war_sys().get_war_camp(i32::from(country));
    if !matches!(camp, 0 | 1) {
        tracing::warn!(player_id, country, camp, "вход в войну стран отклонён: лагерь отсутствует");
        return;
    }
    let Some(owner) = game.take_region_owner(war_region_id) else {
        tracing::warn!(player_id, country, camp, war_region_id, "вход в войну стран отклонён: регион отсутствует");
        return;
    };
    let ServerRegionOwner::Country(region) = &owner else {
        game.restore_region_owner(owner);
        tracing::error!(player_id, country, camp, war_region_id, "вход в войну стран отклонён: неверный вид региона");
        return;
    };
    let position = region.country_war_entry_position(camp, runtime);
    game.restore_region_owner(owner);
    let position = match position {
        Ok(Some(position)) => position,
        Ok(None) => {
            tracing::warn!(player_id, country, camp, war_region_id, "вход в войну стран отклонён: область входа отсутствует");
            return;
        }
        Err(block) => {
            tracing::warn!(player_id, country, camp, war_region_id, ?block, "вход в войну стран отклонён: позиция недоступна");
            return;
        }
    };

    let increment = game.country_param().exploit_increment();
    let maximum = game.country_param().max_exploit();
    let (Some(increment), Some(maximum)) = (increment, maximum) else {
        tracing::error!(player_id, country, camp, war_region_id, ?increment, ?maximum, "вход в войну стран отклонён: параметры заслуг отсутствуют");
        return;
    };
    let region_change = game.change_player_region(
        player_id,
        war_region_id,
        position.x,
        position.y,
        -1,
        0,
        0,
        0,
        runtime,
    );
    let exploit = {
        let player = game
            .find_player_mut(player_id)
            .expect("0x9050B сохраняет live player до ChangeRegion boundary");
        player.set_exploit(player.exploit().wrapping_add(increment as u32), maximum)
    };
    let mut exploit_message = CMessage::new(0x000b_f72e);
    exploit_message.add_ulong(exploit);
    let exploit_delivery = exploit_message.send_to_player(game.net_server(), player_id);

    let notice = format_country_war_exploit_notice(game.get_string_by_id(b"GS0024"), increment);
    let mut notice_message = CMessage::new(0x000b_f816);
    notice_message.add_byte(0);
    add_country_legacy_c_string(&mut notice_message, &notice);
    let notice_delivery = notice_message.send_to_player(game.net_server(), player_id);

    tracing::trace!(
        player_id,
        player_id_complete = decoded_player_id.is_some(),
        country,
        region_id = war_region_id,
        camp,
        ?position,
        ?region_change,
        exploit,
        exploit_delivery,
        notice_delivery,
        "вход в войну стран завершён"
    );
}

fn format_country_war_exploit_notice(template: &[u8], increment: i32) -> Vec<u8> {
    let template = &template[..template
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(template.len())];
    let value = increment.to_string();
    let mut text = Vec::with_capacity(template.len().saturating_add(value.len()));
    if let Some(marker) = template.windows(2).position(|window| window == b"%d") {
        text.extend_from_slice(&template[..marker]);
        text.extend_from_slice(value.as_bytes());
        text.extend_from_slice(&template[marker + 2..]);
    } else {
        text.extend_from_slice(template);
    }
    text.truncate(255);
    text
}

fn add_country_legacy_c_string(message: &mut CMessage, value: &[u8]) {
    let value = &value[..value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len())];
    message.base_mut().add(value);
    message.add_byte(0);
}

fn dispatch_country_governance_message(
    message: &mut CMessage,
    game: &CGame,
    opcode: u32,
) {
    message.resolve_player_context(game);
    let player_id = message.player_id();
    let Some(player_id) = player_id else {
        tracing::warn!(opcode, "запрос управления страной отклонён: контекст игрока отсутствует");
        return;
    };
    let Some(player) = game.find_player(player_id) else {
        tracing::warn!(opcode, player_id, "запрос управления страной отклонён: игрок отсутствует");
        return;
    };
    let country = player.country();
    let target_id = match opcode {
        0x90504..=0x90507 | 0x90509..=0x9050a => Some(message.base_mut().get_long().unwrap_or(0)),
        0x90508 => Some(player_id),
        _ => None,
    };
    let target = target_id.and_then(|target_id| game.find_player(target_id));
    if opcode == 0x9050a && target.is_none() {
        tracing::debug!(opcode, player_id, country, ?target_id, "запрос управления страной отклонён: цель отсутствует");
        return;
    }
    let rejection = match opcode {
        0x90504 if target.is_none() => Some(b"GS0047".as_slice()),
        0x90504 if target.is_some_and(changing_location) => Some(b"GS0017".as_slice()),
        0x90505 if target.is_none() => Some(b"WS0063".as_slice()),
        0x90505 if target.is_some_and(changing_location) => Some(b"GS0018".as_slice()),
        0x90506 if target.is_some_and(changing_location) => Some(b"GS0019".as_slice()),
        0x90507 if target.is_none() => Some(b"WS0084".as_slice()),
        0x90507 if target.is_some_and(changing_location) => Some(b"GS0020".as_slice()),
        0x90508 if target.is_none() => Some(b"WS0080".as_slice()),
        0x90508 if target.is_some_and(changing_location) => Some(b"GS0021".as_slice()),
        0x90509 if target.is_none() => Some(b"WS0072".as_slice()),
        0x90509 if target.is_some_and(changing_location) => Some(b"GS0021".as_slice()),
        0x9050a if target.is_some_and(changing_location) => Some(b"GS0022".as_slice()),
        _ => None,
    };
    if let Some(string_id) = rejection {
        let mut response = CMessage::new(0x000c_030d);
        response.base_mut().add(game.get_string_by_id(string_id));
        response.add_byte(0);
        let delivery = response.send_to_player(game.net_server(), player_id);
        tracing::debug!(
            opcode,
            player_id,
            country,
            string_id = ?String::from_utf8_lossy(string_id),
            delivery,
            "запрос управления страной отклонён"
        );
        return;
    }

    let world_type = match opcode {
        0x90502 => 0x0006_0306,
        0x90503 => 0x0006_0307,
        0x90504 => 0x0006_0308,
        0x90505 => 0x0006_0309,
        0x90506 => 0x0006_030a,
        0x90507 => 0x0006_030b,
        0x90508 => 0x0006_030c,
        0x90509 => 0x0006_030d,
        0x9050a => 0x0006_030f,
        _ => unreachable!("governance opcode проверен перед dispatcher-ом"),
    };
    message.set_message_type(world_type);
    message.add_long(player_id);
    message.add_byte(country);
    let delivery = message.send(game, false);
    tracing::trace!(
        opcode,
        player_id,
        country,
        world_type,
        ?target_id,
        ?delivery,
        "запрос управления страной передан World"
    );
}

fn changing_location(player: &crate::gameserver::appserver::player::CPlayer) -> bool {
    player.in_changing_region() || player.in_changing_server()
}

struct GameCountryWarContext<'a, Runtime> {
    game: &'a mut CGame,
    runtime: &'a mut Runtime,
}

impl<Runtime: GameCountryWarRuntime> CountryWarMessageContext
    for GameCountryWarContext<'_, Runtime>
{
    type Region = i32;
    type SideError = CountryBattleStateBlock;

    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region> {
        matches!(
            self.game.find_region(region_id),
            Some(ServerRegionOwner::Country(_))
        )
        .then_some(region_id)
    }

    fn set_country_sides(
        &mut self,
        region: Self::Region,
        defend_country: i32,
        attack_country: i32,
    ) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.set_country_sides(defend_country, attack_country);
        }
    }

    fn update_contend_player(&mut self, region: Self::Region) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.update_contend_player();
        }
    }

    fn on_declare_begin(&mut self, region: Self::Region, region_id: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_declare_begin(region_id);
        }
    }

    fn on_declare_end(&mut self, region: Self::Region, region_id: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_declare_end(region_id);
        }
    }

    fn on_prepare_begin(&mut self, region: Self::Region, region_id: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_prepare_begin(region_id);
        }
    }

    fn on_prepare_end(&mut self, region: Self::Region, region_id: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_prepare_end(region_id);
        }
    }

    fn on_war_start(&mut self, region: Self::Region, region_id: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_war_start(region_id);
        }
    }

    fn on_war_timeout(&mut self, region: Self::Region, region_id: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_war_timeout(region_id);
        }
    }

    fn on_war_end(&mut self, region: Self::Region, region_id: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_war_end(region_id);
        }
    }

    fn clear_country_region(&mut self, region: Self::Region) {
        self.game.clear_country_region(region, self.runtime);
    }

    fn country_region_side_bytes(
        &mut self,
        region: Self::Region,
    ) -> Result<(u8, u8), Self::SideError> {
        let Some(ServerRegionOwner::Country(region)) = self.game.find_region(region) else {
            unreachable!("country handle получен из того же синхронного CGame map")
        };
        region.country_side_bytes()
    }

    fn reset_country_war_result(&mut self, country: u8) {
        if let Some(country) = self.game.country_handler_mut().country_mut(country) {
            country.country_war_result = 0;
        }
    }

    fn on_country_flag_destroy(&mut self, region: Self::Region, region_id: i32, country: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_flag_destroy(region_id, country);
        }
    }

    fn set_country_war_result(&mut self, country: u8, result: i32) {
        if let Some(country) = self.game.country_handler_mut().country_mut(country) {
            country.country_war_result = result;
        }
    }
}

fn read_country_war_byte<SideError>(
    payload: &[u8],
    cursor: &mut usize,
) -> Result<u8, CountryWarMessageDispatchError<SideError>> {
    let offset = *cursor;
    let available = payload.len().saturating_sub(offset);
    let mut reader = LegacyReader::at(payload, offset).map_err(|_| CountryWarMessageDispatchError::UnexpectedEnd { offset, needed: 1, available })?;
    let value = reader.read_u8().map_err(|_| CountryWarMessageDispatchError::UnexpectedEnd { offset, needed: 1, available })?;
    *cursor = reader.position();
    Ok(value)
}

fn read_country_war_long<SideError>(
    payload: &[u8],
    cursor: &mut usize,
) -> Result<i32, CountryWarMessageDispatchError<SideError>> {
    let offset = *cursor;
    let available = payload.len().saturating_sub(offset);
    let mut reader = LegacyReader::at(payload, offset).map_err(|_| CountryWarMessageDispatchError::UnexpectedEnd { offset, needed: 4, available })?;
    let value = reader.read_i32().map_err(|_| CountryWarMessageDispatchError::UnexpectedEnd { offset, needed: 4, available })?;
    *cursor = reader.position();
    Ok(value)
}

struct CountryRegionAdapter<'a, Context>(&'a mut Context);

impl<Context: CountryWarMessageContext> CountryWarRegionContext
    for CountryRegionAdapter<'_, Context>
{
    type Region = Context::Region;

    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.0.find_country_region(region_id)
    }

    fn set_country_sides(
        &mut self,
        region: Self::Region,
        defend_country: i32,
        attack_country: i32,
    ) {
        self.0
            .set_country_sides(region, defend_country, attack_country);
    }

    fn update_contend_player(&mut self, region: Self::Region) {
        self.0.update_contend_player(region);
    }
}

struct CountryPhaseAdapter<'a, Context>(&'a mut Context);

impl<Context: CountryWarMessageContext> CountryWarPhaseContext
    for CountryPhaseAdapter<'_, Context>
{
    type Region = Context::Region;
    type SideError = Context::SideError;

    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.0.find_country_region(region_id)
    }

    fn on_declare_begin(&mut self, region: Self::Region, region_id: i32) {
        self.0.on_declare_begin(region, region_id);
    }

    fn on_declare_end(&mut self, region: Self::Region, region_id: i32) {
        self.0.on_declare_end(region, region_id);
    }

    fn on_prepare_begin(&mut self, region: Self::Region, region_id: i32) {
        self.0.on_prepare_begin(region, region_id);
    }

    fn on_prepare_end(&mut self, region: Self::Region, region_id: i32) {
        self.0.on_prepare_end(region, region_id);
    }

    fn on_war_start(&mut self, region: Self::Region, region_id: i32) {
        self.0.on_war_start(region, region_id);
    }

    fn on_war_timeout(&mut self, region: Self::Region, region_id: i32) {
        self.0.on_war_timeout(region, region_id);
    }

    fn on_war_end(&mut self, region: Self::Region, region_id: i32) {
        self.0.on_war_end(region, region_id);
    }

    fn clear_country_region(&mut self, region: Self::Region) {
        self.0.clear_country_region(region);
    }

    fn country_region_side_bytes(
        &mut self,
        region: Self::Region,
    ) -> Result<(u8, u8), Self::SideError> {
        self.0.country_region_side_bytes(region)
    }

    fn reset_country_war_result(&mut self, country: u8) {
        self.0.reset_country_war_result(country);
    }
}

struct CountryVictoryAdapter<'a, Context>(&'a mut Context);

impl<Context: CountryWarMessageContext> CountryWarVictoryContext
    for CountryVictoryAdapter<'_, Context>
{
    type Region = Context::Region;

    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.0.find_country_region(region_id)
    }

    fn on_flag_destroy(&mut self, region: Self::Region, region_id: i32, country: i32) {
        self.0.on_country_flag_destroy(region, region_id, country);
    }

    fn set_country_war_result(&mut self, country: u8, result: i32) {
        self.0.set_country_war_result(country, result);
    }
}
