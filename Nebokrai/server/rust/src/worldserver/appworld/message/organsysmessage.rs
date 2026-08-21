//! Статус корпуса: `IMPLEMENTED_PARTIAL` для достигнутых organizing opcode,
//! включая заявку союза `0x60118`, общий session-result dispatch, billboard
//! `0x60125`, улучшение фракции `0x60126`, запрос значка `0x60127`, выбор
//! вкладчика `0x60128` и вклад опыта `0x60129`; остальной owner —
//! `UNKNOWN` (исследовательский декомпилят хранится локально).
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
//! Exact `0x004A771F..0x004A776D` для `0x6011C` читает leave-word ID, затем
//! player ID, разрешает faction через `IsFreePlayer` и вызывает virtual
//! `CFaction::EditLeaveWord(player ID, leave-word ID, EOperator::Delete)` в
//! slot `+0x3C`.
//! Exact `0x004A7772..0x004A784A` для `0x6011D` очищает 0x5000-byte buffer,
//! вызывает `GetStr(..., 0x5000)`, читает player ID, разрешает faction через
//! `IsFreePlayer`, снимает один local `tagTime` и вызывает virtual
//! `CFaction::Pronounce` в slot `+0x34`.
//! Exact shared leaf `0x004A796A..0x004A7971` для `0x60121/0x60123` вызывает
//! один `GetLong` и не использует возвращённое значение.
//! Exact `0x004A69A9..0x004A6EC4` для `0x6011E` читает `(request ID, cookie,
//! player ID, page)`, проверяет faction/master и формирует ответ `0x7FE18`.
//! Пустые ответы не содержат page/payload; успешный ответ содержит total,
//! исходный page и 11-элементный `AddDeclareWarFactionInfoToByteArray` payload.
//! Exact `0x004A784F..0x004A7965` для `0x6011F` читает `(request ID, cookie,
//! player ID, target faction ID, war type)`, декодирует полный player snapshot
//! с текущего message cursor, снимает local `tagTime`, вызывает готовый
//! `CFactionWarSys::DigUpTheHatchet` и отправляет `0x7FE19`. Стоимость войны
//! попадает в ответ только при истинном результате; сам WorldServer её здесь
//! не списывает.
//! Exact `0x004A812D..0x004A82C8` для `0x60125` при первом входе лениво и
//! навсегда локализует `WS0134/WS0133/WS0132`, затем читает `(request ID,
//! billboard type)`. Значение выше `2` не получает ответа. Для `0/1/2`
//! строится socket-response `0x7FE1D`: request ID, соответствующий C-string
//! title и payload готового `AddFactionBillboardToByteArray`, после чего
//! выполняется явный `Update`. Несовпадение нумерации сохранено: serializer
//! распознаёт `1/2/3`, поэтому type `0` пишет count `0`, `1` отдаёт members,
//! `2` — offense; defense недостижим. Отрицательный type исходник не отсекал
//! и индексировал память перед process-static массивом; Rust останавливает
//! этот внутренний out-of-bounds как локальный `BLOCKED_MISSING_FACT`.
//! Exact `0x004A799C..0x004A79FE` для `0x60126` читает `(faction ID,
//! player ID)`, ищет online player, декодирует в него полный snapshot с
//! текущего message cursor и только затем повторно разрешает faction. При
//! non-null faction вызывается virtual slot `+0x50`, то есть уже
//! восстановленный `CFaction::Upgrade(long)`, с полным 32-битным player ID;
//! bool-result игнорируется и wire-ответ не формируется. Приведение к `char`
//! в RAW было артефактом декомпиляции. Дополнительных ownership/tail-проверок
//! старого Linux-донора в EXE нет, поэтому они не перенесены.
//! Exact `0x004A7A03..0x004A7A78` для `0x60127` читает `(faction ID,
//! player ID)`, дважды выполняет nullable faction lookup, а между успешными
//! lookup снимает один полный local `tagTime`. Затем virtual slot `+0x54`
//! вызывает готовый `CFaction::UploadIcon(player ID, &time)`. Online-player
//! lookup, payload decoder и wire-ответ отсутствуют; сам `UploadIcon` время не
//! читает и реализует только master/property/interval gate. Добавленные старым
//! Linux-донором ownership и exact-tail проверки поэтому не переносятся.
//! Exact `0x004A80CF..0x004A8128` для `0x60128` читает `(target player ID,
//! enabled long, requester player ID)`, преобразует enabled строго через
//! `!= 0`, разрешает faction requester-а через ordered `IsFreePlayer` и
//! вызывает virtual slot `+0x128`, то есть готовый
//! `SetControbuter(requester, target, enabled)`. Online-player ownership,
//! payload decoder и wire-ответ отсутствуют. Linux-донор верно подсказал форму
//! трёх полей, но его exact-tail/ownership проверки в EXE не подтверждаются.
//! Exact `0x004A82CD..0x004A83DB` для `0x60129` читает `(faction ID,
//! player ID, experience delta)`, разрешает faction, проверяет virtual
//! `IsControbute(player ID)` в slot `+0x124`, дважды читает прежний опыт через
//! `+0x68` и передаёт в `SetExp` (`+0x6C`) их машинную 32-битную сумму с delta.
//! Результат `SetExp` не влияет на дальнейший log gate. При включённых setup и
//! faction-exp флагах online-player даёт фактические ID/name для SQL
//! `faction_experience_log`; колонки `before_exp/exp` получают старый опыт и
//! именно delta. Wire-ответ отсутствует. Rust callback передаёт typed поля
//! владельцу DB-очереди вместо `_sprintf` в 256-байтовый heap-buffer и тем
//! самым устраняет внутренние overflow/leak, не меняя DB-контракт.
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
use std::ffi::CString;
use std::sync::{Arc, OnceLock};

