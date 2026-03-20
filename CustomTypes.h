#ifndef CUSTOM_TYPES_H
#define CUSTOM_TYPES_H
#include <windows.h>
#include <iostream>
#include <stdlib.h>

////////////////////////////////////////////////////
// Here we declare/define the non-primitive types //
////////////////////////////////////////////////////

struct Vector3 {
    double x, y, z;

    Vector3 Normalized() const;

    constexpr double Dot(Vector3 rhs) const {
        return x * rhs.x + y * rhs.y + z * rhs.z;
    }
};

// Normalized RGB color
struct CNorm {
    double r, g, b;

    // Better for determining how close a color is to another, regardless of the scale. (brightness/darkness)
    constexpr double CDot(CNorm rhs) const {
        return r * rhs.r + g * rhs.g + b * rhs.b;
    }
};

struct ColorHSL {
    double hue; // A degree on the color wheel [0..360]
    double sat; // Percentage of color [0..100]
    double lum; // Percentage of brightness [0..100]
};

// 24-bit RGB color
struct Color {
    unsigned char r, g, b;

    unsigned char Gray() const;
    unsigned char RedDev() const;
    unsigned char GreenDev() const;
    unsigned char BlueDev() const;

    constexpr CNorm Normalized() const {
        constexpr double INV_BYTE_MAX = 1.0 / 255.0;
        return {
            (double)r * INV_BYTE_MAX,
            (double)g * INV_BYTE_MAX,
            (double)b * INV_BYTE_MAX,
        };
    }

    constexpr operator COLORREF() const {
        return RGB(r,g,b);
    }

    ColorHSL ToHSL() const;

    constexpr double CDot(Color rhs) const {
        return Normalized().CDot(rhs.Normalized());
    }
};

class ClockTime {
private:
    // One hour is 45 seconds. A night is 4 minutes 30 seconds, or 270 seconds -- 2700 deciseconds.
    // This can be expressed in 12 bits as 0b101010001100.
    uint16_t deciseconds;
    int pingsSinceChange;

public:
    constexpr ClockTime() :
        deciseconds{ 0u },
        pingsSinceChange{ 0 }
    {};

    constexpr ClockTime(uint16_t deciseconds) :
        deciseconds{ deciseconds },
        pingsSinceChange{ 0 }
    {};

    uint16_t GetDeciseconds() const; // Read-only
    uint16_t GetSeconds() const; // It takes 1 bit more than a char to describe the number of seconds in a night.
    unsigned char GetMinutes() const; // Not sure what we'd need this for, but just in case.
    unsigned char GetHour() const; // What hour of the night we are at

    uint16_t GetWholeHourDeciseconds() const; // Converts hours to deciseconds, for finding how many deciseconds we are through the current hour.
    uint16_t GetDecisecondsSinceHour() const; // Finds how many deciseconds into the current hour we are.

    void UpdateTime(uint16_t);
    int GetPingsSinceChange() const;
};

// What gamestate we are in (what we can see on the screen)
enum class State : unsigned char {
    Camera = 0,
    Vent,
    Duct,
    Office,
};

namespace std {
    std::ostream& operator<<(std::ostream& stream, State state);
}

enum class Camera : unsigned char {
    WestHall = 0,
    EastHall,
    Closet,
    Kitchen,
    PirateCove,
    ShowtimeStage,
    PrizeCounter,
    PartsAndServices,
};

namespace std {
    std::ostream& operator<<(std::ostream& stream, Camera cam);
}

enum class Vent : unsigned char {
    Inactive = 0, // Snares reset after being tripped
    WestSnare,
    NorthSnare,
    EastSnare,
};

namespace std {
    std::ostream& operator<<(std::ostream& stream, Vent vent);
}

enum class Duct : bool {
    West = false,
    East = true,
};

namespace std {
    std::ostream& operator<<(std::ostream& stream, Duct duct);
}

struct OfficeData {
    double officeYaw; // How far left/right we are looking [-1,1]
};

struct CamData {
    Camera camera; // Which camera we are looking at
};

struct VentData {
    Vent ventSnare; // Which vent snare is active
};

struct DuctData {
    Duct closedDuct; // Which duct is currently closed
    POINT audioLure;
};

