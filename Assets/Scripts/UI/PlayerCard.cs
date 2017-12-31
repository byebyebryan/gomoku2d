using UnityEngine;
using System.Collections;
using UnityEngine.UI;

public class PlayerCard : MonoBehaviour
{

    public CellColor cell_color
    {
        get { return color_changer.cell_color; }

        set { color_changer.cell_color = value; }
    }

    public UIColorChanger color_changer;

    public PlayerProfile player_profile;

    public Button2 button;

    public InputField name_input;

    public Text name_text;
    public Text total_stats_text;
    public Text session_stats_text;

    public void SetProfile(PlayerProfile profile)
    {
        player_profile = profile;
        if (name_input != null)
        {
            name_input.text = profile.name;
        }
        else
        {
            name_text.text = profile.name;
        }
        
        if (total_stats_text)
        {
            total_stats_text.text = "Total W " + player_profile.total_win_count + " L " + player_profile.total_loss_count;
        }
        if (session_stats_text)
        {
            session_stats_text.text = "Latest W " + player_profile.LatestWinCount() + " L " +
                                      player_profile.LatestLossCount();
        }
    }

    void Awake()
    {
        button = GetComponent<Button2>();

        color_changer = GetComponent<UIColorChanger>();
        name_input = GetComponent<InputField>();

        name_text = transform.Find("Name").GetComponent<Text>();
        Transform stats = transform.Find("Stats_T");
        if (stats)
        {
            total_stats_text = stats.GetComponent<Text>();
        }

        stats = transform.Find("Stats_S");
        if (stats)
        {
            session_stats_text = stats.GetComponent<Text>();
        }
    }

    public void NameChanged(string text)
    {
        player_profile.name = name_input.text;
        PlayerProfilePanel.instance.profile_cards[player_profile.id].SetProfile(player_profile);
        name_input.DeactivateInputField();
        name_input.interactable = false;
    }

    public void ResyncWithProfile()
    {
        SetProfile(player_profile);
        PlayerProfilePanel.instance.profile_cards[player_profile.id].SetProfile(player_profile);
    }
}
