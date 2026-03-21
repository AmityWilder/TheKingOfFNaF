#include <algorithm>
#include <array>
#include <atomic>
#include <cassert>
#include <cmath>
#include <iomanip>
#include <iostream>
#include <shared_mutex>
#include <thread>
#include <windows.h>

// sadly windows.h creates macro definitions for ALL-LOWERCASE min/max that shadow the std::min/max functions :(
#undef max
#undef min

//
// Global constants -- These give context to unchanging values
//

#ifdef _DEBUG
//#define DEBUG_MUTEX
#endif

constexpr const char* RESET_CURSOR = "\x1b[0;0H";

// Important positions on the screen
namespace pnt {
    // Clock position
    constexpr POINT CLK_POS = { 1807, 85 };
    constexpr int CLK_10SEC_X = 1832;
    constexpr int CLK_SEC_X = 1849;
    constexpr int CLK_DECISEC_X = 1873;

    constexpr POINT TEMPERATURE_POS = { 1818, 1012 };

    constexpr POINT COINS = { 155, 75 };

    constexpr POINT PWR_POS = { 71, 910 };
    constexpr POINT PWR_USG_POS = { 38, 969 };

    constexpr POINT NOISE_POS = { 38,1020 };

    constexpr POINT openCam = { 1280, 1006 }; // Not really needed since the S key exists...

    // Office
    namespace ofc {
        constexpr POINT MASK_POS = { 682, 1006 };
        constexpr POINT VENT_WARNING_POS = { 1580, 1040 }; // The office version of this constant

        constexpr POINT FOXY = { 801, 710 };
    }

    // Camera
    namespace cam {
        constexpr POINT VENT_WARNING_POS = { 1563, 892 }; // Location for testing vent warning in the cameras
        constexpr POINT RESET_VENT_BTN_POS = { 1700, 915 }; // Where the reset vent button is for clicking

        constexpr POINT CAM_01_POS = { 1133, 903 }; // WestHall
        constexpr POINT CAM_02_POS = { 1382, 903 }; // EastHall
        constexpr POINT CAM_03_POS = { 1067, 825 }; // Closet
        constexpr POINT CAM_04_POS = { 1491, 765 }; // Kitchen
        constexpr POINT CAM_05_POS = { 1122, 670 }; // PirateCove
        constexpr POINT CAM_06_POS = { 1422, 590 }; // ShowtimeStage
        constexpr POINT CAM_07_POS = { 1278, 503 }; // PrizeCounter
        constexpr POINT CAM_08_POS = {  988, 495 }; // PartsAndServices

        constexpr int SYS_BTN_X = 1331; // System buttons X position
        constexpr int CAM_SYS_BTN_Y = 153; // Cam system button Y position
        constexpr int VENT_SYS_BTN_Y = 263; // Vent system button Y position
        constexpr int DUCT_SYS_BTN_Y = 373; // Duct system button Y position
    }

    // Vents
    namespace vnt {
        constexpr POINT SNARE_L_POS = { 548, 645 }; // Left snare
        constexpr POINT SNARE_T_POS = { 650, 536 }; // Top snare
        constexpr POINT SNARE_R_POS = { 747, 645 }; // Right snare
    }

    // Ducts
    namespace dct {
        constexpr POINT DUCT_L_POS = {  500, 791 }; // Check left duct
        constexpr POINT DUCT_R_POS = {  777, 791 }; // Check right duct
        constexpr POINT DUCT_L_BTN_POS = {  331, 844 }; // Left duct button
        constexpr POINT DUCT_R_BTN_POS = { 1016, 844 }; // Right duct button
    }
}

constexpr int CAM_RESP_MS = 300; // Time it takes for the camera to be ready for input

constexpr int SECS_PER_MIN = 60; // Real time
constexpr int SECS_PER_HOUR = 45; // Game time
constexpr int DECISECS_PER_SEC = 10;
constexpr int DECISECS_PER_HOUR = SECS_PER_HOUR * DECISECS_PER_SEC;
constexpr int MS_PER_DECISEC = 100;

////////////////////////////////////////////////////
// Here we declare/define the non-primitive types //
////////////////////////////////////////////////////

constexpr size_t CHANNELS_PER_COLOR = 4; // Bitmap channels, not `Color` channels

// Normalized RGB color
struct CNorm {
    double r, g, b;

    // Normalize the color like a vector (necessary for performing dot product properly)
    CNorm VNormalized() const {
        const double invLength = 1.0 / sqrt(r*r + g*g + b*b);
        return { r*invLength, g*invLength, b*invLength };
    }

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

    unsigned char Gray() const {
        return (unsigned char)(((unsigned short)r + (unsigned short)g + (unsigned short)b) / 3);
    }

    unsigned char RedDev() const {
        const int distFromMean = (r - Gray());
        return (unsigned char)sqrt((distFromMean * distFromMean) / 3);
    }

    unsigned char GreenDev() const {
        const int distFromMean = (g - Gray());
        return (unsigned char)sqrt((distFromMean * distFromMean) / 3);
    }

    unsigned char BlueDev() const {
        const int distFromMean = (b - Gray());
        return (unsigned char)sqrt((distFromMean * distFromMean) / 3);
    }

