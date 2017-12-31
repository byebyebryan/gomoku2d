using System;
using UnityEngine;
using System.Collections;
using System.Collections.Generic;


[Serializable]
public class SaveData
{
    public PlayerProfile[] player_profiles;
    public int last_black_player_profile_index;
    public int last_white_player_profile_index;

    public SaveData()
    {
        player_profiles = new PlayerProfile[5];
        for (int i = 0; i < 5; i++)
        {
            PlayerProfile profile = new PlayerProfile();
            profile.id = i;
            profile.name = "Player " + (i + 1).ToString();
            player_profiles[i] = profile;
        }
        last_black_player_profile_index = 0;
        last_white_player_profile_index = 1;
    }

    public PlayerProfile GetLastBlackPlayerProfile()
    {
        return player_profiles[last_black_player_profile_index];
    }

    public PlayerProfile GetLastWhitePlayerProfile()
    {
        return player_profiles[last_white_player_profile_index];
    }
}
