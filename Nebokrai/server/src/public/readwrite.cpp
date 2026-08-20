#include "readwrite.h"

#include <string>

bool ReadTo(std::istream& stream, const char* name)
{
    // ПОДТВЕРЖДЕНО АССЕМБЛЕРОМ 0x00420AC0: первый токен читается безусловно.
    std::string token;
    if (!(stream >> token)) {
        return false;
    }

    while (token != name) {
        // Прямой владелец проверяет stream.eof() перед буквальным терминатором.
        // Проверка полного stream-state — техническая Linux-граница против
        // бесконечного цикла при badbit без EOF; на допустимом вводе порядок тот же.
        if (token == "<end>" || !(stream >> token)) {
            return false;
        }
    }
    return true;
}
