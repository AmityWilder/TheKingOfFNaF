#ifndef GAME_ACTIONS_H
#define GAME_ACTIONS_H
#include "Globals.h"
#include "InputProcessing.h"
#include "Output.h"

///////////////////////////////////////////////////////////////////////////
// This is where basic outputs are combined to make more complex actions //
///////////////////////////////////////////////////////////////////////////

void OfficeLookLeft();
void OfficeLookRight();

// Updates all known game information
void RefreshGameData();

void EnsureSystem(State state);
void OpenCameraIfClosed();
void OpenMonitorIfClosed();
void CloseMonitorIfOpen();
void EnterGameState(State state, Camera cam = Camera::WestHall);

// Playbook of actions
namespace action {
    void HandleFuntimeFoxy();
    void ResetVents();
    void HandleNMBB();
}

void ActOnGameData();

#endif // GAME_ACTIONS_H
