#include "countryidentity.h"

void CCountryIdentity::SetName(std::string_view value)
{
    m_Name.assign(value.substr(0, value.find('\0')));
}