use parking_lot::Mutex;

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::nets::servers::ServerCommandHandle;
use crate::public::netsessionmanager::{CNetSessionManager, NetSessionCallbackOutcome};
use crate::public::date::TagTime;
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, GoodsOriginalNameIndex, query_goods_name,
};
use crate::worldserver::appworld::organizingsystem::faction::{
    FactionContributorContext, FactionExperienceBlock, FactionExperienceUpdate,
    FactionLevelContext, FactionMemberInfoRequest, FactionOrganizingInfoContext,
    FactionUpgradeBlock, FactionUpgradeContext, FactionUpgradeFormatArgument,
    FactionUpgradeOutcome, FactionUploadIconBlock, FactionUploadIconContext,
    FactionUploadIconOutcome,
};
use crate::worldserver::appworld::organizingsystem::factionwarsys::{
    CFactionWarSys, FactionWarDeclarationBlock, FactionWarDeclarationOutcome,
};
use crate::worldserver::appworld::organizingsystem::organizingctrl::{
    COrganizingCtrl, DeclareWarFactionPage, DeclareWarFactionPageBlock, FreePlayerLookup,
    OrganizingContributorBlock, OrganizingContributorOutcome,
    OrganizingFactionExperienceMutation,
    OrganizingLeaveWordBlock, OrganizingLeaveWordEditBlock,
    OrganizingLeaveWordEditOutcome, OrganizingLeaveWordEnableBlock,
    OrganizingFactionWarDeclarationBlock, WorldFactionWarDeclarationEffects,
    OrganizingLeaveWordEnableOutcome, OrganizingLeaveWordOutcome, OrganizingPronounceBlock,
    OrganizingPronounceOutcome, OrganizingUnionApplyForJoinDispatchBlock,
    OrganizingUnionApplyForJoinOutcome,
};
use crate::worldserver::appworld::organizingsystem::organizing::{EOperator, TagTimeValue};
use crate::worldserver::appworld::organizingsystem::organizingparam::COrganizingParam;
use crate::worldserver::appworld::organizingsystem::union::{
    UnionAddFactionEffects, UnionApplicationEndpointBlock, UnionApplicationSessionBlock,
    UnionApplicationSessionReport, UnionApplicationSessionRequest, UnionApplicationSessionRuntime,
    UnionApplicationTerminal, UnionApplyForJoinEffects, UnionFormatArgument,
    begin_union_application_session,
};
use crate::worldserver::appworld::player::{PlayerCodecError, PlayerPropertyCoefficients};
use crate::worldserver::worldserver::game::CGame;

const SESSION_RESULT_MESSAGE_TYPES: [i32; 6] =
    [0x60117, 0x60119, 0x60120, 0x60122, 0x60124, 0x60131];
const UNION_APPLICATION_MESSAGE_TYPE: i32 = 0x60118;
const ENABLE_LEAVE_WORD_MESSAGE_TYPE: i32 = 0x6011A;
const LEAVE_WORD_MESSAGE_TYPE: i32 = 0x6011B;
const EDIT_LEAVE_WORD_MESSAGE_TYPE: i32 = 0x6011C;
const PRONOUNCE_MESSAGE_TYPE: i32 = 0x6011D;
const DECLARE_WAR_FACTION_LIST_MESSAGE_TYPE: i32 = 0x6011E;
const DECLARE_WAR_FACTION_LIST_RESPONSE_TYPE: i32 = 0x7FE18;
const DECLARE_FACTION_WAR_MESSAGE_TYPE: i32 = 0x6011F;
const DECLARE_FACTION_WAR_RESPONSE_TYPE: i32 = 0x7FE19;
const CONSUMED_LONG_MESSAGE_TYPES: [i32; 2] = [0x60121, 0x60123];
const FACTION_BILLBOARD_MESSAGE_TYPE: i32 = 0x60125;
const FACTION_BILLBOARD_RESPONSE_TYPE: i32 = 0x7FE1D;
const UPGRADE_FACTION_MESSAGE_TYPE: i32 = 0x60126;
const UPLOAD_FACTION_ICON_MESSAGE_TYPE: i32 = 0x60127;
const SET_FACTION_CONTRIBUTOR_MESSAGE_TYPE: i32 = 0x60128;
const ADD_FACTION_EXPERIENCE_MESSAGE_TYPE: i32 = 0x60129;
const LEAVE_WORD_INPUT_CAPACITY: usize = 0xD2;
const PRONOUNCE_INPUT_CAPACITY: usize = 0x5000;

