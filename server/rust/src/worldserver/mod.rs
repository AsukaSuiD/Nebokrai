//! Владельцы исторического WorldServer.

pub(crate) mod appworld {
    pub(crate) mod country {
        #[allow(
            clippy::module_inception,
            dead_code,
            reason = "двойной country буквально сохраняет исходный PDB-путь; save-проекция подключена перед полным lifecycle"
        )]
        pub(crate) mod country;
        #[allow(
            dead_code,
            reason = "country generator подключён перед единым CGame::GenerateDBData"
        )]
        pub(crate) mod countryhandler;
        #[allow(
            dead_code,
            reason = "country return-point owner подключён к WorldRegion enter-chain"
        )]
        pub(crate) mod countryparam;
        #[allow(
            dead_code,
            reason = "king point setters подключены к country scalar-sync перед остальным lifecycle"
        )]
        pub(crate) mod king;
        #[allow(
            dead_code,
            reason = "country victory producer подключён через World/Game message boundary перед phase scheduler"
        )]
        pub(crate) mod countrywarsys;
    }
    pub(crate) mod goods {
        #[allow(
            dead_code,
            reason = "battle-fairy compose owner подключён перед reload context wiring"
        )]
        pub(crate) mod cbattlefairyproperty;
        pub(crate) mod cgoods;
        pub(crate) mod cgoodsfactory;
    }
    #[allow(
        dead_code,
        reason = "Goods War runtime-owner подключён к organizing ingress до DB reload и остальных mutations"
    )]
    pub(crate) mod goodswarmember;
    pub(crate) mod incrementlog {
        pub(crate) mod incrementlog;
    }
    pub(crate) mod jjcsystem;
    pub(crate) mod leiting;
    pub(crate) mod misc;
    pub(crate) mod message {
        #[allow(
            dead_code,
            reason = "S2W auction relay, GlobeSetup и player-virtual подключены к ProcessMessage; DB load-ветвь подключена через DbMiscContext"
        )]
        pub(crate) mod auction;
        #[allow(
            dead_code,
            reason = "M2W auction `0x15EB01..=0x15EB08` подключён к ProcessMessage вместе с точными DbMisc-ветвями"
        )]
        pub(crate) mod onmsg_m2w_auction;
        pub(crate) mod gmamessage;
        pub(crate) mod gmmessage;
        pub(crate) mod jjcsysmessage;
        #[allow(
            dead_code,
            reason = "restore-role 0x4FB03 подключён перед остальными log opcodes"
        )]
        pub(crate) mod logmessage;
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
        #[allow(
            dead_code,
            reason = "подключены достигнутые transport/copy-number/honor ветви перед остальными other opcodes"
        )]
        pub(crate) mod othermessage;
        pub(crate) mod playermessage;
        pub(crate) mod servermessage;
        pub(crate) mod teammessage;
        #[allow(
            dead_code,
            reason = "все достигнутые write-log opcode `0x60201..=0x60218` подключены к ProcessMessage"
        )]
        pub(crate) mod writelogmessage;
    }
    pub(crate) mod organizingsystem {
        #[allow(
            dead_code,
            reason = "городское расписание подключено перед callback dispatcher и City enter-chain"
        )]
        pub(crate) mod attackcitysys;
        #[allow(
            dead_code,
            reason = "member/save-state CFaction подключён к callbacks и DB property owner-у"
        )]
        pub(crate) mod faction;
        #[allow(
            dead_code,
            reason = "enemy-faction generator подключён перед единым CGame::GenerateDBData"
        )]
        pub(crate) mod factionwarsys;
        #[allow(
            dead_code,
            reason = "FourNationWar serializer подключён к initial-config до loader/timer lifecycle"
        )]
        pub(crate) mod fournationwarsys;
        #[allow(
            dead_code,
            reason = "общие organizing DTO и eOperator подключены перед callback-ами"
        )]
        pub(crate) mod organizing;
        pub(crate) mod organizingctrl;
        #[allow(
            dead_code,
            reason = "параметры организаций подключены к Init и timer callback до остальных faction callers"
        )]
        pub(crate) mod organizingparam;
        #[allow(
            dead_code,
            reason = "save-state CUnion подключён к CGame::tagDBData перед GenerateSaveData"
        )]
        pub(crate) mod union;
        #[allow(
            dead_code,
            reason = "деревенское расписание подключено перед callback dispatcher и region enter-chain"
        )]
        pub(crate) mod villagewarsys;
    }
    pub(crate) mod player;
    pub(crate) mod region;
    pub(crate) mod script {
        #[allow(
            dead_code,
            reason = "SaveVarData подключён перед второй транзакционной фазой DoSaveData"
        )]
        pub(crate) mod variablelist;
    }
    pub(crate) mod session {
        #[allow(
            dead_code,
            reason = "factory сохраняет весь восстановленный virtual API, включая пока недостигнутые ветви"
        )]
        pub(crate) mod csessionfactory;
    }
    pub(crate) mod skills {
        #[allow(
            dead_code,
            reason = "skill registry подключён к initial-config до полного skill loader-а"
        )]
        pub(crate) mod skillfactory;
    }
    pub(crate) mod worldcityregion;
    pub(crate) mod worldcountrywarregion;
    #[allow(
        dead_code,
        reason = "CWorldRegion подключён к organizing lookup и region generator до полного GenerateDBData"
    )]
    pub(crate) mod worldregion;
    pub(crate) mod worldvillageregion;
}

#[allow(
    clippy::module_inception,
    reason = "двойной worldserver буквально сохраняет исходный PDB-путь server/worldserver/worldserver"
)]
pub(crate) mod worldserver;
