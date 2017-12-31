using System;
using UnityEngine;
using System.Collections;
using System.Collections.Generic;

public class Game : MonoBehaviour
{
    public static Game instance;

    public CellColor current_color;
    public bool game_ended;

    void Awake()
    {
        instance = this;
    }

    void Start()
    {

        current_color = CellColor.Black;
        game_ended = false;

        StateManager.splash_state.OnGameInitSetup += InitGame;

        StateManager.in_game_state.OnEnter += EnterGame;
        StateManager.in_game_state.OnExit += ExitGame;

    }

    public void ReceivingLastMove(Cell cell)
    {
        Board.instance.PlaceStone(cell);

        List<Cell> winning_line;
        if (Gomoku.CheckForWin(cell, out winning_line))
        {
            foreach (Cell w_cell in winning_line)
            {
                w_cell.SetWinningPose();
            }
            Winning(current_color);
        }
        else
        {
            current_color = CellColorUtil.GetReverseCellColor(current_color);
            PlayerCardPanel.instance.PointToColor(current_color);
        }
        Pointer.instance.HardReset();
    }

    public void InitGame()
    {
        Board.instance.CreateCells();
        Pointer.instance.HardReset();
    }

    public void EnterGame()
    {
        current_color = CellColor.Black;
        game_ended = false;

        Board.instance.ShowBoard();
        
    }

    public void ExitGame()
    {
        Board.instance.HideBoard();
        Pointer.instance.SoftReset();
    }

    public void Winning(CellColor color)
    {
        PlayerCard w_player = PlayerCardPanel.instance.FindCardByColor(color);
        w_player.player_profile.AddLastGame(GameResult.Win);
        w_player.ResyncWithProfile();

        PlayerCard l_player = PlayerCardPanel.instance.FindCardByColor(CellColorUtil.GetReverseCellColor(color));
        l_player.player_profile.AddLastGame(GameResult.Loss);
        l_player.ResyncWithProfile();

        game_ended = true;
        current_color = CellColor.Empty;
    }

    public void GameReset()
    {
        Board.instance.ResetBoard();

        if (game_ended)
        {
            PlayerCardPanel.instance.SwitchPlayerColor();
        }

        PlayerCardPanel.instance.PointToColor(CellColor.Black);

        game_ended = false;
        current_color = CellColor.Black;

        Pointer.instance.SoftReset();
    }
}