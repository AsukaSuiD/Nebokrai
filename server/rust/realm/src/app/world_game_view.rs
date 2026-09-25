//! Узкий game-view для обработчиков мировых сообщений Realm.

use std::future::Future;
use std::pin::Pin;

use nebokrai_shared::network::ServerCommandHandle;
use nebokrai_shared::resources::{CGodsBattleConf, GlobeSetupSnapshot};

use crate::activities::jjcsystem::CJJcSystem;
use crate::activities::rsgodsbattle::TiberiusRsGodsBattle;
use crate::app::auction::WorldBaiTanRemoval;
use crate::app::gmmessage::{WorldNamedRegionLookup, WorldRegionIdRouteScan};
use crate::app::world_client::CMyNetClient;
use crate::app::world_message::{CMessage, SendMessageError, WorldLocalMessageQueueBlock};
use crate::app::worldserver::{WorldReloadContext, WorldReloadResult};
use crate::app::worldothermessage::{
    WorldGoodsLink, WorldHonorEliminatorRegistration, WorldPlayerNameChangeReport,
    WorldPlayerNameLookupError,
};
use crate::persistence::rssetup::WorldTdsClient;
use crate::persistence::writelog::WorldWriteLogCommand;
use crate::characters::player::{
    CPlayer, PlayerCodecError, PlayerFactionInfoUpdateBlock, PlayerFactionInfoUpdateReport,
    PlayerPropertyCoefficients,
};
use crate::organizations::faction::CFaction;
use crate::organizations::union::UnionFormatArgument;
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::skillfactory::CSkillFactory;
use crate::sessions::csessionfactory::CSessionFactory;

/// Трёхсторонний поиск имени региона: отсутствующий ключ карты, null-ячейка
/// owner-а и найденное имя. Свёртка в `Option` потеряла бы отличие
/// `NullRegionPointer` (Blocked ветки) от `RegionNotFound` (пустое имя).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldRegionNameLookup<'a> {
    RegionNotFound,
    NullRegionPointer,
    Name(&'a [u8]),
}

/// Snapshot полей login-строки игрока: `_strcmpi` совпадение долгожителям ещё до
/// поставки `worldserver.exe` строки, legacy-порядок map-итерации сохранён.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldLoginAccountPlayer {
    pub team_id: i32,
    pub owner_type: i32,
    pub owner_id: i32,
}

/// Онлайн account маршрут с optional team-form fields; map-owner исходящие
/// states добавляются записями той же backend формы.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldOnlineAccountPlayerRoute {
    pub team_id: i32,
    pub owner_type: i32,
    pub owner_id: i32,
    pub game_server_index: u32,
}

/// Снимок состояния game server-а: наличие коннекта и числовой индекс маршрута.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldGameServerSnapshot {
    pub connected: bool,
    pub index: u32,
}

/// Результат `CGame::exit_team_player`: три исхода проверки внешней
/// session/team-реквизитация plug identifiers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldLoginTimeoutTeamExit {
    SessionMissingOrNotTeam,
    PlugMissing,
    Exited,
}

/// Точки обратного вызова handler-ветвей мира в владельца игры. Реализация
/// живёт у владельца игры (старый `CGame`) и делегирует его inherent-
/// методам; имена намеренно совпадают — inherent priority исключает рекурсию.
pub trait WorldGameView {
    fn current_game_server_sender(&self) -> Option<ServerCommandHandle>;

    fn send_msg_to_game_server(
        &self,
        map_id: i32,
        message: &CMessage,
    ) -> Result<i32, SendMessageError>;

    fn publish_team_session(&mut self, team_id: u32, session_id: i32);

    fn remove_team_session(&mut self, team_id: u32);

    fn get_team_session_id(&self, team_id: u32) -> i32;

    fn player_game_server(&self, player_id: i32) -> Option<WorldGameServerSnapshot>;

    /// Индекс маршрута game server-а региона; `None` — региона нет в карте.
    /// `None` отличим от legacy `0`: ветка private-chat выбирает по нему
    /// `PrivateUnavailable`.
    fn region_game_server_index(&self, region_id: i32) -> Option<u32>;

    fn map_player(&self, player_id: u32) -> Option<&CPlayer>;

    fn online_player_id_by_name(&self, name: &[u8]) -> u32;

    fn online_player_by_cdkey(&self, cdkey: &[u8]) -> Option<&CPlayer>;

    fn configured_world_number(&self) -> Option<u32>;

    fn current_login_client(&self) -> Option<&CMyNetClient>;

    fn game_server_number_by_player_id(&self, player_id: i32) -> i32;

    fn legacy_tick_ms(&self) -> u32;

    fn set_map_player_jjc_identity(&mut self, player_id: u32, level: u8, jjc_level: u32) -> bool;

    fn set_map_player_jjc_snapshot(
        &mut self,
        player_id: u32,
        level: u8,
        jjc_level: u32,
        jjc_score: u32,
        counters: [u8; 0x10],
    ) -> bool;

