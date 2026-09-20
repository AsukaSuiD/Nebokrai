#pragma once
#include "rssetup.h"
#include "../../world/appworld/country/country.h"

/* Исходный владелец: dbaccess/worlddb/dbcountry.cpp/.h. Rust/EXE подтверждают
 * 37 параметров UPDATE страны; DB owner принимает уже клонированный snapshot,
 * не читает live CCountry во время транзакции. */
class CDBCountry { public: static WorldDbResult Load(IWorldDbExecutor&);static WorldDbResult Save(IWorldDbExecutor&,const CountrySaveSnapshot&); };
