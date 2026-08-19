#include "msgqueue.h"

/*
 * CMsgQueue остаётся общим типизированным owner-ом. Реализация template
 * находится в header; конкретные очереди создаются service-specific net-кодом.
 */
