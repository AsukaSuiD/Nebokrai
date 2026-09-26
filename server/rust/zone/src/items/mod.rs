//! Ядро товаров (object/addon, свойства фей) и типы контейнеров Zone с операциями над ними.

pub mod cbattlefairyproperty; // CBattleFairyProperty: конфигурация и свойства боевой феи.
pub mod ccontainer; // CContainer: базовый lifecycle контейнера.
pub mod cgoods; // CGoods: object/addon core товара и кодек persistence-wire.
pub mod fairyproperties; // CFairyProperties: свойства обычной феи и правила роста.
pub mod identity; // генерация идентификаторов предметов поверх Shared CGuid.
