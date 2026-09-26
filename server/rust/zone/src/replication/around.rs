//! Around-runtime view GameServer: разрешение игроков и sessions для
//! around-рассылки одного кадра. Точная пара GameServer `gameserver.exe` +
//! `GameServer.pdb` (идентификаторы сборки —
//! `server/rust/src/manifest/_gameserver_export_manifest.toml`).
//! Wrapper `SendToAround` RVA `0x00014970` читает father-region по `+0x40`,
//! узнаёт area-размеры владельца и текущие tile X/Y фигуры (`0x0005B110/
//! 0x0005B140`) и передаёт управление deep overload `0x00014420`; этот модуль
//! владеет только runtime-view разрешения игроков и sessions, а сам порядок
//! рассылки остаётся у старого domain-send owner-а. Положительные глобальные
//! `AREA_WIDTH/HEIGHT` выражены проверяемой runtime-границей `Option`; узкий
//! snapshot ID/team/shape временно извлечённого игрока по-прежнему пробуется
//! первым для area/main/team/owner разрешения, остальные игроки идут через
//! `AroundPlayerLookup`, sessions/plugs — через `AroundSessionLookup`.

use crate::regions::shape::CShape;
use crate::sessions::cplug::CPlug;
use crate::sessions::csession::CSession;

/// Разрешение игрока и team->session mapping для around-runtime.
pub trait AroundPlayerLookup {
    fn around_player_view(&self, player_id: i32) -> Option<AroundPlayerView<'_>>;
    fn team_session_id(&self, team_id: u32) -> i32;
}

/// Разрешение sessions/plugs для team-хвоста around-рассылки.
pub trait AroundSessionLookup {
    fn query_session(&self, session_id: i32) -> Option<&CSession>;
    fn query_plug(&self, plug_id: i32) -> Option<&CPlug>;
}

/// Узкий immutable view живого игрока для around-разрешения.
#[derive(Clone, Copy)]
pub struct AroundPlayerView<'a> {
    pub player_id: i32,
    pub team_id: i32,
    pub shape: &'a CShape,
}

/// Узкий snapshot ID/team/SHAPE временно извлечённого игрока; поля приватны,
/// наружу чтение идёт только через `GameServerAroundPlayerRef`.
pub struct GameServerAroundPlayer {
    player_id: i32,
    team_id: i32,
    shape: CShape,
}

pub enum GameServerAroundPlayerRef<'a> {
    Detached(&'a GameServerAroundPlayer),
    Live(AroundPlayerView<'a>),
}

impl GameServerAroundPlayerRef<'_> {
    pub fn player_id(&self) -> i32 {
        match self {
            Self::Detached(player) => player.player_id,
            Self::Live(player) => player.player_id,
        }
    }

    pub fn team_id(&self) -> i32 {
        match self {
            Self::Detached(player) => player.team_id,
            Self::Live(player) => player.team_id,
        }
    }

    pub fn shape(&self) -> &CShape {
        match self {
            Self::Detached(player) => &player.shape,
            Self::Live(player) => player.shape,
        }
    }
}

pub struct GameServerAroundRuntime<
    'a,
    P: AroundPlayerLookup + ?Sized,
    S: AroundSessionLookup + ?Sized,
> {
    players: &'a P,
    sessions: &'a S,
    player: Option<GameServerAroundPlayer>,
    area_width: i32,
    area_height: i32,
}

impl<'a, P: AroundPlayerLookup + ?Sized, S: AroundSessionLookup + ?Sized>
    GameServerAroundRuntime<'a, P, S>
{
    /// Создаёт runtime-view только для доказанных положительных area spans.
    pub fn new(players: &'a P, sessions: &'a S, area_width: i32, area_height: i32) -> Option<Self> {
        (area_width > 0 && area_height > 0).then_some(Self {
            players,
            sessions,
            player: None,
            area_width,
            area_height,
        })
    }

    pub fn with_player(mut self, player_id: i32, team_id: i32, shape: &CShape) -> Self {
        self.player = Some(GameServerAroundPlayer {
            player_id,
            team_id,
            shape: shape.clone(),
        });
        self
    }

    pub fn resolve_player(&self, player_id: i32) -> Option<GameServerAroundPlayerRef<'_>> {
        self.player
            .as_ref()
            .filter(|player| player.player_id == player_id)
            .map(GameServerAroundPlayerRef::Detached)
            .or_else(|| {
                self.players
                    .around_player_view(player_id)
                    .map(GameServerAroundPlayerRef::Live)
            })
    }

    pub const fn area_width(&self) -> i32 {
        self.area_width
    }

    pub const fn area_height(&self) -> i32 {
        self.area_height
    }

    pub fn players(&self) -> &P {
        self.players
    }

    pub fn team_session_id(&self, team_id: u32) -> i32 {
        self.players.team_session_id(team_id)
    }

    pub fn query_session(&self, session_id: i32) -> Option<&CSession> {
        self.sessions.query_session(session_id)
    }

    pub fn query_plug(&self, plug_id: i32) -> Option<&CPlug> {
        self.sessions.query_plug(plug_id)
    }
}

/// Coherent-impl шва для реестра `interactions::CSessionFactory`: и трейт,
/// и тип локальные для Zone, impl живёт рядом с трейтом. Тела поиска —
/// inherent-методы реестра.
impl AroundSessionLookup for crate::interactions::csessionfactory::CSessionFactory {
    fn query_session(&self, session_id: i32) -> Option<&CSession> {
        crate::interactions::csessionfactory::CSessionFactory::query_session(self, session_id)
    }

    fn query_plug(&self, plug_id: i32) -> Option<&CPlug> {
        crate::interactions::csessionfactory::CSessionFactory::query_plug(self, plug_id)
    }
}
