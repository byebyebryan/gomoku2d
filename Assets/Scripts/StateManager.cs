using UnityEngine;
using System.Collections;

public class StateManager : MonoBehaviour
{
    public static StateManager instance;
    public static SplashState splash_state;
    public static MainMenuState main_menu_state;
    public static InGameState in_game_state;

    public GameStateBase current_state_runnable;

    public GameState current_game_state
    {
        get { return current_state_runnable != null ? current_state_runnable.State : GameState.NullState; }
    }

    public void ChangeState(GameStateBase next_state)
    {
        if(current_state_runnable != null)
        { current_state_runnable.Exit();}

        current_state_runnable = next_state;

        current_state_runnable.Enter();
    }

    void Awake()
    {
        instance = this;
        splash_state = new SplashState();
        main_menu_state = new MainMenuState();
        in_game_state = new InGameState();
    }

	// Use this for initialization
	void Start () {
	    ChangeState(splash_state);
	}
	
	// Update is called once per frame
	void Update () {
	    if (current_state_runnable != null)
	    {
            current_state_runnable.Update(Time.deltaTime);
	    }
	}
}
