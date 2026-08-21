//! Статус корпуса: `IMPLEMENTED_PARTIAL` для заявки союза `0x60118` и общего
//! session-result dispatch; остальной owner — `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Декомпилятор: Ghidra 12.1.2
//! Полный декомпилят хранится локально и не входит в распространяемый код.
//!
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! `OnOrgasysMessage` RVA `0x000A6110`. Exact диапазоны
//! `0x004A74C6..0x004A7509` и `0x004A7511..0x004A7543` исправляют повреждённый
//! RAW. Общий branch читает
//! `GetLONG64`, `GetLong`, именно `GetChar`, затем `GetLong`, то есть
//! `(session ID, cookie.second, result byte, cookie.first)`. `movzx` расширяет
//! result как unsigned byte до 32 бит. Затем вызывается уже восстановленный
//! `CNetSessionManager::OnSyncCallbackResult(session, first, second, &result)`;
//! manager проверяет cookie, синхронно вызывает endpoint и удаляет session.
//! Эту ветку разделяют opcode `0x60117/0x60119/0x60120/0x60122/0x60124/0x60131`.
//! `0x60118` читает `(master player ID, applicant faction ID)`, разрешает union
//! через `COrganizingCtrl::GetUnion(master player ID)` и при non-null вызывает
//! virtual `ApplyForJoin(applicant faction ID, 0, master player ID)`.
//! Exact `0x004A75FB..0x004A763F` подтверждает соседний `0x6011A`: один
//! `GetLong`, ordered `IsFactionMaster`, nullable faction lookup и virtual
//! `SetLWFunction(true)` в slot `+0x104`.
//! Exact `0x004A7644..0x004A771A` для `0x6011B` очищает 210-byte buffer,
//! вызывает `GetStr(..., 0xD2)`, читает player ID, разрешает его faction через
//! `IsFreePlayer`, копирует все восемь WORD полей одного `GetLocalTime` в
//! `tagTime` и вызывает virtual `CFaction::LeaveWord` в slot `+0x38`.
//!
//! Старый callback держал singleton-указатели и мутировал organizing state
//! непосредственно из `CNetSessionManager`. Rust endpoint вместо небезопасной
//! `'static` ссылки сохраняет terminal action в FIFO под `parking_lot::Mutex`;
//! единственный main-loop owner забирает её сразу после callback dispatch.
//! Confirmation send остаётся синхронным внутри `Beging`: route и клонируемый
//! `ServerCommandHandle` снимаются непосредственно перед созданием session.
//! Это техническая замена lifetime/lock plumbing, а не изменение wire, cookie,
//! timeout или порядка terminal actions.
//!
//! Legacy getters при нехватке возвращают ноль и не двигают cursor; Rust
//! сохраняет это через `unwrap_or(0)`, а не добавляет отсутствовавший общий
//! reject. Дополнительный хвост owner не проверял. Проверки exact payload и
//! GameServer ownership из старого Linux-донора к этой ветке EXE не относятся
//! и здесь не переносятся. `CMessage`/`CBaseMessage` и session manager уже
//! материализованы; функция ниже добавляет только конкретный opcode dispatch.

use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::nets::servers::ServerCommandHandle;
use crate::public::netsessionmanager::{CNetSessionManager, NetSessionCallbackOutcome};
use crate::public::date::TagTime;
use crate::worldserver::appworld::organizingsystem::faction::{
    FactionMemberInfoRequest, FactionOrganizingInfoContext,
};
use crate::worldserver::appworld::organizingsystem::organizingctrl::{
    COrganizingCtrl, OrganizingLeaveWordBlock, OrganizingLeaveWordEnableBlock,
    OrganizingLeaveWordEnableOutcome, OrganizingLeaveWordOutcome,
    OrganizingUnionApplyForJoinDispatchBlock, OrganizingUnionApplyForJoinOutcome,
};
use crate::worldserver::appworld::organizingsystem::organizing::TagTimeValue;
use crate::worldserver::appworld::organizingsystem::union::{
    UnionAddFactionEffects, UnionApplicationEndpointBlock, UnionApplicationSessionBlock,
    UnionApplicationSessionReport, UnionApplicationSessionRequest, UnionApplicationSessionRuntime,
    UnionApplicationTerminal, UnionApplyForJoinEffects, UnionFormatArgument,
    begin_union_application_session,
};
use crate::worldserver::worldserver::game::CGame;

