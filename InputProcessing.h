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

void GenerateSamplePoints(POINT* arr, POINT start, long size);

int TestSamples_CNormMethod(POINT center, CNorm compare, double threshold);
int TestSamples_CNormMethod(Button button, CNorm compare, double threshold);

int TestSamples_ColorMethod(POINT center, Color compare, double threshold);

int TestSamples_GrayMethod(POINT center, unsigned char compare, unsigned char maxDifference);

int MaxInArray(int* arr, size_t size); // Returns the index of the highest value

void LocateOfficeLamp(); // For finding the yaw of the office

bool CheckOnNMBB();

void UpdateState();
