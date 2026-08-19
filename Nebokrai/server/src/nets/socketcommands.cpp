#include "socketcommands.h"

/*
 * CSocketCommands остаётся типизированным owner-ом: конкретные command payload
 * принадлежат clients/servers. Поэтому template-реализация находится в header;
 * отдельные явные инстанцирования появятся только если они реально понадобятся.
 */