const SESSION_RESULT_MESSAGE_TYPES: [i32; 6] =
    [0x60117, 0x60119, 0x60120, 0x60122, 0x60124, 0x60131];
const UNION_APPLICATION_MESSAGE_TYPE: i32 = 0x60118;
const ENABLE_LEAVE_WORD_MESSAGE_TYPE: i32 = 0x6011A;
const LEAVE_WORD_MESSAGE_TYPE: i32 = 0x6011B;
const LEAVE_WORD_INPUT_CAPACITY: usize = 0xD2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct QueuedUnionApplicationTerminal {
    pub(crate) union_id: i32,
    pub(crate) applicant_faction_id: i32,
    pub(crate) terminal: UnionApplicationTerminal,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionApplicationConfirmationDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Default)]
struct WorldUnionApplicationRuntimeState {
    terminals: Mutex<VecDeque<QueuedUnionApplicationTerminal>>,
    confirmations: Mutex<VecDeque<UnionApplicationConfirmationDelivery>>,
    blocks: Mutex<VecDeque<UnionApplicationEndpointBlock>>,
}

/// Process-lifetime очередь между `CNetSessionManager` и organizing owner-ом.
#[derive(Clone, Default)]
pub(crate) struct WorldUnionApplicationRuntimeOwner {
    state: Arc<WorldUnionApplicationRuntimeState>,
}

impl WorldUnionApplicationRuntimeOwner {
    fn endpoint(
        &self,
        sender: Option<ServerCommandHandle>,
        game_server_id: i32,
    ) -> Arc<dyn UnionApplicationSessionRuntime> {
        Arc::new(WorldUnionApplicationEndpointRuntime {
            state: Arc::clone(&self.state),
            sender,
            game_server_id,
        })
    }

    pub(crate) fn pop_terminal(&self) -> Option<QueuedUnionApplicationTerminal> {
        self.state.terminals.lock().pop_front()
    }

    pub(crate) fn take_confirmations(&self) -> Vec<UnionApplicationConfirmationDelivery> {
        self.state.confirmations.lock().drain(..).collect()
    }

    pub(crate) fn take_blocks(&self) -> Vec<UnionApplicationEndpointBlock> {
        self.state.blocks.lock().drain(..).collect()
    }
}

struct WorldUnionApplicationEndpointRuntime {
    state: Arc<WorldUnionApplicationRuntimeState>,
    sender: Option<ServerCommandHandle>,
    game_server_id: i32,
}

impl UnionApplicationSessionRuntime for WorldUnionApplicationEndpointRuntime {
    fn send_union_application_confirmation(
        &self,
        recipient_player_id: i32,
        message: &CMessage,
    ) {
        let result = message.send_to_map_id(self.sender.as_ref(), self.game_server_id);
        self.state
            .confirmations
            .lock()
            .push_back(UnionApplicationConfirmationDelivery {
                recipient_player_id,
                game_server_id: self.game_server_id,
                result,
            });
    }

    fn finish_union_application(
        &self,
        union_id: i32,
        applicant_faction_id: i32,
        terminal: UnionApplicationTerminal,
    ) {
        self.state
            .terminals
            .lock()
            .push_back(QueuedUnionApplicationTerminal {
                union_id,
                applicant_faction_id,
                terminal,
            });
    }

    fn block_union_application_endpoint(&self, block: UnionApplicationEndpointBlock) {
        self.state.blocks.lock().push_back(block);
    }
}

/// Живые callback-и универсальной инфраструктуры, не принадлежащие union state.
pub(crate) struct WorldUnionApplicationEffectCallbacks<'a> {
    pub(crate) random: &'a mut dyn FnMut(i32) -> i32,
    pub(crate) world_string: &'a mut dyn FnMut(&[u8]) -> Vec<u8>,
    pub(crate) format_world_string:
        &'a mut dyn FnMut(&[u8], &[UnionFormatArgument<'_>]) -> Vec<u8>,
    pub(crate) put_war_log: &'a mut dyn FnMut(&[u8]),
    pub(crate) refresh_owned_city: &'a mut dyn FnMut(i32, i32, i32),
}

