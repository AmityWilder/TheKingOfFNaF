#include "InputProcessing.h"
#include <array>
#include <algorithm>
#include <cassert>

// Input should be top-left corner of the number followed by the size
char ReadNumber(int x, int y) {
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
    for (int sample = 0; sample < 9; ++sample) {
        POINT samplePos {
            x + sampleOffsets[sample].x,
            y + sampleOffsets[sample].y
        };
        if (GetPixelColor(samplePos).Gray() > threshold) {
            guessBitflags |= 1 << sample;
        }
    }

    switch (guessBitflags) {
        default: // Error returns zero
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
    }
}

// Run this about once every frame
void ReadGameClock() {
    int time = (int)ReadNumber(pnt::CLK_DECISEC_X, pnt::CLK_POS.y); // Deciseconds
    int seconds = (int)ReadNumber(pnt::CLK_SEC_X, pnt::CLK_POS.y); // Seconds (ones)
    int tensOfSeconds = (int)ReadNumber(pnt::CLK_10SEC_X, pnt::CLK_POS.y); // Seconds (tens)
    int minute = (int)ReadNumber(pnt::CLK_POS.x, pnt::CLK_POS.y); // Minutes

    time = time + DECISECS_PER_SEC * (seconds + 10 * tensOfSeconds + SECS_PER_MIN * minute);

    GAME_STATE.gameData.time.UpdateTime(time);
}

void CheckVentsReset() {
    if (GetPixelColor(pnt::ofc::VENT_WARNING_POS).RedDev() > 35 ||
        GetPixelColor(pnt::cam::VENT_WARNING_POS).RedDev() > 35)
    {
        GAME_STATE.gameData.VentilationNeedsReset();
    }
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
int TestSamples_CNormMethod(POINT center, CNorm compare, double threshold) {
    POINT samplePoint[5];
    GenerateSamplePoints(samplePoint, center, 4);

    int matchCount = 0;
    for (int i = 0; i < 5; ++i) {
        CNorm sample = GetPixelColor(samplePoint[i]).Normalized();
        if (sample.CDot(compare) > threshold) ++matchCount;
    }
    return matchCount;
}

int TestSamples_CNormMethod(Button button, CNorm compare, double threshold) {
    return TestSamples_CNormMethod(GetButtonPos(button), compare, threshold);
}

int TestSamples_ColorMethod(POINT center, Color compare, double threshold) {
    /* Ok I need to explain this because it's a little weird.
    I know it looks like it's just extra steps being added on to the CNorm method, but it's not.
    The difference is that the colors in this method, while converted to [0..1], are not normalized.
    It does make a difference.*/

    CNorm compareVec = compare.Normalized();

    POINT samplePoint[5];
    GenerateSamplePoints(samplePoint, center, 4);

    int matchCount = 0;
    for (int i = 0; i < 5; ++i) {
        Color sample = GetPixelColor(samplePoint[i]);
        CNorm sampleVec = sample.Normalized();

        if (sampleVec.CDot(compareVec) > threshold) ++matchCount;
    }
    return matchCount;
}

int TestSamples_GrayMethod(POINT center, uint8_t compare, uint8_t maxDifference) {
    POINT samplePoint[5];
    GenerateSamplePoints(samplePoint, center, 4);

    int matchCount = 0;
    for (int i = 0; i < 5; ++i) {
        uint8_t sample = GetPixelColor(samplePoint[i]).Gray();
        if (abs(sample - compare) > maxDifference) ++matchCount;
    }
    return matchCount;
}

void LocateOfficeLamp() {
    constexpr int y = 66;
    constexpr int threshold = 200;
    constexpr int start = 723;
    constexpr int width = 585;
    for (int x = start; x < start + width; ++x) {
        if (GetPixelColor(x, y).Gray() > threshold) {
            // 100% of the samples must be 80% matching. Flickering be damned.
            if (TestSamples_GrayMethod({ x,y }, 255, 20) == 5) {
                OfficeData* od = GAME_STATE.GetOfficeData();
                assert(!!od);
                od->officeYaw = ((double)x - (double)start) / (double)width;
                break;
            }
        }
    }
}

bool IsNMBBStanding() {
    constexpr Color PANTS_COLOR = { 0, 28, 120 };
    constexpr POINT SAMPLE_POS = { 1024, 774 };
    constexpr double THRESHOLD = 0.98;
    return (PANTS_COLOR.CDot(GetPixelColor(SAMPLE_POS)) > THRESHOLD);
}

void UpdateState() {
    constexpr double threshold = .99;
    State newState = State::Office;
    // List of how many samples returned as matches for each of the buttons being tested
    std::array<int, 3> statesToTest = { 0,0,0 };
    for (unsigned sysBtn = 0; sysBtn < statesToTest.size(); ++sysBtn) {
        statesToTest[sysBtn] = TestSamples_CNormMethod(SystemButton(sysBtn), clr::SYS_BTN_COLOR_NRM, threshold);
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
                camsToTest[camera] = TestSamples_CNormMethod(CameraButton(camera), clr::CAM_BTN_COLOR_NRM, threshold);
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
