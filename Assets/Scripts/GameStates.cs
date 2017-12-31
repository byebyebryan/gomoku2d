using UnityEngine;
using System.Collections;

public enum GameState
{
    NullState, Splash, MainMenu, InGame, GameEnd, GameReset
}

public abstract class GameStateBase
{
    public abstract GameState State { get; }

    public delegate void StaticEventHandler();

    public delegate void UpdateEventHandler(float delta_time);

    public event StaticEventHandler OnEnter;
    public event UpdateEventHandler OnUpdate;
    public event StaticEventHandler OnExit;

    public virtual void Enter()
    {
        if (OnEnter != null)
        {
            OnEnter();
        }
    }

    public virtual void Update(float delta_time)
    {
        if (OnUpdate != null)
        {
            OnUpdate(delta_time);
        }
    }

    public virtual void Exit()
    {
        if (OnExit != null)
        {
            OnExit();
        }
    }
}

public class SplashState : GameStateBase
{
    public override GameState State
    {
        get { return GameState.Splash; }
    }

    public event StaticEventHandler OnGameInitSetup;

    private float timer;
    private bool init_trigger;

    public override void Enter()
    {
        timer = EditorData.instance.splash_screen_time;
        init_trigger = false;

        base.Enter();
    }

    public override void Update(float delta_time)
    {
        if (timer > 0)
        {
            if (!init_trigger)
            {
                if (OnGameInitSetup != null)
                {
                    OnGameInitSetup();
                }
                init_trigger = true;
            }

            timer -= delta_time;
            if (timer <= 0)
            {
                StateManager.instance.ChangeState(StateManager.main_menu_state);
            }
        }

        base.Update(delta_time);
    }
}

public class MainMenuState : GameStateBase
{
    public override GameState State
    {
        get { return GameState.MainMenu; }
    }
}

public class InGameState : GameStateBase
{
    public override GameState State
    {
        get { return GameState.InGame; }
    }
}