/// Тонкий concrete adapter готовых string/transport/session owners.
pub(crate) struct WorldUnionApplicationEffects<'a> {
    game: &'a CGame,
    manager: &'a CNetSessionManager,
    runtime: &'a WorldUnionApplicationRuntimeOwner,
    callbacks: WorldUnionApplicationEffectCallbacks<'a>,
}

impl<'a> WorldUnionApplicationEffects<'a> {
    pub(crate) fn new(
        game: &'a CGame,
        manager: &'a CNetSessionManager,
        runtime: &'a WorldUnionApplicationRuntimeOwner,
        callbacks: WorldUnionApplicationEffectCallbacks<'a>,
    ) -> Self {
        Self {
            game,
            manager,
            runtime,
            callbacks,
        }
    }
}

impl UnionApplyForJoinEffects for WorldUnionApplicationEffects<'_> {
    type SessionReport = UnionApplicationSessionReport;
    type SessionBlock = UnionApplicationSessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        (self.callbacks.world_string)(string_id)
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }

    fn begin_union_application_session(
        &mut self,
        request: UnionApplicationSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock> {
        // `Beging` вызывает `DoAsyncCall` синхронно; route и клонируемый
        // transport handle снимаются непосредственно перед session creation.
        let game_server_id = self
            .game
            .game_server_number_by_player_id(request.recipient_player_id);
        let endpoint = self
            .runtime
            .endpoint(self.game.current_game_server_sender(), game_server_id);
        begin_union_application_session(self.manager, request, endpoint, |upper_bound| {
            (self.callbacks.random)(upper_bound)
        })
    }
}