    // Convert the color components from 0..=255 to 0.0..=1.0
    constexpr CNorm Normalized() const {
        constexpr double INV_BYTE_MAX = 1.0 / 255.0;
        return {
            (double)r * INV_BYTE_MAX,
            (double)g * INV_BYTE_MAX,
            (double)b * INV_BYTE_MAX,
        };
    }

    double Similarity(Color other) const {
        return Normalized().VNormalized().CDot(other.Normalized().VNormalized());
    }

    constexpr operator COLORREF() const {
        return RGB(r,g,b);
    }

    ColorHSL ToHSL() const {
        const CNorm col = Normalized();

        const double cmax = std::max(col.r, std::max(col.g, col.b));
        const double cmin = std::min(col.r, std::min(col.g, col.b));
        const int cmaxComp = (col.r > col.g)
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
};

// Color constants
namespace clr {
    constexpr Color SYS_BTN_COLOR = { 40, 152, 120 };
    const CNorm SYS_BTN_COLOR_NRM = SYS_BTN_COLOR.Normalized().VNormalized();
    constexpr Color CAM_BTN_COLOR = { 136, 172, 0 };
    const CNorm CAM_BTN_COLOR_NRM = CAM_BTN_COLOR.Normalized().VNormalized();
}

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

    constexpr uint16_t GetDeciseconds() const {
        return deciseconds;
    }

    // It takes 1 bit more than a char to describe the number of seconds in a night.
    constexpr uint16_t GetSeconds() const {
        return (deciseconds / DECISECS_PER_SEC);
    }

    // Not sure what we'd need this for, but just in case.
    constexpr uint8_t GetMinutes() const {
        return (uint8_t)(GetSeconds() / SECS_PER_MIN); // realtime
    }

    // What hour of the night we are at
    constexpr uint8_t GetHour() const {
        return (uint8_t)(GetSeconds() / SECS_PER_HOUR); // gametime
    }

    // Converts hours to deciseconds, for finding how many deciseconds we are through the current hour.
    constexpr uint16_t GetWholeHourDeciseconds() const {
        return ((uint16_t)GetHour() * DECISECS_PER_HOUR);
    }

    // Finds how many deciseconds into the current hour we are.
    constexpr uint16_t GetDecisecondsSinceHour() const {
        return (GetDeciseconds() - GetWholeHourDeciseconds());
    }

    constexpr void UpdateTime(uint16_t newTime) {
        if (newTime > deciseconds && newTime < 6000 && newTime > 0 && ((newTime - deciseconds) < 10 || pingsSinceChange > 10)) {
            deciseconds = newTime;
            pingsSinceChange = 0;
        } else ++pingsSinceChange;
    }

    constexpr int GetPingsSinceChange() const {
        return pingsSinceChange;
    }

    constexpr bool IsDefault() const {
        return deciseconds == 0;
    }
};

namespace std {
    std::ostream& operator<<(std::ostream& stream, ClockTime time) {
        return stream
            << (int)(time.GetMinutes())
            << ':' << (int)(time.GetSeconds() % SECS_PER_MIN)
            << '.' << (int)(time.GetDeciseconds() % DECISECS_PER_SEC);
    }
}

// What gamestate we are in (what we can see on the screen)
enum class State : unsigned char {
    Camera = 0,
    Vent,
    Duct,
    Office,
};

namespace std {
    std::ostream& operator<<(std::ostream& stream, State state) {
        switch (state) {
            case State::Office:
                return stream << "Office";
            case State::Camera:
                return stream << "Camera";
            case State::Vent:
                return stream << "Vent";
            case State::Duct:
                return stream << "Duct";
            default:
                return stream << "Error";
        }
    }
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
    std::ostream& operator<<(std::ostream& stream, Camera cam) {
        switch (cam) {
            case Camera::EastHall:
                return stream << "East hall";
            case Camera::Kitchen:
                return stream << "Kitchen";
            case Camera::PartsAndServices:
                return stream << "Parts and services";
            case Camera::PirateCove:
                return stream << "Pirate cove";
            case Camera::PrizeCounter:
                return stream << "Prize counter";
            case Camera::ShowtimeStage:
                return stream << "Showtime stage";
            case Camera::WestHall:
                return stream << "West hall";
            case Camera::Closet:
                return stream << "Supply closet";
            default:
                return stream << "Error";
        }
    }
}

enum class Vent : unsigned char {
    Inactive = 0, // Snares reset after being tripped
    WestSnare,
    NorthSnare,
    EastSnare,
};

namespace std {
    std::ostream& operator<<(std::ostream& stream, Vent vent) {
        switch (vent) {
            case Vent::Inactive:
                return stream << "Inactive";
            case Vent::WestSnare:
                return stream << "West snare";
            case Vent::NorthSnare:
                return stream << "North snare";
            case Vent::EastSnare:
                return stream << "East snare";
            default:
                return stream << "Error";
        }
    }
}

enum class Duct : bool {
    West = false,
    East = true,
};

namespace std {
    std::ostream& operator<<(std::ostream& stream, Duct duct) {
        switch (duct) {
            case Duct::West:
                return stream << "West";
            case Duct::East:
                return stream << "East";
            default:
                return stream << "Error";
        }
    }
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
    ClockTime nextFFShow;

