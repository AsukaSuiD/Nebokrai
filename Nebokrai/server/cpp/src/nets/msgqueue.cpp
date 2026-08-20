#include "msgqueue.h"

/*
 * CMsgQueue остаётся общим типизированным владельцем. Реализация шаблона
 * находится в header; конкретные очереди создаются service-specific net-кодом.
 */
