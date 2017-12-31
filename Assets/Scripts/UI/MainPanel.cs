using UnityEngine;
using System.Collections;
using UnityEngine.UI;

public class MainPanel : MonoBehaviour
{

    public static MainPanel instance;

    public UIMover splash_mover;

    public UIMover main_panel_mover;
    public UIMover in_game_ctrl_mover;
    public UIMover player_edit_ctrl_mover;

    public Button2 reset_button;
    public Button2 menu_button;
    public Button2 edit_button;
    public Button2 clear_button;
    public Button2 start_button;
    public Button2 quit_button;

    void Awake()
    {
        instance = this;
        main_panel_mover = GetComponent<UIMover>();
    }

    void Start()
    {
        StateManager.splash_state.OnGameInitSetup += InitMenu;
        StateManager.main_menu_state.OnEnter += EnterMenu;
        StateManager.main_menu_state.OnExit += ExitMenu;

        reset_button.OnClick += ResetButtonPressed;
        menu_button.OnClick += MenuButtonPressed;
        edit_button.OnClick += EditButtonPressed;
        clear_button.OnClick += ClearButtonPressed;
        start_button.OnClick += StartButtonPressed;
        quit_button.OnClick += QuitButtonPressed;

        splash_mover.ForceRestingPosition();
        splash_mover.SetActivePosition();
        main_panel_mover.ForceRestingPosition();
    }

    public void InitMenu()
    {
        SaveDataManager.instance.Load();
        PlayerCardPanel.instance.InitAfterLoad();
        PlayerProfilePanel.instance.SyncProfiles();
        PlayerProfilePanel.instance.InitFromPlayerPanel();
    }

    public void EnterMenu()
    {
        splash_mover.SetAltPosition();
        splash_mover.lerp_speed = EditorData.instance.ui_move_lerp_speed;
        main_panel_mover.SetActivePosition();
        PlayerCardPanel.instance.InitForMainMenu();
        PlayerProfilePanel.instance.InitFromPlayerPanel();
        in_game_ctrl_mover.SetRestingPosition();
        player_edit_ctrl_mover.SetActivePosition();
    }

    public void ExitMenu()
    {
        main_panel_mover.SetAltPosition();
        PlayerCardPanel.instance.InitForInGame();
        in_game_ctrl_mover.SetActivePosition();
        player_edit_ctrl_mover.SetRestingPosition();
    }

    public void ResetButtonPressed()
    {
        Game.instance.GameReset();
    }

    public void MenuButtonPressed()
    {
        StateManager.instance.ChangeState(StateManager.main_menu_state);
    }

    public void EditButtonPressed()
    {
        PlayerCardPanel.instance.target_card.name_input.interactable = true;
        PlayerCardPanel.instance.target_card.name_input.ActivateInputField();
    }

    public void ClearButtonPressed()
    {
        PlayerCardPanel.instance.target_card.player_profile.Clear();
        PlayerCardPanel.instance.target_card.ResyncWithProfile();
    }

    public void StartButtonPressed()
    {
        StateManager.instance.ChangeState(StateManager.in_game_state);
    }

    public void QuitButtonPressed()
    {
        Application.Quit();
    }
}
