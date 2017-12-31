using System;
using UnityEngine;
using System.Collections;
using System.Collections.Generic;
using UnityEngine.UI;

public enum GameResult
{
    Loss = 0, Draw = 1, Win = 2
}

[Serializable]
public class PlayerProfile
{
    public int id;
    public string name;
    public int total_game_count;
    public int total_win_count;
    public int total_loss_count;

    public List<GameResult> latest_games;
    public List<GameResult> current_games;

    public PlayerProfile()
    {
        id = 0;
        name = "New Player";
        total_game_count = 0;
        total_win_count = 0;
        total_loss_count = 0;

        latest_games = new List<GameResult>();
        current_games = new List<GameResult>();
    }

    public void Clear()
    {
        total_game_count = 0;
        total_win_count = 0;
        total_loss_count = 0;

        latest_games = new List<GameResult>();
        current_games = new List<GameResult>();
    }

    public float TotalWinRate()
    {
        if (total_game_count == 0)
        {
            return 0;
        }
        else
        {
            return (float)total_win_count / total_game_count;
        }
    }

    public int LatestWinCount()
    {
        if (latest_games.Count == 0)
        {
            return 0;
        }
        else
        {
            int win_count = 0;
            foreach (GameResult result in latest_games)
            {
                if (result == GameResult.Win)
                {
                    win_count++;
                }
            }
            return win_count;
        }
    }

    public int LatestLossCount()
    {
        if (latest_games.Count == 0)
        {
            return 0;
        }
        else
        {
            return latest_games.Count - LatestWinCount();
        }
    }

    public float LatestWinRate()
    {
        if (latest_games.Count == 0)
        {
            return 0;
        }
        else
        {
            return (float)LatestWinCount() / latest_games.Count;
        }
    }

    public int CurrentWinCount()
    {
        if (current_games.Count == 0)
        {
            return 0;
        }
        else
        {
            int win_count = 0;
            foreach (GameResult result in current_games)
            {
                if (result == GameResult.Win)
                {
                    win_count++;
                }
            }
            return win_count;
        }
    }

    public int CurrentLossCount()
    {
        if (current_games.Count == 0)
        {
            return 0;
        }
        else
        {
            return current_games.Count - CurrentWinCount();
        }
    }

    public float CurrentWinRate()
    {
        if (current_games.Count == 0)
        {
            return 0;
        }
        else
        {
            return (float)CurrentWinCount() / current_games.Count;
        }
    }

    public void AddLastGame(GameResult result)
    {
        total_game_count ++;
        if (result == GameResult.Win)
        {
            total_win_count ++;
        }
        else if (result == GameResult.Loss)
        {
            total_loss_count ++;
        }
        UpdateLatestGames(result);
        UpdateCurrentGames(result);
    }

    void UpdateLatestGames(GameResult result)
    {
        if (latest_games.Count == 10)
        {
            latest_games.RemoveAt(0);
        }
        latest_games.Add(result);
    }

    void UpdateCurrentGames(GameResult result)
    {
        current_games.Add(result);
    }
    
}
