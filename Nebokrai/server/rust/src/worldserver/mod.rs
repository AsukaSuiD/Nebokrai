//! Владельцы исторического WorldServer.

pub(crate) mod appworld {
    pub(crate) mod baseobject;
    pub(crate) mod container {
        #[allow(
            dead_code,
            reason = "amount-container codec подключён перед остальными CPlayer container-полями"
        )]
        pub(crate) mod camountlimitgoodscontainer;
        #[allow(
            dead_code,
            reason = "bank owner подключён перед точной CPlayer container sequence"
        )]
        pub(crate) mod cbank;
        #[allow(
            dead_code,
            reason = "battle-fairy owner подключён перед точной CPlayer container sequence"
        )]
        pub(crate) mod cbattlefairycontainer;
        #[allow(
            dead_code,
            reason = "базовый container-state подключён перед listener notification graph"
        )]
        pub(crate) mod ccontainer;
        #[allow(
            dead_code,
            reason = "depot owner подключён перед точной CPlayer container sequence"
        )]
        pub(crate) mod cdepot;
        #[allow(
            dead_code,
            reason = "equipment/volume codec подключён перед точной CPlayer container sequence"
        )]
        pub(crate) mod cequipmentcontainer;
        #[allow(
            dead_code,
            reason = "fairy owner подключён перед точной CPlayer container sequence"
        )]
        pub(crate) mod cfairycontainer;
        #[allow(
            dead_code,
            reason = "достигнутый base stacking подключён к volume positional Add"
        )]
        pub(crate) mod cgoodscontainer;
        #[allow(
            dead_code,
            reason = "cjifen owner и wallet folded-codec подключены перед CPlayer sequence"
        )]
        pub(crate) mod cjifen;
        #[allow(
            dead_code,
            reason = "volume-container codec подключён перед equipment и CPlayer container sequence"
        )]
        pub(crate) mod cvolumelimitgoodscontainer;
        #[allow(
            dead_code,
            reason = "wallet codec подключён перед точной CPlayer container sequence"
        )]
        pub(crate) mod cwallet;
        #[allow(
            dead_code,
            reason = "yuanbao folded-codec подключён перед точной CPlayer container sequence"
        )]
        pub(crate) mod cyuanbao;
    }
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
        #[allow(
            dead_code,
            reason = "достигнутые base-properties подключены для точного goods stacking"
        )]
        pub(crate) mod cgoodsbaseproperties;
        pub(crate) mod cgoodsfactory;
    }
    #[allow(
        dead_code,
        reason = "Goods War runtime-owner подключён к organizing ingress до DB reload и остальных mutations"
    )]
    pub(crate) mod goodswarmember;
    pub(crate) mod jjcsystem;
    pub(crate) mod leiting;
    pub(crate) mod misc;
    #[allow(
        dead_code,
        reason = "listener-контракт подключён перед materialized CPlayer packet traversal"
    )]
    pub(crate) mod listener {
        pub(crate) mod ccontainerlistener;
        pub(crate) mod cseekgoodslistener;
    }
    pub(crate) mod message {
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
        #[allow(
            dead_code,
            reason = "подключены достигнутые transport/copy-number/honor ветви перед остальными other opcodes"
        )]
        pub(crate) mod othermessage;
        pub(crate) mod playermessage;
        pub(crate) mod servermessage;
        pub(crate) mod teammessage;
    }
    pub(crate) mod monster;
    pub(crate) mod moveshape;
    pub(crate) mod npc;
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
    pub(crate) mod shape;
    pub(crate) mod script {
        #[allow(
            dead_code,
            reason = "SaveVarData подключён перед второй транзакционной фазой DoSaveData"
        )]
        pub(crate) mod variablelist;
    }
    pub(crate) mod session {
        pub(crate) mod cplug;
        pub(crate) mod csession;
        #[allow(
            dead_code,
            reason = "factory сохраняет весь восстановленный virtual API, включая пока недостигнутые ветви"
        )]
        pub(crate) mod csessionfactory;
        pub(crate) mod cteam;
        pub(crate) mod cteamate;
    }
    pub(crate) mod skills {
        #[allow(
            dead_code,
            reason = "CSkill owner подключён к initial-config до полного skill loader-а"
        )]
        pub(crate) mod skill;
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
    pub(crate) mod worldwarregion;
}

#[allow(
    clippy::module_inception,
    reason = "двойной worldserver буквально сохраняет исходный PDB-путь server/worldserver/worldserver"
)]
pub(crate) mod worldserver;
