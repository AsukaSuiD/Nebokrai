//! Общий реестр игровых сессий `CSessionFactory` старого GameServer перенесён
//! в Zone `interactions/` волной Z-C6. Здесь реэкспорт отчётных типов и блоков
//! для старого пакета и обёртка владения над Zone-реестром.
//!
//! Обёртка нужна из-за coherent-impl: неперенесённый слой
//! `nets/netserver/message.rs` реализует Zone-шов `AroundSessionLookup`
//! (разрешение session/plug по ID для around-runtime) на `CSessionFactory`, а
//! foreign-trait impl допустим только для типа этого crate. `Deref/DerefMut`
//! ведут в Zone-реестр, поэтому прежние call-site-ы через поле и accessor-ы
//! `CGame` компилируются без правок; qualified-path forwarder-ы ниже сохраняют
//! без правок тело hub-impl, обращающегося полным путём. Разрешение
//! player-owner-ов session (`session_player_ids`) остаётся в Zone-реестре и
//! доходит до кадров `0xC0101/0xC0102` через hub
//! `ContainerObjectMessageSender` у `CGame`.

use std::ops::{Deref, DerefMut};

use crate::gameserver::appserver::session::cplug::CPlug;
use crate::gameserver::appserver::session::csession::CSession;

pub(crate) use nebokrai_zone::interactions::csessionfactory::*;

/// Обёртка старого пакета над Zone-реестром
/// `interactions/csessionfactory::CSessionFactory` (волна Z-C6): единый
/// ID-поток `next_session_id/next_plug_id`, typed-карты и GC живут в Zone;
/// здесь остаётся локальный тип-владелец для coherent-impl шва
/// `AroundSessionLookup` неперенесённого слоя.
#[derive(Debug)]
pub(crate) struct CSessionFactory {
    inner: nebokrai_zone::interactions::csessionfactory::CSessionFactory,
}

impl Default for CSessionFactory {
    fn default() -> Self {
        Self {
            inner: nebokrai_zone::interactions::csessionfactory::CSessionFactory::default(),
        }
    }
}

impl Deref for CSessionFactory {
    type Target = nebokrai_zone::interactions::csessionfactory::CSessionFactory;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CSessionFactory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl CSessionFactory {
    /// Qualified-path delegates: hub `AroundSessionLookup` в
    /// `nets/netserver/message.rs` вызывает эти два поиска полным путём, где
    /// auto-deref не применяется; тела исходных `QuerySession`/`QueryPlug`
    /// живут в Zone-реестре.
    pub(crate) fn query_session(&self, session_id: i32) -> Option<&CSession> {
        self.inner.query_session(session_id)
    }

    pub(crate) fn query_plug(&self, plug_id: i32) -> Option<&CPlug> {
        self.inner.query_plug(plug_id)
    }
}