    constexpr GameData() :
        flags { 0 },
        time(),
        nextFFShow()
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
    GameData gameData;

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

    constexpr GameState() :
        state { State::Office },
        cd { Camera::WestHall },
        gameData()
    {}

    void DisplayData() const;
};

/////////////////////////////////////////////////////////////////////////////////////////////
// Here, all of the variables & constants used throughout the project are declared/defined //
/////////////////////////////////////////////////////////////////////////////////////////////

//
// Global variables -- used by everyone, but changeable.
//

// Get a console handle
HWND WND_CONSOLE;
// Get a handle to device hdc
HDC CONSOLE_HDC;

int SCREEN_HEIGHT;
int SCREEN_WIDTH;

class ScreenDataGuard;

///////////////////////////////////////////////
// This is where we take input from the game //
// e.g.                                      //
// - Test pixel color at { 253, 1004 }       //
///////////////////////////////////////////////

//HWND g_gameWindow = FindWindow(NULL, TEXT("Ultimate Custom Night"));
//HDC g_gameDC = GetDC(g_gameWindow);
HDC DESKTOP_HDC; // get the desktop device context
HDC INTERNAL_HDC; // create a device context to use ourselves

// create a bitmap
HBITMAP H_BITMAP;

BITMAPINFO BMI = {
    .bmiHeader = {
        .biSize = sizeof(BITMAPINFOHEADER),
        .biWidth = SCREEN_WIDTH,
        .biHeight = -SCREEN_HEIGHT,
        .biPlanes = 1,
        .biBitCount = 32,
        .biCompression = BI_RGB,
        .biSizeImage = 0, // 3 * ScreenX * ScreenY; (position, not size)
        .biXPelsPerMeter = 0,
        .biYPelsPerMeter = 0,
        .biClrUsed = 0,
        .biClrImportant = 0,
    },
    .bmiColors = { 0 },
};

class ScreenDataGuard;

class ScreenData {
    bool isShared;
    BYTE* data;

    friend class ScreenDataGuard;

    ScreenData(bool isShared, BYTE* data) :
        isShared { isShared },
        data { data }
    {}

public:
    ~ScreenData();

    void UpdateScreencap() {
        BitBlt(INTERNAL_HDC, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, DESKTOP_HDC, 0, 0, SRCCOPY);
        GetDIBits(DESKTOP_HDC, H_BITMAP, 0, SCREEN_HEIGHT, data, &BMI, DIB_RGB_COLORS);
    }

    Color GetPixelColor(long x, long y) const {
        size_t index = CHANNELS_PER_COLOR * (size_t)((y * (long)SCREEN_WIDTH) + x);

        Color output = {
            data[index + 2], // Red
            data[index + 1], // Green
            data[index],     // Blue
        };

    #ifdef _DEBUG
        // If the function is working correctly, you shouldn't see anything change.
        // If there's something wrong, the pixel being misread will be overwritten with the color being read.
        SetPixel(CONSOLE_HDC, (int)x, (int)y, output);
    #endif

        return output;
    }

    Color GetPixelColor(POINT pos) const {
        return GetPixelColor(pos.x, pos.y);
    }
};

class ScreenDataGuard {
    std::shared_mutex mutex;
    BYTE* data = nullptr;

#ifdef DEBUG_MUTEX
    int exclusiveLocks = 0;
    int sharedLocks = 0;
#endif

public:
    void ResizeBuffer(size_t size) {
        mutex.lock();
        delete[] data; // No effect if data is nullptr
        if (size > 0) data = new BYTE[size];
        mutex.unlock();
    }

    const ScreenData LockShared() {
    #ifdef DEBUG_MUTEX
        std::cout << "Thread " << std::this_thread::get_id() << ": Waiting for shared lock on SCREEN_DATA... " << sharedLocks << " shared, " << exclusiveLocks << " exclusive\n";
    #endif
        mutex.lock_shared();
    #ifdef DEBUG_MUTEX
        ++sharedLocks;
        std::cout << "Thread " << std::this_thread::get_id() << ": Shared lock on SCREEN_DATA obtained. " << sharedLocks << " shared, " << exclusiveLocks << " exclusive\n";
    #endif
        return ScreenData { true, data };
    }

    void UnlockShared() {
        mutex.unlock_shared();
    #ifdef DEBUG_MUTEX
        --sharedLocks;
        std::cout << "Thread " << std::this_thread::get_id() << ": Shared lock on SCREEN_DATA released. " << sharedLocks << " shared, " << exclusiveLocks << " exclusive\n";
    #endif
    }

    ScreenData Lock() {
    #ifdef DEBUG_MUTEX
        std::cout << "Thread " << std::this_thread::get_id() << ": Waiting for exclusive lock on SCREEN_DATA... " << sharedLocks << " shared, " << exclusiveLocks << " exclusive\n";
    #endif
        mutex.lock();
    #ifdef DEBUG_MUTEX
        ++exclusiveLocks;
        std::cout << "Thread " << std::this_thread::get_id() << ": Exclusive lock on SCREEN_DATA obtained. " << sharedLocks << " shared, " << exclusiveLocks << " exclusive\n";
    #endif
        return ScreenData { false, data };
    }

