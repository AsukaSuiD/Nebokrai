#pragma once
#include "rssetup.h"
#include <string>
#include <vector>

/* Исходный владелец: dbaccess/worlddb/rsgenvar.cpp/.h. Параметризованные
 * upsert-команды заменяют старую SQL-конкатенацию, сохраняя SValue/CValue. */
struct GeneralVariableDbRow{std::string name,savedValue,currentValue;};
class CRsGenVar{public:static WorldDbResult Load(IWorldDbExecutor&);static bool Save(IWorldDbExecutor&,const std::vector<GeneralVariableDbRow>&);};
