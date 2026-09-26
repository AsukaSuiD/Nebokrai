//! Ядро товаров (object/addon, свойства фей) и типы контейнеров Zone с операциями над ними.

pub mod camountlimitgoodscontainer; // CAmountLimitGoodsContainer: storage/query/lock ядро amount-limit контейнера.
pub mod cbank; // CBank: lock-gate поверх однослотового wallet.
pub mod cbattlefairyproperty; // CBattleFairyProperty: конфигурация и свойства боевой феи.
pub mod ccontainer; // CContainer: базовый lifecycle контейнера.
pub mod cdepot; // CDepot: lock/position/expansion core склада с extension-anchor группами.
pub mod cequipmentcontainer; // CEquipmentContainer: позиционное ядро экипировки (17 колонок) с add/remove/swap и fairy growth.
pub mod cgoods; // CGoods: object/addon core товара и кодек persistence-wire.
pub mod cgoodscontainer; // CGoodsContainer: базовый derived lifecycle owner type/id/mode и stack-merge.
pub mod cjifen; // CJiFen: JiFen-вариант однослотового currency-контейнера.
pub mod cvolumelimitgoodscontainer; // CVolumeLimitGoodsContainer: cell-модель volume-контейнера.
pub mod cwallet; // generic однослотовый currency-контейнер (CSingleCurrencyContainer) и CWallet.
pub mod cyuanbao; // CYuanBao: YuanBao-вариант однослотового currency-контейнера.
pub mod fairyproperties; // CFairyProperties: свойства обычной феи и правила роста.
pub mod identity; // генерация идентификаторов предметов поверх Shared CGuid.
pub mod playercontainers; // единый extend-id каталог плеер-контейнеров (дизайн D4).