    void Unlock() {
        mutex.unlock();
    #ifdef DEBUG_MUTEX
        --exclusiveLocks;
        std::cout << "Thread " << std::this_thread::get_id() << ": Exclusive lock on SCREEN_DATA released. " << sharedLocks << " shared, " << exclusiveLocks << " exclusive\n";
    #endif
    }

    void UpdateScreencap() {
        Lock().UpdateScreencap();
    }
} SCREEN_DATA;

ScreenData::~ScreenData() {
    if (isShared) {
        SCREEN_DATA.UnlockShared();
    } else {
        SCREEN_DATA.Unlock();
    }
}

GameState GAME_STATE = GameState(); // All the information we have about the state of the game

// These enable us to put the buttons in an array and choose from them instead of just using the literal names
// If you're trying to get the position of just the one thing and don't need to do any sort of "switch" thing, please don't use this. It adds additional steps.
enum class Button {
    Mask = 0,
    ResetVent,

    Cam01, // WestHall
    Cam02, // EastHall
    Cam03, // Closet
    Cam04, // Kitchen
    Cam05, // PirateCove
    Cam06, // ShowtimeStage
    Cam07, // PrizeCounter
    Cam08, // PartsAndServices

    CameraSystem,
    VentSystem,
    DuctSystem,

    SnareLeft,
    SnareTop,
    SnareRight,

    DuctLeft,
    DuctRight,
};

// This is the list the above enum was referring to
const POINT BTN_POSITIONS[18] = {
    pnt::ofc::MASK_POS,
    pnt::cam::RESET_VENT_BTN_POS,
    pnt::cam::CAM_01_POS,
    pnt::cam::CAM_02_POS,
    pnt::cam::CAM_03_POS,
    pnt::cam::CAM_04_POS,
    pnt::cam::CAM_05_POS,
    pnt::cam::CAM_06_POS,
    pnt::cam::CAM_07_POS,
    pnt::cam::CAM_08_POS,
    {
		pnt::cam::SYS_BTN_X,
		pnt::cam::CAM_SYS_BTN_Y
	},
    {
		pnt::cam::SYS_BTN_X,
		pnt::cam::VENT_SYS_BTN_Y
	},
    {
		pnt::cam::SYS_BTN_X,
		pnt::cam::DUCT_SYS_BTN_Y
	},
    pnt::vnt::SNARE_L_POS,
    pnt::vnt::SNARE_T_POS,
    pnt::vnt::SNARE_R_POS,
    pnt::dct::DUCT_L_BTN_POS,
    pnt::dct::DUCT_R_BTN_POS
};

// Returns the button enum of a camera int
Button CameraButton(int cam) {
    return Button((int)Button::Cam01 + cam);
}

// Returns the button enum of a camera enum
Button CameraButton(Camera cam) {
    return CameraButton((int)cam);
}

// Returns the button enum of a system int
Button SystemButton(int system) {
    return Button((int)Button::CameraSystem + system);
}

// Returns the button enum of a state enum
Button SystemButton(State system) {
    return SystemButton((int)system);
}

// Pick the button position from the list of button positions
POINT GetButtonPos(Button button) {
    return BTN_POSITIONS[(int)button];
}

////////////////////////////////////////////////////////////////////////////////////
// This is where the input we've taken from the game gets turned into useful data //
////////////////////////////////////////////////////////////////////////////////////

bool IsNMBBStanding() {
    constexpr Color PANTS_COLOR = { 0, 28, 120 };
    constexpr POINT SAMPLE_POS = { 1024, 774 };
    constexpr double THRESHOLD = 0.98;
    return (PANTS_COLOR.Similarity(SCREEN_DATA.LockShared().GetPixelColor(SAMPLE_POS)) > THRESHOLD);
}

// Input should be top-left corner of the number followed by the size
//
// Returns -1 on error
int ReadNumber(int x, int y) {
    constexpr POINT sampleOffsets[] = {
        { 5,0 },
        { 5,8 },
        { 0,8 },
        { 10,8 },
        { 0,12 },
        { 5,12 },
        { 10,12 },
        { 0,7 },
        { 10,7 }
    };
    constexpr uint8_t threshold = 100; // Minimum brightness value of the pixel

    int guessBitflags = 0;
    {
        ScreenData screen = SCREEN_DATA.LockShared();
        for (int sample = 0; sample < 9; ++sample) {
            POINT samplePos {
                x + sampleOffsets[sample].x,
                y + sampleOffsets[sample].y
            };
            if (screen.GetPixelColor(samplePos).Gray() > threshold) {
                guessBitflags |= 1 << sample;
            }
        }
    }

    switch (guessBitflags) {
        case 0b110101101: return 0;
        case 0b000100011: return 1;
        case 0b001110011: return 2;
        case 0b000100001: return 3;
        case 0b010001110: return 4;
        case 0b100101001: return 5;
        case 0b010101101: return 6;
        case 0b000000011: return 7;
        case 0b000101101: return 8;
        case 0b100100001: return 9;
        default: return -1; // Error
    }
}

