//! Подготовка содержимого Realm; исполнение сценариев принадлежит Zone.

pub mod battlefairyproperty; // конфигурация объединения battle fairy.
pub mod cgoods; // товар CGoods.
pub mod cgoodsfactory; // фабрика товаров: реестры по ID и original-name.
mod clientresource; // жизненный цикл ресурсов World: установка, чтение и освобождение.
pub mod countryparam; // параметры стран CCountryParam.
pub mod dbgoods; // DB-owner товаров мира: трейт DbGoodsOwner, data-семья и Tiberius-реализация.
pub mod goods; // базовые свойства товара CGoodsBaseProperties.
pub mod goodsdb; // DB snapshots goods-домена.
pub mod goodslistener; // listener обхода товаров при сохранении полей.
pub mod organizing; // общие структуры организаций (tagMemInfo).
mod quests; // Realm-владелец каталога заданий поверх Shared-формата.
mod scriptfiles; // поиск файлов сценариев.
mod scripts; // сборка и загрузка ресурсов сценариев Realm.
pub mod skill; // wire-описание одного навыка World->Game.
pub mod skillfactory; // кэш навыков CSkillFactory.
mod timetoreturn; // календарные таблицы событий TimeToReturn.
pub mod variablelist; // переменные сценариев CVariableList.

pub use clientresource::{
    DefaultClientResourceOwner, DefaultClientResourceReplacement, LOAD_SERVER_RESOURCE_SUCCESS_LOG,
}; // владелец ресурсов World и лог-маркер успешной загрузки.
pub use quests::{QuestCatalog, QUEST_EX_PATH, QUEST_PATH}; // каталог заданий и пути его файлов.
pub use timetoreturn::{
    TimeToReturn, TimeToReturnCallbacks, TimeToReturnContext, TimeToReturnFireDisposition,
    TimeToReturnFireReport, TimeToReturnLoadError, TimeToReturnLoadReport, TimeToReturnParam,
}; // типы владельца TimeToReturn и отчёты его загрузки/срабатывания.

pub use scriptfiles::{find_script_files, ScriptFileScan, ScriptFileScanError}; // обход каталога сценариев и его ошибки.
pub use scripts::{
    normalize_script_path, ScriptListSource, ScriptLoadContext, ScriptLoadReport,
    ScriptReleaseState, ScriptRequiredFile, ScriptResources,
}; // пути, контексты и отчёты загрузки ресурсов сценариев.