    fn online_player_by_id(&self, player_id: u32) -> Option<&CPlayer>;

    fn decord_online_player_by_id(
        &mut self,
        player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<bool, PlayerCodecError>;

    fn decode_online_player_lei_ting(
        &mut self,
        player_id: u32,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<bool, PlayerCodecError>;

    fn reset_honor_eliminate_info(&mut self, rank_mask: u32) -> bool;

    fn register_honor_eliminator(
        &mut self,
        player_id: u32,
        eliminator_id: u32,
    ) -> WorldHonorEliminatorRegistration;

    fn add_goods_link(&mut self, link: WorldGoodsLink) -> u32;

    fn find_goods_link(&self, index: u32) -> Option<&WorldGoodsLink>;

    fn add_item_to_bai_tan_request_list(&mut self, ip: u32, player_id: i32) -> bool;

    fn del_item_from_bai_tan_list(&mut self, player_id: i32) -> WorldBaiTanRemoval;

    fn game_server(&self, index: u32) -> Option<WorldGameServerSnapshot>;

    fn push_write_log_command(&self, command: WorldWriteLogCommand) -> usize;

    fn get_string_by_id(&self, string_id: &[u8]) -> &[u8];

    fn map_player_id_by_name(&self, name: &[u8]) -> u32;

    fn named_region_lookup(&self, name: &[u8]) -> WorldNamedRegionLookup;

    fn region_routes_by_owner_id(&self, region_id: i32) -> WorldRegionIdRouteScan;

    fn region_name(&self, region_id: i32) -> WorldRegionNameLookup<'_>;

    fn queue_local_world_message(&self, message: CMessage) -> Result<(), WorldLocalMessageQueueBlock>;

    fn allocate_leave_word_id(&mut self) -> i32;

    /// Обновляет проекцию faction-информации игрока по переданной фракции.
    fn update_player_faction_info_from_faction(
        &self,
        faction: &CFaction,
        player_id: i32,
    ) -> Result<Option<PlayerFactionInfoUpdateReport>, PlayerFactionInfoUpdateBlock>;

    fn online_player_count(&self) -> usize;

    /// In-memory списки `restore_players`/`deletion_players` при restore-role
    /// ходе: заявленные взаимные семантики delete/append сохраняют прежний
    /// порядок и повторный gate; id хранится как u32 на время заявки.
    fn is_restore_player_exist(&self, player_id: u32) -> bool;

    fn delete_restore_player(&mut self, player_id: u32);

    fn deletion_player_time(&self, player_id: u32) -> i32;

    fn delete_deletion_player(&mut self, player_id: u32);

    fn append_restore_player(&mut self, player_id: u32);

    fn append_deletion_player(&mut self, player_id: u32, deletion_time: i32);

    /// Списки login/online/offline при cleanup/disconnect: login-строка
    /// маршрута сохранён как legacy-lookup фиксну БД адресной очередью;
    /// список login и player_load-очередь у старого владельца, взамен login
    /// или SB account offline записей;
    /// team session exit — те же session-factory что и owner world queue.
    fn login_player_by_account(&self, account: &[u8]) -> Option<WorldLoginAccountPlayer>;

    fn remove_login_player(&mut self, player_id: u32) -> bool;

    fn remove_player_load_data(&self, player_id: i32) -> bool;

    fn append_offline_player_id(&mut self, player_id: u32) -> bool;

    fn online_player_route_by_account(&self, account: &[u8]) -> Option<WorldOnlineAccountPlayerRoute>;

    /// Проверки обработчика `create_role`: занятость имени в live-карте и в
    /// двух слоях загруженных DB-данных. Реализации делегируют одноимённым
    /// inherent-методам владельца игры; порядок вызовов остаётся у обработчика.
    fn is_name_exist_in_map_player(&self, name: &[u8]) -> Result<bool, WorldPlayerNameLookupError>;

    fn is_name_exist_in_db_creation(&self, name: &[u8]) -> Result<bool, WorldPlayerNameLookupError>;

    fn is_name_exist_in_db_data(&self, name: &[u8]) -> Result<bool, WorldPlayerNameLookupError>;

    /// Форматирование строки live StringTable владельца игры: реализация
    /// делегирует inherent `CGame::format_world_string` с тем списком
    /// аргументов, который формирует вызывающая сторона шва.
    fn format_world_string(&self, string_id: &[u8], arguments: &[UnionFormatArgument<'_>]) -> Vec<u8>;

    fn check_create_role_name(
        &self,
        name: &mut Vec<u8>,
        allow_short: bool,
        apply_filter: bool,
    ) -> bool;

    fn allocate_player_id(&mut self) -> i32;

    /// Занятость имени игроком в стадии создания: свёртка `Option<&CPlayer>`
    /// inherent-результата в предикат, как делает сам обработчик.
    fn creation_player_by_name(
        &self,
        name: &[u8],
    ) -> Result<bool, WorldPlayerNameLookupError>;

    fn exit_team_player(
        &mut self,
        factory: &mut CSessionFactory,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
    ) -> WorldLoginTimeoutTeamExit;

    fn replace_online_player_silience_time(&mut self, player_id: u32, silience_time: i32) -> Option<i32>;

    /// Единственный владелец перезагрузки мира: реализация делегирует
    /// inherent `CGame::reload` старого пакета. Boxed future, а не `async fn`,
    /// сохраняет объектную безопасность: view потребляется как
    /// `&mut dyn WorldGameView` сессиями и GM-диспетчером.
    #[allow(clippy::type_complexity, reason = "точный вызов inherent reload с теми же аргументами")]
    fn reload<'a>(
        &'a mut self,
        context: &'a mut dyn WorldReloadContext,
        jjc: &'a mut CJJcSystem,
        gods_battle: &'a mut CGodsBattleConf,
        skills: &'a mut CSkillFactory,
        rs_gods_battle: Option<&'a mut TiberiusRsGodsBattle>,
        profile: &'a [u8],
        send_to_game_servers: bool,
        reload_server_resources: bool,
    ) -> Pin<Box<dyn Future<Output = WorldReloadResult> + 'a>>;

    /// Переименование игрока с DB-нагрузкой через owner: boxed future по
    /// ADR-0013 поверх inherent `CGame::change_map_player_name`, async-
    /// контракт и порядок DB-эффектов не меняются.
    #[allow(clippy::type_complexity, reason = "точный вызов inherent change_map_player_name с теми же аргументами")]
    fn change_map_player_name<'a>(
        &'a mut self,
        player_id: u32,
        requested_name: Option<&'a [u8]>,
        globe_setup: &'a GlobeSetupSnapshot,
        rs_player: &'a mut (dyn WorldRenameDbView + 'a),
        player_database: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = Result<WorldPlayerNameChangeReport, WorldPlayerNameLookupError>> + 'a>>;
}

/// Узкий dyn-заменитель одного DB-запроса переименования. `RsPlayerOwner`
/// с RPITIT-возвратами и generic-методами dyn-несовместим (vtable),
/// поэтому запрос `is_name_exist` публикуется как boxed future по той же
/// форме ADR-0013; реализация живёт у владельца игрока в старом пакете и
/// делегирует `RsPlayerOwner::<CPlayer>::is_name_exist`.
pub trait WorldRenameDbView {
    fn is_name_exist<'a>(
        &'a mut self,
        player_name: &'a [u8],
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>>;
}

/// Узкий dyn-заменитель трёх DB-запросов ветви удаления роли: страна
/// игрока, дата постановки на удаление и имя для журнала удаления. По той
/// же причине dyn-несовместимости `RsPlayerOwner`, что и у
/// [`WorldRenameDbView`], каждый запрос публикуется boxed future по
/// ADR-0013; реализация живёт у владельца игрока в старом пакете и
/// делегирует `RsPlayerOwner::<CPlayer>` одноимённым методам.
#[allow(clippy::type_complexity, reason = "boxed-формы повторяют параметры owner-методов один к одному")]
pub trait WorldDeleteRoleDbView {
    fn get_player_country_by_id<'a>(
        &'a mut self,
        player_id: u32,
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = u8> + 'a>>;

    fn get_player_deletion_date<'a>(
        &'a mut self,
        player_id: u32,
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = i32> + 'a>>;

    fn get_player_name_by_id<'a>(
        &'a mut self,
        player_id: u32,
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = Vec<u8>> + 'a>>;
}

/// Узкий dyn-заменитель двух DB-запросов ветви создания роли: счётчик
/// персонажей аккаунта и проверка занятости имени в базе. По той же причине
/// dyn-несовместимости `RsPlayerOwner`, что и у [`WorldRenameDbView`], оба
/// запроса публикуются boxed future по ADR-0013; реализация живёт у
/// владельца игрока в старом пакете и делегирует одноимённым методам
/// `RsPlayerOwner::<CPlayer>`.
#[allow(clippy::type_complexity, reason = "boxed-формы повторяют параметры owner-методов один к одному")]
pub trait WorldCreateRoleDbView {
    fn get_player_count_in_cdkey<'a>(
        &'a mut self,
        account: &'a [u8],
        creation_count: u8,
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = Option<u8>> + 'a>>;

    fn is_name_exist<'a>(
        &'a mut self,
        player_name: &'a [u8],
        active_transaction: Option<&'a mut WorldTdsClient>,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>>;
}

/// Узкий dyn-шов проверки должности игрока в стране к владельцу страновых
/// состояний: `CCountryHandler` держит live-таблицу, а контекст текста
/// короля получает игру извне — поэтому метод принимает `&dyn WorldGameView`
/// коротким перезаймом от обработчика. Реализация живёт в адаптере у
/// dispatcher-а старого пакета и повторяет исходную цепочку
/// `get_country → has_job` с effects-структурой кода delete-role.
pub trait WorldDeleteRoleCountryGate {
    fn country_has_job(
        &mut self,
        game: &dyn WorldGameView,
        country: u8,
        player_id: i32,
    ) -> bool;
}

