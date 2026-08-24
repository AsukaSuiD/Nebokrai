//! Владельцы исторического GameServer.

pub(crate) mod appserver {
    pub(crate) mod ai {
        pub(crate) mod baseai;
    }
    pub(crate) mod area;
    pub(crate) mod baseobject;
    pub(crate) mod build;
    pub(crate) mod citygate;
    pub(crate) mod container {
        pub(crate) mod camountlimitgoodscontainer;
        pub(crate) mod camountlimitgoodsshadowcontainer;
        pub(crate) mod cbank;
        pub(crate) mod cbattlefairycontainer;
        pub(crate) mod ccontainer;
        pub(crate) mod cdepot;
        pub(crate) mod cequipmentcomposeshadowcontainer;
        pub(crate) mod cequipmentcontainer;
        pub(crate) mod cequipmentdakongcontainer;
        pub(crate) mod cequipmentupgradeshadowcontainer;
        pub(crate) mod cfairycontainer;
        pub(crate) mod cgoodscontainer;
        pub(crate) mod cgoodsshadowcontainer;
        pub(crate) mod cjifen;
        pub(crate) mod cshadowwallet;
        pub(crate) mod cshadowyuanbao;
        pub(crate) mod cvolumelimitgoodscontainer;
        pub(crate) mod cvolumelimitgoodsshadowcontainer;
        pub(crate) mod cwallet;
        pub(crate) mod cyuanbao;
    }
    pub(crate) mod country {
        pub(crate) mod country;
        pub(crate) mod countryhandler;
        pub(crate) mod countryparam;
        pub(crate) mod countrywarsys;
    }
    pub(crate) mod goodswarmember;
    pub(crate) mod goods {
        pub(crate) mod cbattlefairyproperty;
        pub(crate) mod cgoods;
        pub(crate) mod cgoodsbaseproperties;
        pub(crate) mod cgoodsfactory;
        pub(crate) mod fairyproperties;
    }
    pub(crate) mod skills {
        pub(crate) mod skillbaseproperties;
        pub(crate) mod skillfactory;
    }
    pub(crate) mod message {
        pub(crate) mod countrymessage;
        pub(crate) mod depotmessage;
        pub(crate) mod gmamessage;
        pub(crate) mod gmmessage;
        pub(crate) mod goodsmessage;
        pub(crate) mod onmsg_w2s_auction;
        pub(crate) mod organsysmessage;
        pub(crate) mod sequencestring;
        pub(crate) mod servermessage;
        pub(crate) mod skillmessage;
    }
    pub(crate) mod monster;
    pub(crate) mod moveshape;
    pub(crate) mod npc;
    pub(crate) mod player;
    pub(crate) mod proxyserverregion;
    pub(crate) mod organizingsystem {
        pub(crate) mod attackcitysys;
        pub(crate) mod fournationwarsys;
        pub(crate) mod villagewarsys;
    }
    pub(crate) mod region;
    pub(crate) mod servercityregion;
    pub(crate) mod servercountryregion;
    pub(crate) mod servergodsbattleregion;
    pub(crate) mod servernationregion;
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
    pub(crate) mod honorranks;
    pub(crate) mod playerranks;
}
