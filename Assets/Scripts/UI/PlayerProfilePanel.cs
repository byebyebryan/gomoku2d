using UnityEngine;
using System.Collections.Generic;

public class PlayerProfilePanel : MonoBehaviour
{
    public static PlayerProfilePanel instance;

    public List<PlayerCard> profile_cards;
    public ToggleGroup toggle_group;

    public CellColor cell_color;

    public PlayerCard toggled_card;

    public void SyncProfiles()
    {
        for (int i = 0; i < profile_cards.Count; i++)
        {
            profile_cards[i].SetProfile(SaveDataManager.instance.save_data.player_profiles[i]);
        }
    }

    public void InitFromPlayerPanel()
    {
        foreach (PlayerCard card in profile_cards)
        {
            card.button.ResetButton();
        }

        PlayerCard white_card = profile_cards[PlayerCardPanel.instance.player_cards[1].player_profile.id];
        white_card.cell_color = CellColor.White;
        white_card.button.Toggle();
        white_card.button.toggle_override = true;

        ChangeColor(CellColor.Black);
        PlayerCard black_card = profile_cards[PlayerCardPanel.instance.player_cards[0].player_profile.id];
        black_card.button.Toggle();
        toggled_card = black_card;
    }

    public void SwitchPlayer()
    {
        if (toggled_card != null)
        {
            toggled_card.button.toggle_override = true;
        }
        PlayerCard card = profile_cards[PlayerCardPanel.instance.target_card.player_profile.id];
        card.button.toggle_override = false;
        toggle_group.DirectToggle(card.player_profile.id);
    }

    public void ChangeColor(CellColor color)
    {
        foreach (PlayerCard card in profile_cards)
        {
            if (!card.button.toggle_override)
            {
                card.cell_color = color;
            }
        }
    }

    public void OnSelectedProfile(int index)
    {
        PlayerCardPanel.instance.target_card.SetProfile(profile_cards[index].player_profile);
        toggled_card = profile_cards[index];
    }

    void Awake()
    {
        instance = this;

        toggle_group = GetComponent<ToggleGroup>();

        profile_cards = new List<PlayerCard>();
    }

    // Use this for initialization
    void Start () {
        toggle_group.OnSelectionChange += OnSelectedProfile;

        foreach (Button2 button in toggle_group.buttons)
        {
            PlayerCard card = button.GetComponent<PlayerCard>();
            profile_cards.Add(card);
        }

        cell_color = CellColor.Black;
    }
}