static FACTION_BILLBOARD_TITLES: OnceLock<[Vec<u8>; 3]> = OnceLock::new();

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
    pub(crate) faction_level_log_enabled: bool,
    pub(crate) write_faction_level_log:
        &'a mut dyn FnMut(i32, &[u8], i32, i32, &[u8]),
    pub(crate) faction_experience_log_enabled: bool,
    pub(crate) write_faction_experience_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32, i32),
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

/// Concrete player/goods/string/log adapter для готового `CFaction::Upgrade`.
struct WorldFactionUpgradeEffects<'game, 'callbacks, 'effects, 'update> {
    game: &'game CGame,
    registry: &'game GoodsBasePropertiesRegistry,
    original_name_index: &'game GoodsOriginalNameIndex,
    use_log_system: bool,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
}

impl FactionOrganizingInfoContext for WorldFactionUpgradeEffects<'_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some((self.callbacks.world_string)(string_id))
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionLevelContext for WorldFactionUpgradeEffects<'_, '_, '_, '_> {
    fn format_world_string_signed(
        &mut self,
        string_id: &'static [u8],
        value: i32,
    ) -> Vec<u8> {
        (self.callbacks.format_world_string)(
            string_id,
            &[UnionFormatArgument::Signed(value)],
        )
    }
}

impl FactionUpgradeContext for WorldFactionUpgradeEffects<'_, '_, '_, '_> {
    fn player_money(&self, player_id: i32) -> Option<u32> {
        self.game
            .online_player_by_id(player_id as u32)
            .map(|player| player.money())
    }

    fn goods_in_packet(&self, player_id: i32, original_name: &[u8]) -> i32 {
        let Ok(original_name) = CString::new(original_name) else {
            return 0;
        };
        self.game
            .online_player_by_id(player_id as u32)
            .map_or(0, |player| {
                player.check_goods_in_packet(
                    Some(original_name.as_c_str()),
                    self.original_name_index,
                )
            })
    }

    fn goods_display_name(&self, original_name: &[u8]) -> Option<Vec<u8>> {
        let goods_id = self.original_name_index.get(original_name).copied()?;
        query_goods_name(self.registry, goods_id).map(ToOwned::to_owned)
    }

    fn format_upgrade_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionUpgradeFormatArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                FactionUpgradeFormatArgument::Text(value) => UnionFormatArgument::Text(value),
                FactionUpgradeFormatArgument::Signed(value) => {
                    UnionFormatArgument::Signed(*value)
                }
            })
            .collect::<Vec<_>>();
        (self.callbacks.format_world_string)(string_id, &arguments)
    }

    fn update_player_faction_info(&mut self, player_id: i32) {
        (self.update_player)(player_id);
    }

    fn faction_level_log_enabled(&self) -> bool {
        self.use_log_system && self.callbacks.faction_level_log_enabled
    }

    fn write_faction_level_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        level: i32,
        master_id: i32,
        master_name: &[u8],
    ) {
        (self.callbacks.write_faction_level_log)(
            faction_id,
            faction_name,
            level,
            master_id,
            master_name,
        );
    }
}

/// Узкий string/player adapter для готового `CFaction::UploadIcon`.
struct WorldFactionUploadIconEffects<'game, 'callbacks, 'effects> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
}

impl FactionOrganizingInfoContext for WorldFactionUploadIconEffects<'_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some((self.callbacks.world_string)(string_id))
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionUploadIconContext for WorldFactionUploadIconEffects<'_, '_, '_> {
    fn format_upload_icon_interval(
        &mut self,
        string_id: &'static [u8],
        interval_minutes: i32,
    ) -> Vec<u8> {
        (self.callbacks.format_world_string)(
            string_id,
            &[UnionFormatArgument::Signed(interval_minutes)],
        )
    }
}

/// Узкий string/player adapter для готового `CFaction::SetControbuter`.
struct WorldFactionContributorEffects<'game, 'callbacks, 'effects, 'update> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
}