// Run this about once every frame
//
// Returns true on success, sets global time to 0 and returns false if a number failed to be read
bool ReadGameClock() {
    do {
        int time = ReadNumber(pnt::CLK_DECISEC_X, pnt::CLK_POS.y); // Deciseconds
        if (time == -1) break;
        int seconds = ReadNumber(pnt::CLK_SEC_X, pnt::CLK_POS.y); // Seconds (ones)
        if (seconds == -1) break;
        int tensOfSeconds = ReadNumber(pnt::CLK_10SEC_X, pnt::CLK_POS.y); // Seconds (tens)
        if (tensOfSeconds == -1) break;
        int minute = ReadNumber(pnt::CLK_POS.x, pnt::CLK_POS.y); // Minutes
        if (minute == -1) break;

        time = time + DECISECS_PER_SEC * (seconds + 10 * tensOfSeconds + SECS_PER_MIN * minute);

        GAME_STATE.gameData.time.UpdateTime(time);
        return true;
    } while (false);
    GAME_STATE.gameData.time.UpdateTime(0);
    return false;
}

bool DoesVentilationNeedReset() {
    return SCREEN_DATA
        .LockShared()
        .GetPixelColor(
            GAME_STATE.GetState() == State::Office
                ? pnt::ofc::VENT_WARNING_POS
                : pnt::cam::VENT_WARNING_POS
        ).RedDev() > 35;
}

void GenerateSamplePoints(POINT arr[5], POINT start, long scale) {
    arr[0] = start;
    arr[1] = { arr[0].x, arr[0].y + scale };
    arr[2] = { arr[0].x + scale, arr[0].y };
    arr[3] = { arr[0].x, arr[0].y - scale };
    arr[4] = { arr[0].x - scale, arr[0].y };
}

/// <summary>
///
/// </summary>
/// <param name="center">Point around which to generate the sample points</param>
/// <param name="compare">Normalized color against which to compare the color at the sample points</param>
/// <param name="threshold">0..1 double value for the minimum similarity required to consider a sample point a "match"</param>
/// <returns>Total number of sample points which exceeded the threshold</returns>
int TestSamples(POINT center, CNorm compare, double threshold) {
    POINT samplePoint[5];
    GenerateSamplePoints(samplePoint, center, 4);

    int matchCount = 0;
    ScreenData screen = SCREEN_DATA.LockShared();
    for (int i = 0; i < 5; ++i) {
        CNorm sample = screen.GetPixelColor(samplePoint[i]).Normalized();
        if (sample.VNormalized().CDot(compare) > threshold) ++matchCount;
    }
    return matchCount;
}

int TestSamples(Button button, CNorm compare, double threshold) {
    return TestSamples(GetButtonPos(button), compare, threshold);
}

int TestSamples(POINT center, Color compare, double threshold) {
    return TestSamples(center, compare.Normalized().VNormalized(), threshold);
}

int TestSamples(POINT center, uint8_t compare, uint8_t maxDifference) {
    POINT samplePoint[5];
    GenerateSamplePoints(samplePoint, center, 4);

    int matchCount = 0;
    ScreenData screen = SCREEN_DATA.LockShared();
    for (int i = 0; i < 5; ++i) {
        uint8_t sample = screen.GetPixelColor(samplePoint[i]).Gray();
        if (abs(sample - compare) > maxDifference) ++matchCount;
    }
    return matchCount;
}

// Returns the position of the maximum value
template<class I>
size_t MaxInArray(I begin, I end) {
    return std::distance(begin, std::max_element(begin, end));
}

// For finding the yaw of the office
void LocateOfficeLamp() {
    constexpr int y = 66;
    constexpr int threshold = 200;
    constexpr int start = 723;
    constexpr int width = 585;
    ScreenData screen = SCREEN_DATA.LockShared();
    for (int x = start; x < start + width; ++x) {
        if (screen.GetPixelColor(x, y).Gray() > threshold) {
            // 100% of the samples must be 80% matching. Flickering be damned.
            if (TestSamples({ x,y }, 255, 20) == 5) {
                OfficeData* od = GAME_STATE.GetOfficeData();
                assert(!!od);
                od->officeYaw = ((double)x - (double)start) / (double)width;
                break;
            }
        }
    }
}

void UpdateState() {
    constexpr double threshold = .99;
    State newState = State::Office;
    // List of how many samples returned as matches for each of the buttons being tested
    std::array<int, 3> statesToTest = { 0,0,0 };
    for (unsigned sysBtn = 0; sysBtn < statesToTest.size(); ++sysBtn) {
        statesToTest[sysBtn] = TestSamples(SystemButton(sysBtn), clr::SYS_BTN_COLOR_NRM, threshold);
    }
    size_t indexOfMax = MaxInArray(statesToTest.begin(), statesToTest.end());
    // We must have over 50% of the samples returning as matches
    if (statesToTest[indexOfMax] == 5) {
        newState = State(indexOfMax);
    }
    // Update the global state
    switch (newState) {
        case State::Office:
            GAME_STATE.SwitchToOffice(OfficeData { });
            break;

        case State::Camera: {
            std::array<int, 8> camsToTest = {};
            for (unsigned camera = 0; camera < camsToTest.size(); ++camera) {
                camsToTest[camera] = TestSamples(CameraButton(camera), clr::CAM_BTN_COLOR_NRM, threshold);
            }
            // If we've confirmed the state then there's no doubt we can identify the camera
            GAME_STATE.SwitchToCam(CamData {
                Camera(MaxInArray(camsToTest.begin(), camsToTest.end()))
            });
        }
        break;

        case State::Vent:
            GAME_STATE.SwitchToVent(VentData { });
            break;

        case State::Duct:
            GAME_STATE.SwitchToDuct(DuctData { });
            break;
    }
}

