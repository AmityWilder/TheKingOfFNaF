#include "GameActions.h"
#include "Globals.h"

// Updates all known game information
void RefreshGameData() {
    UpdateState();
    ReadGameClock();
    CheckVentsReset();
    //LocateOfficeLamp(); // Needs work
}

void ToggleMonitor() {
    SimulateKeypress(VirtualKey::CameraToggle);
    Sleep(CAM_RESP_MS);
    UpdateState();
}

void OpenMonitorIfClosed() {
    if (GAME_STATE.state == State::Office) {
        ToggleMonitor();
    }
}

void CloseMonitorIfOpen() {
    if (GAME_STATE.state != State::Office) {
        ToggleMonitor();
    }
}

void EnsureSystem(State system) {
    OpenMonitorIfClosed();
    if (GAME_STATE.state != system) {
        SimulateMouseClickAt(GetButtonPos(SystemButton(system)));
    }
}

void OpenCameraIfClosed() {
    EnsureSystem(State::Camera);
    Sleep(1); // In case the next step is another button press elsewhere
}

// `cam` only used if `state == State::Camera`
void EnterGameState(State state, Camera cam) {
    if (GAME_STATE.state != state) {
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
        UpdateScreencap();
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

    if (GAME_STATE.state == State::Office) {
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
