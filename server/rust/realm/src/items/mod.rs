//! Инвентарные контейнеры мира Realm: база, состояние, слушатели и слоты игрока.

pub mod camountlimitgoodscontainer; // контейнер товаров с лимитом количества.
pub mod cbank; // запираемый банк поверх wallet.
pub mod cbattlefairycontainer; // контейнер battle fairy поверх volume-owner-а.
pub mod ccontainer; // базовый контейнер: listeners, GUID forwarders и cleanup.
pub mod ccontainerlistener; // базовый listener обхода контейнера.
pub mod cdepot; // запираемый склад.
pub mod cequipmentcontainer; // экипировка со слотами по base-properties.
pub mod cfairycontainer; // fairy-контейнер с hatch-time.
pub mod cgoodscontainer; // общая часть контейнеров товаров: stacking и wire.
pub mod cjifen; // однослотовый JiFen с marker-wire.
pub mod cseekgoodslistener; // listener поиска товаров по цели.
pub mod cvolumelimitgoodscontainer; // контейнер с GUID-ячейками и лимитом объёма.
pub mod cwallet; // однослотовый кошелёк золотых монет.
pub mod cyuanbao; // однослотовый кошелёк YuanBao.
