//! Владельцы исторического GameServer.

pub(crate) mod appserver {
    pub(crate) mod ai {
        pub(crate) mod baseai;
        pub(crate) mod playerai;
    }
    pub(crate) mod area;
    pub(crate) mod baseobject;
    pub(crate) mod build;
    #[path = "other states/chbystate.rs"]
    pub(crate) mod chbystate;
    pub(crate) mod citygate;
    #[path = "other states/exstate.rs"]
    pub(crate) mod exstate;
    #[path = "other states/ridestate.rs"]
    pub(crate) mod ridestate;
    #[path = "other states/teamstate.rs"]
    pub(crate) mod teamstate;
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
    pub(crate) mod gameeffectjournal;
    pub(crate) mod jjcsystem;
    pub(crate) mod legacycodec;
    pub(crate) mod masterinfo;
    pub(crate) mod goods {
        pub(crate) mod cbattlefairyproperty;
        pub(crate) mod cgoods;
        pub(crate) mod cgoodsbaseproperties;
        pub(crate) mod cgoodsfactory;
        pub(crate) mod fairyproperties;
    }
    pub(crate) mod skills {
        pub(crate) mod baseattack;
        pub(crate) mod archery;
        pub(crate) mod archeryphalanx;
        pub(crate) mod basemagic;
        pub(crate) mod basemagicphalanx;
        pub(crate) mod battlefairybasemagic;
        pub(crate) mod battlefairybasemagicphalanx;
        pub(crate) mod callosity;
        pub(crate) mod callositystate;
        pub(crate) mod fightdefense;
        pub(crate) mod kernel;
        pub(crate) mod realmappellation;
        pub(crate) mod skillbaseproperties;
        pub(crate) mod skillfactory;
    }
    pub(crate) mod states {
        pub(crate) mod attackpower;
    }
    pub(crate) mod message {
        pub(crate) mod containermessage;
        pub(crate) mod countrymessage;
        pub(crate) mod depotmessage;
        pub(crate) mod gmamessage;
        pub(crate) mod gmmessage;
        pub(crate) mod goodsmessage;
        pub(crate) mod incrementshopmessage;
        pub(crate) mod jjcsystemmessage;
        pub(crate) mod logmessage;
        pub(crate) mod onmsg_c2s_auction;
        pub(crate) mod onmsg_w2s_auction;
        pub(crate) mod organsysmessage;
        pub(crate) mod othermessage;
        pub(crate) mod petmessage;
        pub(crate) mod playermessage;
        pub(crate) mod playershopmessage;
        pub(crate) mod regionmessage;
        pub(crate) mod sequencestring;
        pub(crate) mod servermessage;
        pub(crate) mod shapemessage;
        pub(crate) mod shopmessage;
        pub(crate) mod skillmessage;
        pub(crate) mod teammessage;
        pub(crate) mod unibillmessage;
    }
    #[path = "message packaging/cs2ccontainerobjectamountchange.rs"]
    pub(crate) mod cs2ccontainerobjectamountchange;
    #[path = "message packaging/cs2ccontainerobjectmove.rs"]
    pub(crate) mod cs2ccontainerobjectmove;
    pub(crate) mod monster;
    pub(crate) mod monsterworld;
    pub(crate) mod moveshape;
    pub(crate) mod npc;
    pub(crate) mod pksys;
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
        pub(crate) mod cequipmentcompose;
        pub(crate) mod cequipmentdakong;
        pub(crate) mod cequipmentupgrade;
        pub(crate) mod cpersonalshopbuyer;
        pub(crate) mod cpersonalshopseller;
        pub(crate) mod cplug;
        pub(crate) mod csession;
        pub(crate) mod csessionfactory;
        pub(crate) mod cteam;
        pub(crate) mod cteamate;
        pub(crate) mod ctrader;
    }
    pub(crate) mod script {
        pub(crate) mod buffskillfunc;
        pub(crate) mod function;
        pub(crate) mod parser;
        pub(crate) mod script;
        pub(crate) mod variablelist;
    }
    pub(crate) mod shape;
    pub(crate) mod summonshape;
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
