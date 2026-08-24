//! Buyer plug personal shop GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/session/cpersonalshopbuyer.cpp`. Buyer хранит owner/session/plug,
//! находит seller среди plug-ов той же session, а terminal callback сбрасывает
//! `PROGRESS_SHOPPING` и посылает `0xC0008(session)`. Registry lookup и wire
//! исполняются `CSessionFactory`/`playershopmessage`; safe IDs заменяют RTTI.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CPersonalShopBuyer {
    plug_id: i32,
    session_id: i32,
    owner_id: i32,
}

impl CPersonalShopBuyer {
    pub(crate) const fn inserted(plug_id: i32, session_id: i32, owner_id: i32) -> Self {
        Self {
            plug_id,
            session_id,
            owner_id,
        }
    }

    pub(crate) const fn plug_id(&self) -> i32 {
        self.plug_id
    }

    pub(crate) const fn session_id(&self) -> i32 {
        self.session_id
    }

    pub(crate) const fn owner_id(&self) -> i32 {
        self.owner_id
    }
}