impl FactionOrganizingInfoContext for WorldFactionContributorEffects<'_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some((self.callbacks.world_string)(string_id))
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionContributorContext for WorldFactionContributorEffects<'_, '_, '_, '_> {
    fn format_contributor_string(
        &mut self,
        string_id: &'static [u8],
        member_name: &[u8],
    ) -> Vec<u8> {
        (self.callbacks.format_world_string)(
            string_id,
            &[UnionFormatArgument::Text(member_name)],
        )
    }

    fn update_player_faction_info(&mut self, player_id: i32) {
        (self.update_player)(player_id);
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

fn capture_local_tag_time() -> TagTimeValue {
    let local_time = TagTime::local_now();
    TagTimeValue {
        year: local_time.year,
        month: local_time.month,
        day_of_week: local_time.day_of_week,
        day: local_time.day,
        hour: local_time.hour,
        minute: local_time.minute,
        second: local_time.second,
        milliseconds: local_time.milliseconds,
    }
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
    let time = capture_local_tag_time();
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

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingLeaveWordEditDispatch {
    pub(crate) leave_word_id: i32,
    pub(crate) player_id: i32,
    pub(crate) outcome: OrganizingLeaveWordEditOutcome,
}

/// Выполняет `0x6011C`: player membership и `EditLeaveWord(..., Delete)`.
pub(crate) fn dispatch_leave_word_edit(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
) -> Option<Result<OrganizingLeaveWordEditDispatch, OrganizingLeaveWordEditBlock>> {
    if message.message_type() != EDIT_LEAVE_WORD_MESSAGE_TYPE {
        return None;
    }

    let leave_word_id = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    Some(
        organizing
            .edit_leave_word_for_player(game, player_id, leave_word_id, EOperator::Delete)
            .map(|outcome| OrganizingLeaveWordEditDispatch {
                leave_word_id,
                player_id,
                outcome,
            }),
    )
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingPronounceDispatch {
    pub(crate) player_id: i32,
    pub(crate) content: Vec<u8>,
    pub(crate) time: TagTimeValue,
    pub(crate) outcome: OrganizingPronounceOutcome,
}

/// Выполняет `0x6011D`: bounded C-string, player membership и `Pronounce`.
pub(crate) fn dispatch_pronounce(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
) -> Option<Result<OrganizingPronounceDispatch, OrganizingPronounceBlock>> {
    if message.message_type() != PRONOUNCE_MESSAGE_TYPE {
        return None;
    }

    let mut content = message
        .base_mut()
        .get_str_bytes(PRONOUNCE_INPUT_CAPACITY)
        .unwrap_or_default();
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let time = capture_local_tag_time();
    Some(
        organizing
            .pronounce_for_player(game, player_id, &mut content, time)
            .map(|outcome| OrganizingPronounceDispatch {
                player_id,
                content,
                time,
                outcome,
            }),
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingConsumedLongDispatch {
    pub(crate) message_type: i32,
    pub(crate) value: i32,
}

/// Выполняет общий leaf `0x60121/0x60123`: читает и отбрасывает один `Long`.
pub(crate) fn dispatch_consumed_long(
    message: &mut CMessage,
) -> Option<OrganizingConsumedLongDispatch> {
    let message_type = message.message_type();
    if !CONSUMED_LONG_MESSAGE_TYPES.contains(&message_type) {
        return None;
    }
    let value = message.base_mut().get_long().unwrap_or(0);
    Some(OrganizingConsumedLongDispatch {
        message_type,
        value,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDeclareWarFactionListNotice {
    MissingFaction,
    MasterRequired,
    NoFactions,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingDeclareWarFactionListResponse {
    pub(crate) socket_id: i32,
    pub(crate) total_factions: i32,
    pub(crate) included_page: Option<i32>,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDeclareWarFactionListOutcome {
    Empty {
        faction_id: i32,
        notice: OrganizingDeclareWarFactionListNotice,
        response: OrganizingDeclareWarFactionListResponse,
    },
    PageOutsideRange {
        faction_id: i32,
        total_factions: i32,
        page: i32,
    },
    Page {
        faction_id: i32,
        page: DeclareWarFactionPage,
        response: OrganizingDeclareWarFactionListResponse,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDeclareWarFactionListBlock {
    Membership { map_key: i32 },
    Page(DeclareWarFactionPageBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingDeclareWarFactionListDispatch {
    pub(crate) request_id: i64,
    pub(crate) cookie: i32,
    pub(crate) player_id: i32,
    pub(crate) page: i32,
    pub(crate) outcome: OrganizingDeclareWarFactionListOutcome,
}

/// Выполняет `0x6011E` и строит точный socket-response `0x7FE18`.
pub(crate) fn dispatch_declare_war_faction_list<Context>(
    message: &mut CMessage,
    organizing: &COrganizingCtrl,
    faction_wars: &CFactionWarSys,
    context: &mut Context,
    sender: Option<&ServerCommandHandle>,
) -> Option<
    Result<OrganizingDeclareWarFactionListDispatch, OrganizingDeclareWarFactionListBlock>,
>
where
    Context: FactionOrganizingInfoContext,
{
    if message.message_type() != DECLARE_WAR_FACTION_LIST_MESSAGE_TYPE {
        return None;
    }

    let socket_id = message.socket_id();
    let request_id = message.base_mut().get_long64().unwrap_or(0);
    let cookie = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let page = message.base_mut().get_long().unwrap_or(0);
    let faction_id = match organizing.is_free_player(player_id) {
        FreePlayerLookup::NoFaction => 0,
        FreePlayerLookup::Faction(faction_id) => faction_id,
        FreePlayerLookup::BlockedNullFaction { map_key } => {
            return Some(Err(OrganizingDeclareWarFactionListBlock::Membership { map_key }));
        }
    };

    let outcome = if organizing.faction_by_id(faction_id).is_none() {
        send_declare_war_faction_list_notice(
            context,
            player_id,
            OrganizingDeclareWarFactionListNotice::MissingFaction,
        );
        OrganizingDeclareWarFactionListOutcome::Empty {
            faction_id,
            notice: OrganizingDeclareWarFactionListNotice::MissingFaction,
            response: send_declare_war_faction_list_response(
                sender, socket_id, player_id, 0, request_id, cookie, None,
            ),
        }
    } else if organizing
        .faction_by_id(faction_id)
        .is_some_and(|faction| faction.is_master(player_id) == 0)
    {
        send_declare_war_faction_list_notice(
            context,
            player_id,
            OrganizingDeclareWarFactionListNotice::MasterRequired,
        );
        OrganizingDeclareWarFactionListOutcome::Empty {
            faction_id,
            notice: OrganizingDeclareWarFactionListNotice::MasterRequired,
            response: send_declare_war_faction_list_response(
                sender, socket_id, player_id, 0, request_id, cookie, None,
            ),
        }
    } else {
        let total_factions = match organizing.declare_war_faction_count() {
            Ok(total_factions) => total_factions,
            Err(block) => {
                return Some(Err(OrganizingDeclareWarFactionListBlock::Page(block)));
            }
        };
        if total_factions == 0 {
            send_declare_war_faction_list_notice(
                context,
                player_id,
                OrganizingDeclareWarFactionListNotice::NoFactions,
            );
            OrganizingDeclareWarFactionListOutcome::Empty {
                faction_id,
                notice: OrganizingDeclareWarFactionListNotice::NoFactions,
                response: send_declare_war_faction_list_response(
                    sender, socket_id, player_id, 0, request_id, cookie, None,
                ),
            }
        } else {
            let start = page.wrapping_mul(11).wrapping_sub(11);
            if start >= total_factions {
                OrganizingDeclareWarFactionListOutcome::PageOutsideRange {
                    faction_id,
                    total_factions,
                    page,
                }
            } else {
                let page_data = match organizing.declare_war_faction_page(
                    faction_id,
                    page,
                    faction_wars,
                ) {
                    Ok(page_data) => page_data,
                    Err(block) => {
                        return Some(Err(OrganizingDeclareWarFactionListBlock::Page(block)));
                    }
                };
                let response = send_declare_war_faction_list_response(
                    sender,
                    socket_id,
                    player_id,
                    total_factions,
                    request_id,
                    cookie,
                    Some(&page_data),
                );
                OrganizingDeclareWarFactionListOutcome::Page {
                    faction_id,
                    page: page_data,
                    response,
                }
            }
        }
    };

    Some(Ok(OrganizingDeclareWarFactionListDispatch {
        request_id,
        cookie,
        player_id,
        page,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingDeclareFactionWarResponse {
    pub(crate) socket_id: i32,
    pub(crate) result_money: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDeclareFactionWarOutcome {
    PlayerOffline,
    Declaration(FactionWarDeclarationOutcome),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDeclareFactionWarBlock {
    PlayerDecode(PlayerCodecError),
    Declaration(FactionWarDeclarationBlock<OrganizingFactionWarDeclarationBlock>),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingDeclareFactionWarDispatch {
    pub(crate) request_id: i64,
    pub(crate) cookie: i32,
    pub(crate) player_id: i32,
    pub(crate) target_faction_id: i32,
    pub(crate) war_type: i32,
    pub(crate) outcome: OrganizingDeclareFactionWarOutcome,
    pub(crate) response: OrganizingDeclareFactionWarResponse,
}

/// Выполняет `0x6011F`: player decode, объявление войны и socket-response `0x7FE19`.
pub(crate) fn dispatch_declare_faction_war(
    message: &mut CMessage,
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    faction_wars: &mut CFactionWarSys,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
    sender: Option<&ServerCommandHandle>,
) -> Option<Result<OrganizingDeclareFactionWarDispatch, OrganizingDeclareFactionWarBlock>> {
    if message.message_type() != DECLARE_FACTION_WAR_MESSAGE_TYPE {
        return None;
    }

    let socket_id = message.socket_id();
    let request_id = message.base_mut().get_long64().unwrap_or(0);
    let cookie = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let target_faction_id = message.base_mut().get_long().unwrap_or(0);
    let war_type = message.base_mut().get_long().unwrap_or(0);
    let player_online = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        match game.decord_online_player_by_id(
            player_id as u32,
            source,
            cursor,
            registry,
            coefficients,
        ) {
            Ok(player_online) => player_online,
            Err(source) => {
                return Some(Err(OrganizingDeclareFactionWarBlock::PlayerDecode(source)));
            }
        }
    };

    let (outcome, result_money) = if player_online {
        let declaration_time = TagTime::local_now();
        let mut effects = WorldFactionWarDeclarationEffects::new(
            game,
            organizing,
            &mut *callbacks.world_string,
            &mut *callbacks.format_world_string,
            &mut *callbacks.put_war_log,
            update_player,
        );
        let declaration = match faction_wars.dig_up_the_hatchet(
            player_id,
            target_faction_id,
            war_type,
            declaration_time,
            &mut effects,
        ) {
            Ok(declaration) => declaration,
            Err(source) => {
                return Some(Err(OrganizingDeclareFactionWarBlock::Declaration(source)));
            }
        };
        let result_money = if declaration.legacy_result() {
            faction_wars.get_dec_war_money_by_type(war_type)
        } else {
            0
        };
        (
            OrganizingDeclareFactionWarOutcome::Declaration(declaration),
            result_money,
        )
    } else {
        (OrganizingDeclareFactionWarOutcome::PlayerOffline, 0)
    };

    let mut response = CMessage::new(DECLARE_FACTION_WAR_RESPONSE_TYPE);
    response.base_mut().add_long64(request_id);
    response.base_mut().add_long(cookie);
    response.base_mut().add_long(player_id);
    response.base_mut().add_long(result_money);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send_to_socket(sender, socket_id);

    Some(Ok(OrganizingDeclareFactionWarDispatch {
        request_id,
        cookie,
        player_id,
        target_faction_id,
        war_type,
        outcome,
        response: OrganizingDeclareFactionWarResponse {
            socket_id,
            result_money,
            wire,
            delivery,
        },
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionBillboardResponse {
    pub(crate) socket_id: i32,
    pub(crate) title: Vec<u8>,
    pub(crate) payload: Vec<u8>,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionBillboardOutcome {
    TypeAboveRange {
        request_id: i32,
        billboard_type: i32,
    },
    Sent {
        request_id: i32,
        billboard_type: i32,
        response: OrganizingFactionBillboardResponse,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionBillboardBlock {
    pub(crate) request_id: i32,
    pub(crate) billboard_type: i32,
}

/// Выполняет `0x60125` и сохраняет несовпадающую нумерацию request/serializer.
pub(crate) fn dispatch_faction_billboard(
    message: &mut CMessage,
    organizing: &COrganizingCtrl,
    world_string: &mut dyn FnMut(&[u8]) -> Vec<u8>,
    sender: Option<&ServerCommandHandle>,
) -> Option<Result<OrganizingFactionBillboardOutcome, OrganizingFactionBillboardBlock>> {
    if message.message_type() != FACTION_BILLBOARD_MESSAGE_TYPE {
        return None;
    }

    // Три `GetStringByID` исходник выполнял при первом входе в case, ещё до
    // чтения request и проверки типа, после чего process-static строки уже не
    // реагировали на reload string table.
    let titles = FACTION_BILLBOARD_TITLES.get_or_init(|| {
        [
            legacy_c_string_prefix(&world_string(b"WS0134")).to_vec(),
            legacy_c_string_prefix(&world_string(b"WS0133")).to_vec(),
            legacy_c_string_prefix(&world_string(b"WS0132")).to_vec(),
        ]
    });
    let socket_id = message.socket_id();
    let request_id = message.base_mut().get_long().unwrap_or(0);
    let billboard_type = message.base_mut().get_long().unwrap_or(0);
    if billboard_type > 2 {
        return Some(Ok(OrganizingFactionBillboardOutcome::TypeAboveRange {
            request_id,
            billboard_type,
        }));
    }
    let Ok(title_index) = usize::try_from(billboard_type) else {
        // BLOCKED_MISSING_FACT: EXE проверяет только `2 < type`, после чего
        // отрицательный type индексирует process-static `std::string[3]` до
        // массива. Результат такого out-of-bounds чтения не имитируется.
        return Some(Err(OrganizingFactionBillboardBlock {
            request_id,
            billboard_type,
        }));
    };

    let title = titles[title_index].clone();
    let mut payload = Vec::new();
    organizing.add_faction_billboard_to_byte_array(&mut payload, billboard_type);

    let mut response = CMessage::new(FACTION_BILLBOARD_RESPONSE_TYPE);
    response.base_mut().add_long(request_id);
    response.base_mut().add(&title);
    response.base_mut().add_byte(0);
    response.base_mut().add(&payload);
    response.base_mut().update();
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send_to_socket(sender, socket_id);

    Some(Ok(OrganizingFactionBillboardOutcome::Sent {
        request_id,
        billboard_type,
        response: OrganizingFactionBillboardResponse {
            socket_id,
            title,
            payload,
            wire,
            delivery,
        },
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionUpgradeOutcome {
    PlayerOffline,
    FactionMissing,
    Upgrade(FactionUpgradeOutcome),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionUpgradeBlock {
    PlayerDecode(PlayerCodecError),
    Upgrade(FactionUpgradeBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionUpgradeDispatch {
    pub(crate) faction_id: i32,
    pub(crate) player_id: i32,
    pub(crate) outcome: OrganizingFactionUpgradeOutcome,
}

/// Выполняет `0x60126`: live player snapshot и virtual `CFaction::Upgrade`.
#[allow(
    clippy::too_many_arguments,
    reason = "opcode использует прежние game/faction/goods/string/log singleton-ы"
)]
pub(crate) fn dispatch_faction_upgrade(
    message: &mut CMessage,
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    registry: &GoodsBasePropertiesRegistry,
    original_name_index: &GoodsOriginalNameIndex,
    coefficients: &PlayerPropertyCoefficients,
    use_log_system: bool,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionUpgradeDispatch, OrganizingFactionUpgradeBlock>> {
    if message.message_type() != UPGRADE_FACTION_MESSAGE_TYPE {
        return None;
    }

    let faction_id = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let player_online = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        match game.decord_online_player_by_id(
            player_id as u32,
            source,
            cursor,
            registry,
            coefficients,
        ) {
            Ok(player_online) => player_online,
            Err(source) => {
                return Some(Err(OrganizingFactionUpgradeBlock::PlayerDecode(source)));
            }
        }
    };
    if !player_online {
        return Some(Ok(OrganizingFactionUpgradeDispatch {
            faction_id,
            player_id,
            outcome: OrganizingFactionUpgradeOutcome::PlayerOffline,
        }));
    }

    let game_ref: &CGame = game;
    let mut effects = WorldFactionUpgradeEffects {
        game: game_ref,
        registry,
        original_name_index,
        use_log_system,
        callbacks,
        update_player,
    };
    let outcome = match organizing.upgrade_faction(
        game_ref,
        parameters,
        faction_id,
        player_id,
        &mut effects,
    ) {
        Ok(Some(outcome)) => OrganizingFactionUpgradeOutcome::Upgrade(outcome),
        Ok(None) => OrganizingFactionUpgradeOutcome::FactionMissing,
        Err(source) => return Some(Err(OrganizingFactionUpgradeBlock::Upgrade(source))),
    };

    Some(Ok(OrganizingFactionUpgradeDispatch {
        faction_id,
        player_id,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionUploadIconOutcome {
    FactionMissing,
    UploadIcon {
        time: TagTimeValue,
        outcome: FactionUploadIconOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionUploadIconDispatch {
    pub(crate) faction_id: i32,
    pub(crate) player_id: i32,
    pub(crate) outcome: OrganizingFactionUploadIconOutcome,
}

/// Выполняет `0x60127`: два faction lookup и local-time перед `UploadIcon`.
pub(crate) fn dispatch_faction_upload_icon(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
) -> Option<Result<OrganizingFactionUploadIconDispatch, FactionUploadIconBlock>> {
    if message.message_type() != UPLOAD_FACTION_ICON_MESSAGE_TYPE {
        return None;
    }

    let faction_id = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    if organizing.faction_by_id(faction_id).is_none() {
        return Some(Ok(OrganizingFactionUploadIconDispatch {
            faction_id,
            player_id,
            outcome: OrganizingFactionUploadIconOutcome::FactionMissing,
        }));
    }

    let time = capture_local_tag_time();
    let mut effects = WorldFactionUploadIconEffects { game, callbacks };
    let outcome = match organizing.upload_faction_icon(
        parameters,
        faction_id,
        player_id,
        &time,
        &mut effects,
    ) {
        Ok(Some(outcome)) => OrganizingFactionUploadIconOutcome::UploadIcon { time, outcome },
        Ok(None) => OrganizingFactionUploadIconOutcome::FactionMissing,
        Err(source) => return Some(Err(source)),
    };

    Some(Ok(OrganizingFactionUploadIconDispatch {
        faction_id,
        player_id,
        outcome,
    }))
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionContributorDispatch {
    pub(crate) target_player_id: i32,
    pub(crate) enabled_value: i32,
    pub(crate) requester_player_id: i32,
    pub(crate) outcome: OrganizingContributorOutcome,
}

/// Выполняет `0x60128`: requester membership и virtual `SetControbuter`.
pub(crate) fn dispatch_faction_contributor(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionContributorDispatch, OrganizingContributorBlock>> {
    if message.message_type() != SET_FACTION_CONTRIBUTOR_MESSAGE_TYPE {
        return None;
    }

    let target_player_id = message.base_mut().get_long().unwrap_or(0);
    let enabled_value = message.base_mut().get_long().unwrap_or(0);
    let requester_player_id = message.base_mut().get_long().unwrap_or(0);
    let mut effects = WorldFactionContributorEffects {
        game,
        callbacks,
        update_player,
    };
    Some(
        organizing
            .set_contributor_for_player(
                game,
                parameters,
                requester_player_id,
                target_player_id,
                enabled_value != 0,
                &mut effects,
            )
            .map(|outcome| OrganizingFactionContributorDispatch {
                target_player_id,
                enabled_value,
                requester_player_id,
                outcome,
            }),
    )
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionExperienceOutcome {
    FactionMissing,
    PlayerNotContributor,
    Applied {
        before_experience: i32,
        update: FactionExperienceUpdate,
        log_written: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingFactionExperienceDispatch {
    pub(crate) faction_id: i32,
    pub(crate) player_id: i32,
    pub(crate) experience_delta: i32,
    pub(crate) outcome: OrganizingFactionExperienceOutcome,
}

/// Выполняет `0x60129`: contributor gate, wrapping delta и optional DB-log.
pub(crate) fn dispatch_faction_experience(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    use_log_system: bool,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
) -> Option<Result<OrganizingFactionExperienceDispatch, FactionExperienceBlock>> {
    if message.message_type() != ADD_FACTION_EXPERIENCE_MESSAGE_TYPE {
        return None;
    }

    let faction_id = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let experience_delta = message.base_mut().get_long().unwrap_or(0);
    let mutation = match organizing.add_contributor_experience(
        game,
        faction_id,
        player_id,
        experience_delta,
    ) {
        Ok(mutation) => mutation,
        Err(source) => return Some(Err(source)),
    };
    let (actual_faction_id, faction_name, before_experience, update) = match mutation {
        OrganizingFactionExperienceMutation::FactionNotFound => {
            return Some(Ok(OrganizingFactionExperienceDispatch {
                faction_id,
                player_id,
                experience_delta,
                outcome: OrganizingFactionExperienceOutcome::FactionMissing,
            }));
        }
        OrganizingFactionExperienceMutation::PlayerNotContributor => {
            return Some(Ok(OrganizingFactionExperienceDispatch {
                faction_id,
                player_id,
                experience_delta,
                outcome: OrganizingFactionExperienceOutcome::PlayerNotContributor,
            }));
        }
        OrganizingFactionExperienceMutation::Applied {
            faction_id,
            faction_name,
            before_experience,
            update,
        } => (faction_id, faction_name, before_experience, update),
    };

    let mut log_written = false;
    if use_log_system && callbacks.faction_experience_log_enabled {
        if let Some(player) = game.online_player_by_id(player_id as u32) {
            (callbacks.write_faction_experience_log)(
                actual_faction_id,
                &faction_name,
                player.get_id(),
                player.get_name(),
                before_experience,
                experience_delta,
            );
            log_written = true;
        }
    }

    Some(Ok(OrganizingFactionExperienceDispatch {
        faction_id,
        player_id,
        experience_delta,
        outcome: OrganizingFactionExperienceOutcome::Applied {
            before_experience,
            update,
            log_written,
        },
    }))
}

fn send_declare_war_faction_list_notice<Context>(
    context: &mut Context,
    player_id: i32,
    notice: OrganizingDeclareWarFactionListNotice,
) where
    Context: FactionOrganizingInfoContext,
{
    let first_string_id = match notice {
        OrganizingDeclareWarFactionListNotice::MissingFaction => b"WS0122".as_slice(),
        OrganizingDeclareWarFactionListNotice::MasterRequired => b"WS0123".as_slice(),
        OrganizingDeclareWarFactionListNotice::NoFactions => b"WS0124".as_slice(),
    };
    let first_text = context.world_string(first_string_id).unwrap_or_default();
    let second_text = context.world_string(b"WS0121").unwrap_or_default();
    context.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id: player_id,
        first_text: legacy_c_string_prefix(&first_text),
        second_text: legacy_c_string_prefix(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

fn send_declare_war_faction_list_response(
    sender: Option<&ServerCommandHandle>,
    socket_id: i32,
    player_id: i32,
    total_factions: i32,
    request_id: i64,
    cookie: i32,
    page: Option<&DeclareWarFactionPage>,
) -> OrganizingDeclareWarFactionListResponse {
    let mut response = CMessage::new(DECLARE_WAR_FACTION_LIST_RESPONSE_TYPE);
    response.base_mut().add_long(player_id);
    response.base_mut().add_long(total_factions);
    response.base_mut().add_long64(request_id);
    response.base_mut().add_long(cookie);
    if let Some(page) = page {
        response.base_mut().add_long(page.requested_page);
        response.base_mut().add(&page.payload);
    }
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send_to_socket(sender, socket_id);
    OrganizingDeclareWarFactionListResponse {
        socket_id,
        total_factions,
        included_page: page.map(|page| page.requested_page),
        wire,
        delivery,
    }
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
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
