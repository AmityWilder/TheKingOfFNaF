#include "CustomTypes.h"
#include "Globals.h"
#include <cmath>
#include <iomanip>

Vector3 Vector3::Normalized() const {
    double invLength = 1.0 / sqrt(x*x + y*y + z*z);
    return { x*invLength, y*invLength, z*invLength };
}

uint16_t ClockTime::GetDeciseconds() const {
    return deciseconds;
}

uint16_t ClockTime::GetSeconds() const {
    return (deciseconds / DECISECS_PER_SEC);
}

uint8_t ClockTime::GetMinutes() const {
    return (uint8_t)(GetSeconds() / SECS_PER_MIN); // 60 seconds (realtime)
}

uint8_t ClockTime::GetHour() const {
    return (uint8_t)(GetSeconds() / SECS_PER_HOUR); // 45 seconds (gametime)
}

uint16_t ClockTime::GetWholeHourDeciseconds() const {
    return ((uint16_t)GetHour() * DECISECS_PER_HOUR);
}

uint16_t ClockTime::GetDecisecondsSinceHour() const {
    return (GetDeciseconds() - GetWholeHourDeciseconds());
}

void ClockTime::UpdateTime(uint16_t newTime) {
    if (newTime > deciseconds && newTime < 6000 && newTime > 0 && ((newTime - deciseconds) < 10 || pingsSinceChange > 10)) {
        deciseconds = newTime;
        pingsSinceChange = 0;
    } else ++pingsSinceChange;
}

int ClockTime::GetPingsSinceChange() const {
    return pingsSinceChange;
}

uint8_t Color::Gray() const {
    return (uint8_t)(((unsigned short)r + (unsigned short)g + (unsigned short)b) / 3);
}

uint8_t Color::RedDev() const {
    int distFromMean = (r - Gray());
    return (uint8_t)sqrt((distFromMean * distFromMean) / 3);
}

uint8_t Color::GreenDev() const {
    int distFromMean = (g - Gray());
    return (uint8_t)sqrt((distFromMean * distFromMean) / 3);
}

uint8_t Color::BlueDev() const {
    int distFromMean = (b - Gray());
    return (uint8_t)sqrt((distFromMean * distFromMean) / 3);
}

namespace std {
    std::ostream& operator<<(std::ostream& stream, State state) {
        switch (state) {
            case State::Office: return stream << "Office";
            case State::Camera: return stream << "Camera";
            case State::Vent: return stream << "Vent";
            case State::Duct: return stream << "Duct";
            default: return stream << "Error";
        }
    }

    std::ostream& operator<<(std::ostream& stream, Camera cam) {
        switch (cam) {
            case Camera::EastHall: return stream << "East hall";
            case Camera::Kitchen: return stream << "Kitchen";
            case Camera::PartsAndServices: return stream << "Parts and services";
            case Camera::PirateCove: return stream << "Pirate cove";
            case Camera::PrizeCounter: return stream << "Prize counter";
            case Camera::ShowtimeStage: return stream << "Showtime stage";
            case Camera::WestHall: return stream << "West hall";
            case Camera::Closet: return stream << "Supply closet";
            default: return stream << "Error";
        }
    }

    std::ostream& operator<<(std::ostream& stream, Vent vent) {
        switch (vent) {
            case Vent::Inactive: return stream << "Inactive";
            case Vent::WestSnare: return stream << "West snare";
            case Vent::NorthSnare: return stream << "North snare";
            case Vent::EastSnare: return stream << "East snare";
            default: return stream << "Error";
        }
    }

    std::ostream& operator<<(std::ostream& stream, Duct duct) {
        switch (duct) {
            case Duct::West: return stream << "West";
            case Duct::East: return stream << "East";
            default: return stream << "Error";
        }
    }
}

void GameState::DisplayData() const {
    std::cout << "\x1b[0;0H"
        << "Time: "
            << (int)(gameData.time.GetMinutes())
            << ':' << (int)(gameData.time.GetSeconds() % SECS_PER_MIN)
            << '.' << (int)(gameData.time.GetDeciseconds() % DECISECS_PER_SEC) << '\n'
        << '\n'
        << std::setw(13) << std::right << "Ventilation: " << (gameData.DoesVentilationNeedReset() ? "WARNING" : "good   ") << '\n'
        << std::setw(13) << std::right << "Left door: "   << (gameData.IsDoorClosed(0) ? "closed" : "open  ") << '\n'
        << std::setw(13) << std::right << "Front vent: "  << (gameData.IsDoorClosed(1) ? "closed" : "open  ") << '\n'
        << std::setw(13) << std::right << "Right door: "  << (gameData.IsDoorClosed(2) ? "closed" : "open  ") << '\n'
        << std::setw(13) << std::right << "Right vent: "  << (gameData.IsDoorClosed(3) ? "closed" : "open  ") << '\n'
        << std::setw(13) << std::right << "Flashlight: "  << (gameData.IsFlashlightOn() ? "on " : "off") << '\n'
        << '\n';

    std::cout << '<';
    for (const State s : { State::Camera, State::Vent, State::Duct, State::Office }) {
        const char* delim = (s == state) ? "[]" : "  ";
        std::cout << delim[0] << s << delim[1];
    }
    std::cout << '>';

    std::cout << "\n                                 \x1b[1G";

    switch (state) {
        case State::Camera:
            std::cout << std::setw(12) << std::right << "Looking at: " << cd.camera;
            break;

        case State::Office:
            std::cout << std::setw(5) << std::right << "Yaw: " << od.officeYaw;
            break;

        default:
            std::cout << "TODO";
            break;
    }

    std::cout << "\n\n";
}

ColorHSL Color::ToHSL() const {
    CNorm col = Normalized();

    // sadly windows.h creates macro definitions for ALL-LOWERCASE min/max that shadow the std::min/max functions :(
    double cmax = max(col.r, max(col.g, col.b));
    double cmin = min(col.r, min(col.g, col.b));
    int cmaxComp = (col.r > col.g)
        ? ((col.r > col.b) ? 0 : 2)
        : ((col.g > col.b) ? 1 : 2);

    double delta = cmax - cmin;

    double h, s, l;

    // Hue
    if (delta == 0.0) h = 0.0;
    else {
        switch (cmaxComp) {
            case 0: h = 60.0 *  ((col.g - col.b) / delta);        break; // Red
            case 1: h = 60.0 * (((col.b - col.r) / delta) + 2.0); break; // Green
            case 2: h = 60.0 * (((col.r - col.g) / delta) + 4.0); break; // Blue
        }
    }

    // Lum
    l = 0.5 * (cmax + cmin);

    // Sat
    if (delta == 0.0) s = 0;
    else s = delta / (1 - abs(2 * l - 1));

    // Finished
    return { h,s,l };
}
