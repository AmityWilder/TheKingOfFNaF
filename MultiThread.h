#pragma once
#include "InputProcessing.h"
#include "GameActions.h"
#include <thread>

extern bool THREADS_SHOULD_LOOP;
extern bool FIRST_TIME_SCREEN_UPDATE;

void Produce();

void Consume();

void Direct();

void CreateHelpers();
