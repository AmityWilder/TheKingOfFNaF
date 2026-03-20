#ifndef INPUT_PROCESSING_H
#define INPUT_PROCESSING_H
#include "Input.h"
#include <algorithm>

////////////////////////////////////////////////////////////////////////////////////
// This is where the input we've taken from the game gets turned into useful data //
////////////////////////////////////////////////////////////////////////////////////

// Input should be top-left corner of the number followed by the size
char ReadNumber(int x, int y);

// Run this about once every frame
void ReadGameClock();

void CheckVentsReset();

void GenerateSamplePoints(POINT arr[5], POINT start, long size);

int TestSamples(POINT center, CNorm compare, double threshold);
int TestSamples(Button button, CNorm compare, double threshold);

int TestSamples(POINT center, Color compare, double threshold);

int TestSamples(POINT center, uint8_t compare, uint8_t maxDifference);

// Returns the position of the maximum value
template<class I>
size_t MaxInArray(I begin, I end) {
    return std::distance(begin, std::max_element(begin, end));
}

void LocateOfficeLamp(); // For finding the yaw of the office

bool IsNMBBStanding();

void UpdateState();

#endif // INPUT_PROCESSING_H