impl UnionAddFactionEffects for WorldUnionApplicationEffects<'_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        (self.callbacks.world_string)(string_id)
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8> {
        (self.callbacks.format_world_string)(string_id, arguments)
    }

    fn put_war_log(&mut self, text: &[u8]) {
        (self.callbacks.put_war_log)(text);
    }

    fn refresh_owned_city(&mut self, region_id: i32, faction_id: i32, union_id: i32) {
        (self.callbacks.refresh_owned_city)(region_id, faction_id, union_id);
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionOrganizingInfoContext for WorldUnionApplicationEffects<'_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some((self.callbacks.world_string)(string_id))
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingSessionResultDispatch {
    NotHandled,
    Delivered {
        message_type: i32,
        session_id: i64,
        cookie_first: i32,
        cookie_second: i32,
        result: i32,
        outcome: NetSessionCallbackOutcome,
    },
}

/// Выполняет общий session-result branch `OnOrgasysMessage`.
pub(crate) fn dispatch_organizing_session_result(
    message: &mut CMessage,
    manager: &CNetSessionManager,
) -> OrganizingSessionResultDispatch {
    let message_type = message.message_type();
    if !SESSION_RESULT_MESSAGE_TYPES.contains(&message_type) {
        return OrganizingSessionResultDispatch::NotHandled;
    }

    let session_id = message.base_mut().get_long64().unwrap_or(0);
    let cookie_second = message.base_mut().get_long().unwrap_or(0);
    let result = message
        .base_mut()
        .get_char()
        .map_or(0, |result| i32::from(result as u8));
    let cookie_first = message.base_mut().get_long().unwrap_or(0);
    let outcome =
        manager.on_sync_callback_result(session_id, cookie_first, cookie_second, &result);
    OrganizingSessionResultDispatch::Delivered {
        message_type,
        session_id,
        cookie_first,
        cookie_second,
        result,
        outcome,
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingUnionApplicationDispatch<SessionReport> {
    pub(crate) master_player_id: i32,
    pub(crate) applicant_faction_id: i32,
    pub(crate) outcome: OrganizingUnionApplyForJoinOutcome<SessionReport>,
}

/// Читает и выполняет точную producer-ветвь union application.
pub(crate) fn dispatch_union_application<Effects>(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    effects: &mut Effects,
) -> Option<
    Result<
        OrganizingUnionApplicationDispatch<Effects::SessionReport>,
        OrganizingUnionApplyForJoinDispatchBlock<Effects::SessionBlock>,
    >,
>
where
    Effects: UnionApplyForJoinEffects,
{
    if message.message_type() != UNION_APPLICATION_MESSAGE_TYPE {
        return None;
    }

    let master_player_id = message.base_mut().get_long().unwrap_or(0);
    let applicant_faction_id = message.base_mut().get_long().unwrap_or(0);
    Some(
        organizing
            .apply_for_union_join(
                game,
                master_player_id,
                applicant_faction_id,
                0,
                master_player_id,
                effects,
            )
            .map(|outcome| OrganizingUnionApplicationDispatch {
                master_player_id,
                applicant_faction_id,
                outcome,
            }),
    )
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingLeaveWordEnableDispatch {
    pub(crate) player_id: i32,
    pub(crate) outcome: OrganizingLeaveWordEnableOutcome,
}

/// Выполняет `0x6011A`: master lookup и virtual `SetLWFunction(true)`.
pub(crate) fn dispatch_leave_word_enable<Context>(
    message: &mut CMessage,
    organizing: &mut COrganizingCtrl,
    context: &mut Context,
) -> Option<Result<OrganizingLeaveWordEnableDispatch, OrganizingLeaveWordEnableBlock>>
where
    Context: FactionOrganizingInfoContext,
{
    if message.message_type() != ENABLE_LEAVE_WORD_MESSAGE_TYPE {
        return None;
    }
    let player_id = message.base_mut().get_long().unwrap_or(0);
    Some(
        organizing
            .enable_leave_word_for_master(player_id, context)
            .map(|outcome| OrganizingLeaveWordEnableDispatch { player_id, outcome }),
    )
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingLeaveWordDispatch {
    pub(crate) player_id: i32,
    pub(crate) content: Vec<u8>,
    pub(crate) time: TagTimeValue,
    pub(crate) outcome: OrganizingLeaveWordOutcome,
}

/// Выполняет `0x6011B`: bounded C-string, player membership и `LeaveWord`.
pub(crate) fn dispatch_leave_word(
    message: &mut CMessage,
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
) -> Option<Result<OrganizingLeaveWordDispatch, OrganizingLeaveWordBlock>> {
    if message.message_type() != LEAVE_WORD_MESSAGE_TYPE {
        return None;
    }

    let mut content = message
        .base_mut()
        .get_str_bytes(LEAVE_WORD_INPUT_CAPACITY)
        .unwrap_or_default();
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let local_time = TagTime::local_now();
    let time = TagTimeValue {
        year: local_time.year,
        month: local_time.month,
        day_of_week: local_time.day_of_week,
        day: local_time.day,
        hour: local_time.hour,
        minute: local_time.minute,
        second: local_time.second,
        milliseconds: local_time.milliseconds,
    };
    Some(
        organizing
            .leave_word_for_player(game, player_id, &mut content, time)
            .map(|outcome| OrganizingLeaveWordDispatch {
                player_id,
                content,
                time,
                outcome,
            }),
    )
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\organsysmessage.cpp

// ============================================================================
// FUNCTION: OnOrgasysMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\organsysmessage.cpp:30
// RVA: 0x000A6110
// ADDRESS: 004a6110
// PROTOTYPE: void __cdecl OnOrgasysMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ac1b5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\organsysmessage.cpp
// RVA: 0x000AC1B5
// ADDRESS: 004ac1b5
// PROTOTYPE: undefined Catch@004ac1b5()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ac2a9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\organsysmessage.cpp
// RVA: 0x000AC2A9
// ADDRESS: 004ac2a9
// PROTOTYPE: undefined Catch@004ac2a9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ac3c6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\organsysmessage.cpp
// RVA: 0x000AC3C6
// ADDRESS: 004ac3c6
// PROTOTYPE: undefined Catch@004ac3c6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00532c30
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\organsysmessage.cpp
// RVA: 0x00132C30
// ADDRESS: 00532c30
// PROTOTYPE: undefined Unwind@00532c30()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//














// COMPONENT_VARIANT_END: WorldServer