////////////////////////////////////////////////////
// This is where we send basic output to the game //
// e.g.                                           //
// - Press "d" key                                //
// - Move mouse to { 24, 36 }                     //
////////////////////////////////////////////////////

enum class VirtualKey : int {
    VK_W = 'W',
    FrontVent = VK_W,
    VK_A = 'A',
    LeftDoor = VK_A,
    VK_S = 'S',
    CameraToggle = VK_S,
    VK_D = 'D',
    RightDoor = VK_D,
    VK_F = 'F',
    RightVent = VK_F,
    VK_C = 'C',
    CatchFish = VK_C,
    VK_Enter = '\n',
    CloseAd = VK_Enter,
    VK_Space = ' ',
    DeskFan = VK_Space,
    VK_1 = '1',
    VK_2 = '2',
    VK_3 = '3',
    VK_4 = '4',
    VK_5 = '5',
    VK_6 = '6',
    VK_X = 'X',
    VK_Z = 'Z',
    Flashlight = VK_Z,
    Esc = '\x1b',
};

constexpr INPUT KeyInput(VirtualKey key, bool keyUp) {
    return {
        .type = INPUT_KEYBOARD,
        .ki = {
            .wVk = (WORD)key,
            .wScan = 0,
            .dwFlags = (DWORD)(keyUp ? KEYEVENTF_KEYUP : 0),
            .time = 0,
            .dwExtraInfo = 0,
        },
    };
}

enum class TranslateType : DWORD {
    Relative = 0,
    Absolute = MOUSEEVENTF_ABSOLUTE,
};

enum class M1State : DWORD {
    None = 0,
    Press = MOUSEEVENTF_LEFTDOWN,
    Release = MOUSEEVENTF_LEFTUP,
};

struct MouseMovement {
    long x;
    long y;
    TranslateType translation;
};

constexpr INPUT MouseInput(const MouseMovement* movement, M1State m1 = M1State::None) {
    return {
        .type = INPUT_MOUSE,
        .mi = {
            .dx = movement ? movement->x : 0,
            .dy = movement ? movement->y : 0,
            .mouseData = 0,
            .dwFlags = (movement ? MOUSEEVENTF_MOVE | (DWORD)movement->translation : 0) | (DWORD)m1,
            .time = 0, // Pleaseeeee don't mess with this... it makes the monitor go funky...
            .dwExtraInfo = 0,
        },
    };
}

void SimulateKeypress(VirtualKey key) {
    {
        INPUT input = KeyInput(key, false);
        SendInput(1, &input, sizeof(INPUT));
        Sleep(10);
    }
    {
        INPUT input = KeyInput(key, true);
        SendInput(1, &input, sizeof(INPUT));
        Sleep(2);
    }
}

POINT GetMousePos() {
    POINT p;
    if (GetCursorPos(&p)) return p;
    else return { 0,0 };
}
void SimulateMouseMove(long x, long y) {
    MouseMovement move = { x, y };
    INPUT input = MouseInput(&move);
    input.mi.dwFlags = MOUSEEVENTF_MOVE;
    SendInput(1, &input, sizeof(input));
}
void SimulateMouseMove(POINT p) {
    SimulateMouseMove(p.x, p.y);
}
void SimulateMouseGoto(long x, long y) {
    MouseMovement move = { x * 34, y * 61 };
    INPUT input = MouseInput(&move);
    input.mi.dwFlags |= MOUSEEVENTF_ABSOLUTE;
    SendInput(1, &input, sizeof(input));
}
void SimulateMouseGoto(POINT p) {
    SimulateMouseGoto(p.x, p.y);
}

void SimulateMouseClick() {
    {
        INPUT input = MouseInput(nullptr, M1State::Press);
        SendInput(1, &input, sizeof(input));
        Sleep(10);
    }
    {
        INPUT input = MouseInput(nullptr, M1State::Release);
        SendInput(1, &input, sizeof(INPUT));
    }
}
inline void SimulateMouseClickAt(POINT p) {
    SimulateMouseGoto(p);
    SimulateMouseClick();
}

// Assumes we are already in the office
void OfficeLookLeft() {
    assert(GAME_STATE.GetState() == State::Office); // We cannot look left/right in cameras
    SimulateMouseGoto(8, 540);
    Sleep(5 * MS_PER_DECISEC);
}

// Assumes we are already in the office
void OfficeLookRight() {
    assert(GAME_STATE.GetState() == State::Office); // We cannot look left/right in cameras
    SimulateMouseGoto(1910, 540);
    Sleep(5 * MS_PER_DECISEC);
}

