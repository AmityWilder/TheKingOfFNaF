#pragma once
#include "Input.h"

////////////////////////////////////////////////////////////////////////////////////
// This is where the input we've taken from the game gets turned into useful data //
////////////////////////////////////////////////////////////////////////////////////

// Input should be top-left corner of the number followed by the size
char ReadNumber(int x, int y);

// Run this about once every frame
void ReadGameClock();

void CheckVentsReset();

void GenerateSamplePoints(POINT arr[5], POINT start, long size);

int TestSamples_CNormMethod(POINT center, CNorm compare, double threshold);
int TestSamples_CNormMethod(Button button, CNorm compare, double threshold);

int TestSamples_ColorMethod(POINT center, Color compare, double threshold);

int TestSamples_GrayMethod(POINT center, uint8_t compare, uint8_t maxDifference);

// Returns the position of the maximum value
template<class I>
size_t MaxInArray(I begin, I end) {
    return std::distance(begin, std::max_element(begin, end))
}

void LocateOfficeLamp(); // For finding the yaw of the office

bool IsNMBBStanding();

void UpdateState();
