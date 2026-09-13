//! Владельцы исторического GameServer.

pub(crate) mod appserver {
    pub(crate) mod ai {
        pub(crate) mod aifactory;
        pub(crate) mod archer;
        pub(crate) mod baseai;
        pub(crate) mod bossblue;
        pub(crate) mod bossfiend;
        pub(crate) mod bossidle;
        pub(crate) mod carriage;
        pub(crate) mod cityguardwithbow;
        pub(crate) mod cityguardwithsword;
        pub(crate) mod fixedpositionarcher;
        pub(crate) mod gladiator;
        pub(crate) mod godsbattlemonster;
        pub(crate) mod godsbattleguardwithsword;
        pub(crate) mod guardwithbow;
        pub(crate) mod guardwithsword;
        pub(crate) mod guardcountry;
        pub(crate) mod guardcountry2;
        pub(crate) mod guardtarget;
        pub(crate) mod jiumai;
        pub(crate) mod lord;
        pub(crate) mod monsterai;
        pub(crate) mod nationcouguardwithsword;
        pub(crate) mod nationgladiator;
        pub(crate) mod passivegladiator;
        pub(crate) mod pet;
        pub(crate) mod playerai;
        pub(crate) mod puninesscreature;
        pub(crate) mod smartgladiator;
        pub(crate) mod stupidarcher;
        pub(crate) mod stupidgladiator;
        pub(crate) mod vilcouguardwithbow;
        pub(crate) mod vilcouguardwithsword;
        pub(crate) mod warattackmonster;
        pub(crate) mod wardeffendmonster;
    }
    pub(crate) mod area;
    pub(crate) mod baseobject;
    pub(crate) mod build;
    #[path = "other states/chbystate.rs"]
    pub(crate) mod chbystate;
    #[path = "other states/autoprotectstate.rs"]
    pub(crate) mod autoprotectstate;
    pub(crate) mod citygate;
    #[path = "other states/exstate.rs"]
    pub(crate) mod exstate;
    #[path = "other states/improveexpstate.rs"]
    pub(crate) mod improveexpstate;
    #[path = "other states/notdisappearafterdead.rs"]
    pub(crate) mod notdisappearafterdead;
    #[path = "other states/particularstate.rs"]
    pub(crate) mod particularstate;
    #[path = "other states/ridestate.rs"]
    pub(crate) mod ridestate;
    #[path = "other states/restorehpstate.rs"]
    pub(crate) mod restorehpstate;
    #[path = "other states/restorempstate.rs"]
    pub(crate) mod restorempstate;
    #[path = "other states/restorestate.rs"]
    pub(crate) mod restorestate;
    #[path = "other states/teamstate.rs"]
    pub(crate) mod teamstate;
    #[path = "other states/scriptstate.rs"]
    pub(crate) mod scriptstate;
    #[path = "other states/usegoodsenlargedefstate.rs"]
    pub(crate) mod usegoodsenlargedefstate;
    #[path = "other states/usegoodsenlargeelmdefstate.rs"]
    pub(crate) mod usegoodsenlargeelmdefstate;
    #[path = "other states/usegoodsenlargefullmissstate.rs"]
    pub(crate) mod usegoodsenlargefullmissstate;
    #[path = "other states/usegoodsenlargemaxhpstate.rs"]
    pub(crate) mod usegoodsenlargemaxhpstate;
    #[path = "other states/usegoodsenlargemaxmpstate.rs"]
    pub(crate) mod usegoodsenlargemaxmpstate;
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
        pub(crate) mod agility;
        pub(crate) mod agility2;
        pub(crate) mod agilitystate;
        pub(crate) mod agilitystate2;
        pub(crate) mod baseattack;
        pub(crate) mod archery;
        pub(crate) mod baseprojectilecast;
        pub(crate) mod baseprojectilecheck;
        pub(crate) mod baseprojectilephalanx;
        pub(crate) mod elementprojectileattack;
        pub(crate) mod archeryphalanx;
        pub(crate) mod basemagic;
        pub(crate) mod basemagicphalanx;
        pub(crate) mod battlefairybasemagic;
        pub(crate) mod battlefairybasemagicphalanx;
        pub(crate) mod battlefairyskill;
        pub(crate) mod battlefairyattribute;
        pub(crate) mod battlefairyattributestate;
        pub(crate) mod battlefairytransfer;
        pub(crate) mod bloodloss;
        pub(crate) mod bloodlossstate;
        pub(crate) mod blind;
        pub(crate) mod bossbluefury;
        pub(crate) mod bossbluefurystate;
        pub(crate) mod bossbluequake;
        pub(crate) mod bossbluequakestate;
        pub(crate) mod bossfiendpenetrate;
        pub(crate) mod bossfiendsummon;
        pub(crate) mod callosity;
        pub(crate) mod callosity2;
        pub(crate) mod callositystate;
        pub(crate) mod chuckstone;
        pub(crate) mod chaossphere;
        pub(crate) mod chaosspherephalanx;
        pub(crate) mod chainlightning;
        pub(crate) mod cure;
        pub(crate) mod curestate;
        pub(crate) mod daubpoison;
        pub(crate) mod daubpoisonstate;
        pub(crate) mod directprojectile;
        pub(crate) mod corpsecandleblasting;
        pub(crate) mod corpseptomaine;
        pub(crate) mod fatalblow;
        pub(crate) mod fatalblowphalanx;
        pub(crate) mod firebolt;
        pub(crate) mod fireboltphalanx;
        pub(crate) mod fireball;
        pub(crate) mod fireballphalanx;
        pub(crate) mod itemskill2;
        pub(crate) mod thunderfirephalanx;
        pub(crate) mod firewall;
        pub(crate) mod firewallphalanx;
        pub(crate) mod godpunishment;
        pub(crate) mod godpunishmentphalanx;
        pub(crate) mod godthunder;
        pub(crate) mod godthunderphalanx;
        pub(crate) mod elementphalanxattack;
        pub(crate) mod directelementattack;
        pub(crate) mod maskedelementphalanx;
        pub(crate) mod godthunder2;
        pub(crate) mod godthunderphalanx2;
        pub(crate) mod lightning;
        pub(crate) mod fury;
        pub(crate) mod furystate;
        pub(crate) mod gibe;
        pub(crate) mod ghostcut;
        pub(crate) mod ghostcutattack;
        pub(crate) mod ghostcutvisual;
        pub(crate) mod ghostcut2;
        pub(crate) mod ghostcut3;
        pub(crate) mod frontcellsword;
        pub(crate) mod frontcellswordcast;
        pub(crate) mod frontcellswordvisual;
        pub(crate) mod knightcut;
        pub(crate) mod knightcutattack;
        pub(crate) mod knightcutvisual;
        pub(crate) mod knightcutstate;
        pub(crate) mod armybreak;
        pub(crate) mod armybreak2;
        pub(crate) mod rage;
        pub(crate) mod ragebreak;
        pub(crate) mod ragebreakstate;
        pub(crate) mod rush;
        pub(crate) mod rushstate;
        pub(crate) mod rush2;
        pub(crate) mod rushstate2;
        pub(crate) mod roar;
        pub(crate) mod roarstate;
        pub(crate) mod energyholding;
        pub(crate) mod selfcastvisual;
        pub(crate) mod selfstatecast;
        pub(crate) mod energyholdingstate;
        pub(crate) mod accumulatedstate;
        pub(crate) mod inversechopped;
        pub(crate) mod flash;
        pub(crate) mod dash;
        pub(crate) mod playercast;
        pub(crate) mod rangedweaponcast;
        pub(crate) mod crossbowattack;
        pub(crate) mod crossbowcastvisual;
        pub(crate) mod scopedarrowcast;
        pub(crate) mod skillpath;
        pub(crate) mod armybreakattack;
        pub(crate) mod weaponattack;
        pub(crate) mod impactattack;
        pub(crate) mod mosouvisual;
        pub(crate) mod thunder;
        pub(crate) mod swallow;
        pub(crate) mod swallowattack;
        pub(crate) mod swallowvisual;
        pub(crate) mod thunderblow;
        pub(crate) mod thunderblow2;
        pub(crate) mod thunderblow2visual;
        pub(crate) mod thunderblowphalanx;
        pub(crate) mod thunderslash;
        pub(crate) mod thunderslashvisual;
        pub(crate) mod thunderslashphalanx;
        pub(crate) mod thunderphalanx;
        pub(crate) mod thunder2;
        pub(crate) mod thunder2phalanx;
        pub(crate) mod tianhuo;
        pub(crate) mod tianhuophalanx;
        pub(crate) mod enlargemaxhp;
        pub(crate) mod enlargemaxhpstate;
        pub(crate) mod enlargemaxmp;
        pub(crate) mod enlargemaxmpstate;
        pub(crate) mod enlargefullmiss;
        pub(crate) mod enlargefullmissstate;
        pub(crate) mod energybolt;
        pub(crate) mod pathprojectilevisual;
        pub(crate) mod fightdefense;
        pub(crate) mod hearten;
        pub(crate) mod heartenstate;
        pub(crate) mod heartlessarrow;
        pub(crate) mod heartlessarrow2;
        pub(crate) mod heartlessarrow3;
        pub(crate) mod heartlessarrowphalanx2;
        pub(crate) mod heartlessarrowphalanx3;
        pub(crate) mod lightingarrow;
        pub(crate) mod arrowcastvisual;
        pub(crate) mod lightingarrow2;
        pub(crate) mod lightingarrowphalanx;
        pub(crate) mod meteorarrow;
        pub(crate) mod meteorarrowmass;
        pub(crate) mod meteorarrowphalanx;
        pub(crate) mod meteorarrowstate;
        pub(crate) mod rainarrow;
        pub(crate) mod rainarrowphalanx;
        pub(crate) mod heal;
        pub(crate) mod heal2;
        pub(crate) mod healstate;
        pub(crate) mod superheal;
        pub(crate) mod superheal2;
        pub(crate) mod huoxieshu;
        pub(crate) mod immediatestate;
        pub(crate) mod immediatestateinstallation;
        pub(crate) mod infernol;
        pub(crate) mod jucut;
        pub(crate) mod kerosene;
        pub(crate) mod kerosenestate;
        pub(crate) mod ignition;
        pub(crate) mod combustioncast;
        pub(crate) mod lightningsword;
        pub(crate) mod lightningsword2;
        pub(crate) mod lightningsword3;
        pub(crate) mod lightningsword4;
        pub(crate) mod littleflash;
        pub(crate) mod littleflash2;
        pub(crate) mod kernel;
        pub(crate) mod lifeshield;
        pub(crate) mod leafcut;
        pub(crate) mod leafcutapply;
        pub(crate) mod leafcutvisual;
        pub(crate) mod leafcut2;
        pub(crate) mod leafcut3;
        pub(crate) mod leafcutstate;
        pub(crate) mod leafcutstate2;
        pub(crate) mod leafcutstate3;
        pub(crate) mod lifeshieldstate;
        pub(crate) mod littlestar;
        pub(crate) mod lingzhishu;
        pub(crate) mod lordfastattack;
        pub(crate) mod lordwiderangingattack;
        pub(crate) mod machineshield;
        pub(crate) mod machineshieldstate;
        pub(crate) mod machinerystomp;
        pub(crate) mod manashield;
        pub(crate) mod manashieldstate;
        pub(crate) mod monsterbaseattack;
        pub(crate) mod monsterattack;
        pub(crate) mod monsterfastattack;
        pub(crate) mod monsterprojectile;
        pub(crate) mod monsterrangeattack;
        pub(crate) mod monstertaming;
        pub(crate) mod mosou;
        pub(crate) mod monsterthorn;
        pub(crate) mod natural;
        pub(crate) mod nonfun;
        pub(crate) mod origin;
        pub(crate) mod originstate;
        pub(crate) mod petscontrol;
        pub(crate) mod pillar;
        pub(crate) mod pillarstate;
        pub(crate) mod poisonarrow;
        pub(crate) mod poisonarrowstate;
        pub(crate) mod poisonfog;
        pub(crate) mod poisonfogphalanx;
        pub(crate) mod poisonfogstate;
        pub(crate) mod poisonmoth;
        pub(crate) mod scorpion;
        pub(crate) mod boalock;
        pub(crate) mod boalockattack;
        pub(crate) mod boalockstate;
        pub(crate) mod fallingstar;
        pub(crate) mod strike;
        pub(crate) mod targetedprojectile;
        pub(crate) mod strikestate;
        pub(crate) mod promotion;
        pub(crate) mod promotionstate;
        pub(crate) mod pojia;
        pub(crate) mod pojiastate;
        pub(crate) mod pobing;
        pub(crate) mod pobingstate;
        pub(crate) mod pomo;
        pub(crate) mod pomostate;
        pub(crate) mod pofa;
        pub(crate) mod pofastate;
        pub(crate) mod knockout;
        pub(crate) mod knockoutruntime;
        pub(crate) mod knockoutstate;
        pub(crate) mod snowstorm;
        pub(crate) mod snowstormphalanx;
        pub(crate) mod soulcollect;
        pub(crate) mod soulcollectstate;
        pub(crate) mod soulmirror;
        pub(crate) mod weak;
        pub(crate) mod zonalcast;
        pub(crate) mod weakphalanx;
        pub(crate) mod weakstate;
        pub(crate) mod godbless;
        pub(crate) mod godblessstate;
        pub(crate) mod godbless2;
        pub(crate) mod godblessstate2;
        pub(crate) mod rapture;
        pub(crate) mod sevenshootingstar;
        pub(crate) mod seal;
        pub(crate) mod sealstate;
        pub(crate) mod yinyang;
        pub(crate) mod yinyang2;
        pub(crate) mod yinyangphalanx;
        pub(crate) mod yinyangphalanx2;
        pub(crate) mod realmappellation;
        pub(crate) mod skeletonarchery;
        pub(crate) mod snakebolt;
        pub(crate) mod spiderpoison;
        pub(crate) mod spiderpoisonstate;
        pub(crate) mod spidermist;
        pub(crate) mod spidermistphalanx;
        pub(crate) mod spiderweb;
        pub(crate) mod spiderwebstate;
        pub(crate) mod sporeblasting;
        pub(crate) mod spriteburn;
        pub(crate) mod spriteburnstate;
        pub(crate) mod blindstate;
        pub(crate) mod swordship;
        pub(crate) mod swordshipstate;
        pub(crate) mod summoncorpsecandle;
        pub(crate) mod summoncreatureskill;
        pub(crate) mod summonskeleton;
        pub(crate) mod summonspore;
        pub(crate) mod skillbaseproperties;
        pub(crate) mod skillfactory;
        pub(crate) mod statefactory;
        pub(crate) mod shieldstate;
        pub(crate) mod selfshield;
        pub(crate) mod stateskill;
        pub(crate) mod taiji;
        pub(crate) mod taijistate;
        pub(crate) mod tianshenxiafanstate;
        pub(crate) mod wangsheng;
        pub(crate) mod wangshengstate;
        pub(crate) mod yakshaslash;
        pub(crate) mod wuxing;
        pub(crate) mod wuxingearth;
        pub(crate) mod wuxingfire;
        pub(crate) mod wuxingmetal;
        pub(crate) mod wuxingstate;
        pub(crate) mod wuxingwater;
        pub(crate) mod wuxingwood;
        pub(crate) mod yujia;
        pub(crate) mod yujiastate;
        pub(crate) mod yubing;
        pub(crate) mod yubingstate;
        pub(crate) mod yumo;
        pub(crate) mod yumostate;
        pub(crate) mod yunshenglightning;
        pub(crate) mod yufa;
        pub(crate) mod yufastate;
        pub(crate) mod zombieclaw;
    }
    pub(crate) mod states {
        pub(crate) mod periodicattack;
        pub(crate) mod poison;
        pub(crate) mod automaticrestore;
        pub(crate) mod attackpower;
        pub(crate) mod skill;
        pub(crate) mod state;
        pub(crate) mod summonskill;
        pub(crate) mod visualeffect;
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
    pub(crate) mod listener {
        pub(crate) mod ccontainerlistener;
        pub(crate) mod cgoodslistlistener;
        pub(crate) mod cgoodsparticularpropertylistener;
        pub(crate) mod cgoodsrepairlistener;
        pub(crate) mod cgoodsrepairpricelistener;
        pub(crate) mod cseekgoodslistener;
        pub(crate) mod cupgradepricelistener;
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
        pub(crate) mod jjcfunc;
        pub(crate) mod parser;
        pub(crate) mod script;
        pub(crate) mod variablelist;
    }
    pub(crate) mod shape;
    pub(crate) mod summonshape;
    pub(crate) mod summonedcreature;
}

#[allow(
    clippy::module_inception,
    reason = "двойной gameserver буквально сохраняет исходный PDB-путь server/gameserver/gameserver"
)]
pub(crate) mod gameserver {
    pub(crate) mod game;
    pub(crate) mod honorranks;
    pub(crate) mod playerranks;
    pub(crate) mod runtime;
    pub(crate) mod runtimespawn;
}