///////////////////////////////////////////////////////////////////////////
// This is where basic outputs are combined to make more complex actions //
///////////////////////////////////////////////////////////////////////////

// Updates all known game information
void RefreshGameData() {
    UpdateState();
    ReadGameClock();
    if (DoesVentilationNeedReset()) {
        GAME_STATE.gameData.VentilationNeedsReset();
    }
    //LocateOfficeLamp(); // Needs work
}

void ToggleMonitor() {
    SimulateKeypress(VirtualKey::CameraToggle);
    Sleep(CAM_RESP_MS);
    UpdateState();
}

void OpenMonitorIfClosed() {
    if (GAME_STATE.GetState() == State::Office) {
        ToggleMonitor();
    }
}

void CloseMonitorIfOpen() {
    if (GAME_STATE.GetState() != State::Office) {
        ToggleMonitor();
    }
}

void EnsureSystem(State system) {
    OpenMonitorIfClosed();
    if (GAME_STATE.GetState() != system) {
        SimulateMouseClickAt(GetButtonPos(SystemButton(system)));
    }
}

void OpenCameraIfClosed() {
    EnsureSystem(State::Camera);
    Sleep(1); // In case the next step is another button press elsewhere
}

// `cam` only used if `state == State::Camera`
void EnterGameState(State state, Camera cam = Camera::WestHall) {
    if (GAME_STATE.GetState() != state) {
        if (state == State::Office) {
            CloseMonitorIfOpen();
        } else {
            OpenMonitorIfClosed();
        }
        switch (state) {
            case State::Office:
                break;

            case State::Camera:
                if (const CamData* cd = GAME_STATE.GetCamData(); !cd || cd->camera != cam) {
                    SimulateMouseClickAt(GetButtonPos(CameraButton(cam)));
                }
                break;

            case State::Duct:
                SimulateMouseClickAt(GetButtonPos(Button::DuctSystem));
                break;

            case State::Vent:
                SimulateMouseClickAt(GetButtonPos(Button::VentSystem));
                break;
        }
        Sleep(1);
    }
}

// Playbook of actions
namespace action {
    void HandleFuntimeFoxy() {
        OpenCameraIfClosed();
        SimulateMouseClickAt(GetButtonPos(Button::Cam06));
    }

    void ResetVents() {
        OpenMonitorIfClosed(); // We don't need to care which system, only that the monitor is up.
        SimulateMouseClickAt(GetButtonPos(Button::ResetVent));
        GAME_STATE.gameData.VentilationHasBeenReset();
        Sleep(10);
    }

    void HandleNMBB() {
        Sleep(17); // Wait a little bit to make sure we have time for the screen to change
        SCREEN_DATA.UpdateScreencap();
        if (IsNMBBStanding()) { // Double check--NMBB will kill us if we flash him wrongfully
            // If he is in fact still up, flash the light on him to put him back down
            SimulateKeypress(VirtualKey::Flashlight);
        }
    }
}

void ActOnGameData() {
    /**************************************************************************************************
     * Definitions
     * ===========
     * "Behavioral events" - Some things are not events so much as hints that our current behavior is
     *   unsustainable due to external data. When one of these occurs, we cannot simply 'handle' it
     *   and be done, and must change our behavioral pattern to better suit the needs of the event
     *   until the state has returned to neutral. Thankfully, the behavioral changes are usually
     *   transient and only require temporarily disabling certain systems.
     *
     * "Inturruption events" - Events which give us abrupt notice which we have only a short window to
     *   react to. We don't know ahead of time when these events will occur, and they can trigger
     *   automatically without intervention.
     *
     * "Transition events" - Events triggered by a change in gamestate (like opening or closing the
     *   monitor). These events usually aren't timed and can be done at leisure, but they limit the
     *   actions we can perform without handling them.
     *
     * "Timed events" - Events which are time-sensitive relative to the in-game clock or a countdown.
     *   These are usually long-term and while high-priority, can be done when convenient.
     *
     * "Transient events" - These are events which can be detected & reacted to without any dependence
     *   upon or changes to the current game state.
     *
     * "Distractor events" - Depending on the event, these events can be quick difficult to react to.
     *   They make it much harder to react to other events, and may even take away our control.
     *   Thankfully these events are usually either very short in duration or can be handled by rote.
     **************************************************************************************************/

    if (GAME_STATE.GetState() == State::Office) {
        if (IsNMBBStanding()) {
            action::HandleNMBB();
        }
    }

    if (GAME_STATE.gameData.DoesVentilationNeedReset()) {
        action::ResetVents();
    }

    // We have <= 1 seconds before the next hour
    if ((DECISECS_PER_HOUR - GAME_STATE.gameData.time.GetDecisecondsSinceHour()) <= (DECISECS_PER_SEC + (CAM_RESP_MS / MS_PER_DECISEC))) {
        action::HandleFuntimeFoxy();
        Sleep(10);
    }

    // Lowest priority actions should go down here //
}