// This is the type which actually stores the data we have about the gamestate
class GameData {
    static constexpr uint8_t VENTILATION_NEEDS_RESET_FLAG = 1;
    static constexpr uint8_t FLASHLIGHT_FLAG = VENTILATION_NEEDS_RESET_FLAG << 1;
    // in order from left to right
    static constexpr uint8_t DOOR0_CLOSED_FLAG = FLASHLIGHT_FLAG << 1;
    static constexpr uint8_t DOOR1_CLOSED_FLAG = DOOR0_CLOSED_FLAG << 1;
    static constexpr uint8_t DOOR2_CLOSED_FLAG = DOOR1_CLOSED_FLAG << 1;
    static constexpr uint8_t DOOR3_CLOSED_FLAG = DOOR2_CLOSED_FLAG << 1;

    uint8_t flags;

public:
    ClockTime time;

    constexpr GameData() :
        flags { 0 },
        time()
    {}

    constexpr bool DoesVentilationNeedReset() const {
        return flags & VENTILATION_NEEDS_RESET_FLAG;
    }
    constexpr void VentilationHasBeenReset() {
        flags &= ~VENTILATION_NEEDS_RESET_FLAG;
    }
    constexpr void VentilationNeedsReset() {
        flags |= VENTILATION_NEEDS_RESET_FLAG;
    }
    constexpr void ToggleVentilationReset() {
        flags ^= VENTILATION_NEEDS_RESET_FLAG;
    }

    constexpr bool IsFlashlightOn() const {
        return flags & FLASHLIGHT_FLAG;
    }
    constexpr void TurnFlashlightOff() {
        flags &= ~FLASHLIGHT_FLAG;
    }
    constexpr void TurnFlashlightOn() {
        flags |= FLASHLIGHT_FLAG;
    }
    constexpr void ToggleFlashlight() {
        flags ^= FLASHLIGHT_FLAG;
    }

    constexpr bool IsDoorClosed(int door) const {
        return flags & DOOR0_CLOSED_FLAG << door;
    }
    constexpr void OpenDoor(int door) {
        flags &= ~(DOOR0_CLOSED_FLAG << door);
    }
    constexpr void CloseDoor(int door) {
        flags |= DOOR0_CLOSED_FLAG << door;
    }
    constexpr void ToggleDoor(int door) {
        flags ^= DOOR0_CLOSED_FLAG << door;
    }
};

class GameState {
    State state; // What state we are in (office, checking cameras, ducts, vents)
    union { // The metadata about the state (what part of the office, which camera)
        OfficeData od;
        CamData cd;
        VentData vd;
        DuctData dd;
    }; // Information about the current state that can tell us how to interpret information

public:
    State GetState() const {
        return state;
    }

    void SwitchToOffice(OfficeData data) {
        state = State::Office;
        od = data;
    }
    void SwitchToCam(CamData data) {
        state = State::Camera;
        cd = data;
    }
    void SwitchToVent(VentData data) {
        state = State::Vent;
        vd = data;
    }
    void SwitchToDuct(DuctData data) {
        state = State::Duct;
        dd = data;
    }

    const OfficeData* GetOfficeData() const {
        return (state == State::Office) ? &od : nullptr;
    }
    const CamData* GetCamData() const {
        return (state == State::Camera) ? &cd : nullptr;
    }
    const VentData* GetVentData() const {
        return (state == State::Vent) ? &vd : nullptr;
    }
    const DuctData* GetDuctData() const {
        return (state == State::Duct) ? &dd : nullptr;
    }

    OfficeData* GetOfficeData() {
        return (state == State::Office) ? &od : nullptr;
    }
    CamData* GetCamData() {
        return (state == State::Camera) ? &cd : nullptr;
    }
    VentData* GetVentData() {
        return (state == State::Vent) ? &vd : nullptr;
    }
    DuctData* GetDuctData() {
        return (state == State::Duct) ? &dd : nullptr;
    }

    GameData gameData;

    constexpr GameState() :
        state { State::Office },
        cd { Camera::WestHall },
        gameData()
    {}

    void DisplayData() const;
};

#endif // CUSTOM_TYPES_H
