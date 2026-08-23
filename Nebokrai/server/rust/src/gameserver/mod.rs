//! Владельцы исторического GameServer.

pub(crate) mod appserver {
    pub(crate) mod ai {
        pub(crate) mod baseai;
    }
    pub(crate) mod area;
    pub(crate) mod baseobject;
    pub(crate) mod build;
    pub(crate) mod citygate;
    pub(crate) mod country {
        pub(crate) mod countryparam;
        pub(crate) mod countrywarsys;
    }
    pub(crate) mod goodswarmember;
    pub(crate) mod goods {
        pub(crate) mod cbattlefairyproperty;
    }
    pub(crate) mod message {
        pub(crate) mod countrymessage;
        pub(crate) mod organsysmessage;
        pub(crate) mod sequencestring;
        pub(crate) mod servermessage;
    }
    pub(crate) mod moveshape;
    pub(crate) mod player;
    pub(crate) mod organizingsystem {
        pub(crate) mod attackcitysys;
        pub(crate) mod villagewarsys;
    }
    pub(crate) mod region;
    pub(crate) mod servercityregion;
    pub(crate) mod servercountryregion;
    pub(crate) mod servergodsbattleregion;
    pub(crate) mod serverregion;
    pub(crate) mod servervillageregion;
    pub(crate) mod serverwarregion;
    pub(crate) mod session {
        pub(crate) mod cplug;
        pub(crate) mod csession;
        pub(crate) mod csessionfactory;
    }
    pub(crate) mod shape;
}

#[allow(
    clippy::module_inception,
    reason = "двойной gameserver буквально сохраняет исходный PDB-путь server/gameserver/gameserver"
)]
pub(crate) mod gameserver {
    pub(crate) mod game;
    pub(crate) mod playerranks;
}
