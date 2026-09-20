#include "cbattlefairycontainer.h"

/*
 * BattleFairy не добавляет собственного состояния: Rust подтверждает тот же
 * volume-owner и отдельный PDB-vtable только ради специализации вызовов.
 */
