#include "CustomTypes.h"
#include "Globals.h"
#include <cmath>
#include <iomanip>

const uint16_t& ClockTime::GetDeciseconds() const {
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
    return ((uint16_t)GetHour() * DECISECONDS_PER_HOUR);
}

uint16_t ClockTime::GetDecisecondsSinceHour() const {
    return (GetDeciseconds() - GetWholeHourDeciseconds());
}

void ClockTime::UpdateTime(const uint16_t& newTime) {
    if (newTime > deciseconds && newTime < 6000 && newTime > 0 && ((newTime - deciseconds) < 10 || pingsSinceChange > 10)) {
        deciseconds = newTime;
        pingsSinceChange = 0;
    } else ++pingsSinceChange;
}

const int& ClockTime::GetPingsSinceChange() {
    return pingsSinceChange;
}

uint8_t Color::Gray() const {
    return (uint8_t)(((int)r + (int)g + (int)b) / 3);
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

void GameState::DisplayData() const {
    std::cout << "\x1b[0;0HTime: "
        << (int)(gameData.time.GetMinutes())
        << ':' << (int)(gameData.time.GetSeconds() % SECS_PER_MIN)
        << '.' << (int)(gameData.time.GetDeciseconds() % DECISECS_PER_SEC)
        << "\n\nStatuses\n========\nVentilation " << std::setw(7) << (gameData.ventilationNeedsReset ? "WARNING" : "good")
        << "\nLeft  door  " << std::setw(6) << (gameData.doorsClosed[0] ? "closed" : "open")
        << "\nFront vent  " << std::setw(6) << (gameData.doorsClosed[1] ? "closed" : "open")
        << "\nRight door  " << std::setw(6) << (gameData.doorsClosed[2] ? "closed" : "open")
        << "\nRight vent  " << std::setw(6) << (gameData.doorsClosed[3] ? "closed" : "open")
        << "\nFlashlight  " << std::setw(3) << (gameData.flashlight ? "on" : "off")
        << "\nGamestate\n=========\nState: " << std::setw(6) << state << "\n                                 \x1b[1G";
    switch (state) {
        case State::Camera:
            std::cout << "Looking at: " << std::setw(18) << stateData.cd.camera;
            break;

        case State::Office:
            std::cout << "Yaw: " << stateData.od.officeYaw;
            break;

        default:
            std::cout << "TODO";
            break;
    }
    std::cout << "\n\n";
}

ColorHSL Color::ToHSL() const {
    CNorm col = Normalized();

    double cmax;
    double cmin;
    int cmaxComp = (col.r > col.g)
        ? ((col.r > col.b) ? 0 : 2)
        : ((col.g > col.b) ? 1 : 2);

    switch (cmaxComp) {
        case 0: cmax = col.r; break;
        case 1: cmax = col.g; break;
        case 2: cmax = col.b; break;
    }
    cmin = std::min(col.r, std::min(col.g, col.b));
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