void GameState::DisplayData() const {
    std::cout << RESET_CURSOR
        << "Time: " << gameData.time << '\n'
        << '\n'
        << "Ventilation: " << (gameData.DoesVentilationNeedReset() ? "WARNING" : "good   ") << '\n'
        << "  Left door: " << (gameData.IsDoorClosed(0) ? "closed" : "open  ") << '\n'
        << " Front vent: " << (gameData.IsDoorClosed(1) ? "closed" : "open  ") << '\n'
        << " Right door: " << (gameData.IsDoorClosed(2) ? "closed" : "open  ") << '\n'
        << " Right vent: " << (gameData.IsDoorClosed(3) ? "closed" : "open  ") << '\n'
        << " Flashlight: " << (gameData.IsFlashlightOn() ? "on " : "off") << '\n'
        << "Next Funtime Foxy show: " << gameData.nextFFShow << '\n'
        << '\n';

    std::cout << '<';
    for (const State s : { State::Camera, State::Vent, State::Duct, State::Office }) {
        const char* delim = (s == state) ? "[]" : "  ";
        std::cout << delim[0] << s << delim[1];
    }
    std::cout << ">\n";

    switch (state) {
        case State::Camera:
            std::cout
                << "Looking at: " << "CAM 0" << ((int)cd.camera + 1) << " | " << std::setw(18) << std::left << cd.camera << '\n';
            break;

        case State::Office:
            std::cout
                << "Yaw: " << od.officeYaw << '\n'
                << "Nightmare Balloon Boy: " << (IsNMBBStanding() ? "standing" : "sitting ") << '\n';
            break;

        default:
            std::cout << "TODO\n";
            break;
    }

    std::cout << '\n';
}

std::atomic<bool> THREADS_SHOULD_LOOP;

void Produce() {
    while (THREADS_SHOULD_LOOP.load()) {
        SCREEN_DATA.UpdateScreencap(); // Update our internal copy of what the gamescreen looks like so we can sample its pixels
        Sleep(2);
    }
}

void Consume() {
    while (THREADS_SHOULD_LOOP.load()) {
        RefreshGameData(); // Using the screencap we just generated, update the game data statuses for decision making
        if (!GAME_STATE.gameData.time.IsDefault()) {
            GAME_STATE.DisplayData(); // Output the data for the user to view
        } else {
            std::cout << RESET_CURSOR << "Waiting for clock to be visible...";
        }
        ActOnGameData(); // Based upon the game data, perform all actions necessary to return the game to a neutral state
        Sleep(4);
    }
}

void CreateHelpers() {
    SCREEN_DATA.UpdateScreencap(); // first time screen update
    THREADS_SHOULD_LOOP.store(true);
    std::thread producer(Produce); // Spawn a thread for reading the screen pixels
    std::thread consumer(Consume); // Spawn a thread for acting on that data

    // !! SAFETY !!
    // Make sure that user control override doesn't disable the user from closing the program
    while (THREADS_SHOULD_LOOP.load()) {
        Sleep(2); // Give the user time to provide input
        if (GetKeyState((int)VirtualKey::Esc) & ~1) { // mask to ignore the "toggled" bit
            std::cout.clear();
            std::cout << "\nUser has chosen to reclaim control. Task ended.\n";
            THREADS_SHOULD_LOOP.store(false); // This tells the worker threads to stop
        }
    }

    std::cout << "Waiting on worker threads...\n";
    // Wait for threads to safely finish their respective functions before destructing them
    producer.join();
    consumer.join();
    std::cout << "Worker threads joined.\n";
}

void SimulateKeyDown(VirtualKey key) {
    INPUT input = KeyInput(key, false);
    SendInput(1, &input, sizeof(INPUT));
    Sleep(2);
}

void SimulateKeyUp(VirtualKey key) {
    INPUT input = KeyInput(key, true);
    SendInput(1, &input, sizeof(INPUT));
    Sleep(2);
}

int main() {
    // SETUP //

    WND_CONSOLE = GetConsoleWindow(); // Get a console handle
    CONSOLE_HDC = GetDC(WND_CONSOLE); // Get a handle to device hdc

    SCREEN_HEIGHT = GetSystemMetrics(SM_CYVIRTUALSCREEN);
    SCREEN_WIDTH = GetSystemMetrics(SM_CXVIRTUALSCREEN);

    SCREEN_DATA.ResizeBuffer(CHANNELS_PER_COLOR * (size_t)SCREEN_WIDTH * (size_t)SCREEN_HEIGHT);

    DESKTOP_HDC = GetDC(NULL); // get the desktop device context
    INTERNAL_HDC = CreateCompatibleDC(DESKTOP_HDC); // create a device context to use ourselves

    H_BITMAP = CreateCompatibleBitmap(DESKTOP_HDC, SCREEN_WIDTH, SCREEN_HEIGHT);

    SelectObject(INTERNAL_HDC, H_BITMAP); // Get a handle to our bitmap

    // GAME LOOP //

    CreateHelpers();

    // WRAP UP //

    DeleteObject(H_BITMAP); // Free the bitmap memory to the OS

    DeleteDC(INTERNAL_HDC); // Destroy our internal display handle

    ReleaseDC(NULL, DESKTOP_HDC); // Free the desktop handle

    SCREEN_DATA.ResizeBuffer(0);

    return 0;
}
