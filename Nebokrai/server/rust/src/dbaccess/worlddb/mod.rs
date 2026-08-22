//! Владельцы доступа WorldServer к исходной World DB.

#[allow(
    dead_code,
    reason = "CDBCountry::Save подготовлен для Update Country Data фазы DoSaveData"
)]
pub(crate) mod dbcountry;
#[allow(
    dead_code,
    reason = "CDBIncrementLog подключён к concrete CIncrementLog init-owner"
)]
pub(crate) mod dbincrementlog;
#[allow(
    dead_code,
    reason = "goods DB-owner подключён через CRsPlayer::CreatePlayer перед New Character transaction"
)]
pub(crate) mod dbgoods;
#[allow(
    dead_code,
    reason = "CDbMisc queue/output owner подключён к World MainLoop; auction producers и SQL callbacks достигаются следующими проходами"
)]
pub(crate) mod dbmisc;
#[allow(
    dead_code,
    reason = "CGoodsListener подключён через SaveGoodsFiled/CreatePlayer перед New Character transaction"
)]
pub(crate) mod goodslistener;
#[allow(
    dead_code,
    reason = "CLargess::SaveLoadDetails подготовлен для следующей LoadDetails phase DoSaveData"
)]
pub(crate) mod largess;
#[allow(
    dead_code,
    reason = "CPlayerDataQueue подключена между полным World DB-load worker-ом и CGame::ProcessPlayerDataQueue"
)]
pub(crate) mod playerdataqueue;
#[allow(
    dead_code,
    reason = "CPlayerLoadQueue подключена к CGame, полному DB-load worker-у и account-cleanup 0x4FB06"
)]
pub(crate) mod playerloadqueue;
#[allow(
    dead_code,
    reason = "CRsEnemyFactions::SaveAllEnemyFactions подключён к EnemyFactions transaction"
)]
pub(crate) mod rsenemyfactions;
#[allow(
    dead_code,
    reason = "CRsFaction delete подключён к DoSaveData, три leaf-а подготовлены для SaveFaction"
)]
pub(crate) mod rsfaction;
#[allow(
    dead_code,
    reason = "CRsGenVar::Save подключён перед второй транзакционной фазой DoSaveData"
)]
pub(crate) mod rsgenvar;
#[allow(
    dead_code,
    reason = "CRSGodsBattle save-владельцы подготовлены для GodsBattle transaction"
)]
pub(crate) mod rsgodsbattle;
#[allow(
    dead_code,
    reason = "CRsJJcSys::SaveJJcData подключён через CreatePlayerAbilities/CreatePlayer"
)]
pub(crate) mod rsjjcsys;
#[allow(
    dead_code,
    reason = "CRsPlayer create/restore/delete подключены к транзакционным фазам DoSaveData"
)]
pub(crate) mod rsplayer;
#[allow(
    dead_code,
    reason = "CRsRegion::Save и полный region snapshot подключены до единого save-owner"
)]
pub(crate) mod rsregion;
pub(crate) mod rssetup;
#[allow(
    dead_code,
    reason = "CRsUnion::DelConfederation подключён к Delete Union фазе DoSaveData"
)]
pub(crate) mod rsunion;
#[allow(
    dead_code,
    reason = "CWriteLogQueue подключена к typed World write-log worker и всем log producers"
)]
pub(crate) mod writelogqueue;
