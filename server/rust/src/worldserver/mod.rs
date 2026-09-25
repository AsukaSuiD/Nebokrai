//! Владельцы исторического WorldServer.

pub(crate) mod appworld {
    pub(crate) mod country {
        #[allow(
            clippy::module_inception,
            reason = "двойной country буквально сохраняет исходный PDB-путь"
        )]
        pub(crate) mod country;
        pub(crate) mod countryhandler;
        pub(crate) mod countryparam;
        pub(crate) mod king;
        #[allow(
            dead_code,
            reason = "country victory producer подключён через World/Game message boundary перед phase scheduler"
        )]
        pub(crate) mod countrywarsys;
    }
    pub(crate) mod goods {
        pub(crate) mod cbattlefairyproperty;
        pub(crate) mod cgoods;
        pub(crate) mod cgoodsfactory;
    }
    pub(crate) mod goodswarmember;
    pub(crate) mod incrementlog {
        pub(crate) mod incrementlog;
    }
    pub(crate) mod jjcsystem;
    pub(crate) mod leiting;
    pub(crate) mod misc;
    pub(crate) mod message {
        pub(crate) mod auction;
        pub(crate) mod onmsg_m2w_auction;
        pub(crate) mod gmamessage;
        pub(crate) mod gmmessage;
        pub(crate) mod jjcsysmessage;
        #[allow(
            dead_code,
            reason = "общий organizing session-result branch подключён перед остальными organizing opcodes"
        )]
        pub(crate) mod organsysmessage;
        #[allow(
            dead_code,
            reason = "узкий country victory dispatcher подключён перед остальными country opcodes"
        )]
        pub(crate) mod countrymessage;
        pub(crate) mod othermessage;
        pub(crate) mod playermessage;
        pub(crate) mod servermessage;
        pub(crate) mod teammessage;
        pub(crate) mod writelogmessage;
    }
    pub(crate) mod organizingsystem {
        pub(crate) mod attackcitysys;
        pub(crate) mod faction;
        pub(crate) mod factionwarsys;
        pub(crate) mod fournationwarsys;
        pub(crate) mod organizing;
        pub(crate) mod organizingctrl;
        pub(crate) mod organizingparam;
        pub(crate) mod union;
        pub(crate) mod villagewarsys;
    }
    pub(crate) mod player;
    pub(crate) mod region;
    pub(crate) mod script {
        pub(crate) mod variablelist;
    }
    pub(crate) mod session {
        pub(crate) mod csessionfactory;
    }
    pub(crate) mod skills {
        pub(crate) mod skillfactory;
    }
    pub(crate) mod worldcityregion;
    pub(crate) mod worldcountrywarregion;
    pub(crate) mod worldregion;
    pub(crate) mod worldvillageregion;
}

#[allow(
    clippy::module_inception,
    reason = "двойной worldserver буквально сохраняет исходный PDB-путь server/worldserver/worldserver"
)]
pub(crate) mod worldserver;
