using UnityEngine;
using System.Collections;
using System.Collections.Generic;
using UnityEngine.UI;

public class PlayerCardPanel : MonoBehaviour
{
    public static PlayerCardPanel instance;

    public List<PlayerCard> player_cards;
    public ToggleGroup toggle_group;

    public PlayerCardPointer pointer;

    public PlayerCard target_card
    {
        get { return pointer.target_card; }
        set { pointer.SetTargetCard(value);}
    }

    public void OnSelectedPlayer(int index)
    {
        pointer.SetTargetCard(player_cards[index]);
        PlayerProfilePanel.instance.SwitchPlayer();
        PlayerProfilePanel.instance.ChangeColor(player_cards[index].cell_color);
    }

    public void UpdateLatestProfileSelection()
    {
        SaveDataManager.instance.save_data.last_black_player_profile_index = FindCardByColor(CellColor.Black).player_profile.id;
        SaveDataManager.instance.save_data.last_white_player_profile_index = FindCardByColor(CellColor.White).player_profile.id;
    }

    public void InitAfterLoad()
    {
        player_cards[0].SetProfile(SaveDataManager.instance.save_data.GetLastBlackPlayerProfile());
        player_cards[1].SetProfile(SaveDataManager.instance.save_data.GetLastWhitePlayerProfile());
        ResetPlayerColor();
    }

    public void InitForMainMenu()
    {
        pointer.ForceHide();
        toggle_group.EnableButtons();
        ResetPlayerColor();
        player_cards[0].button.Toggle();
    }

    public void InitForInGame()
    {
        pointer.ForceShow();
        toggle_group.DisableButtons();
        ResetPlayerColor();
        UpdateLatestProfileSelection();
        PointToColor(CellColor.Black);
    }

    public PlayerCard FindCardByColor(CellColor color)
    {
        foreach (PlayerCard card in player_cards)
        {
            if (card.cell_color == color)
            {
                return card;
            }
        }
        return null;
    }

    public void PointToColor(CellColor color)
    {
        pointer.SetTargetCard(FindCardByColor(color));
    }

    public void ResetPlayerColor()
    {
        player_cards[0].cell_color = CellColor.Black;
        player_cards[1].cell_color = CellColor.White;
        pointer.target_card = player_cards[0];
    }

    public void SwitchPlayerColor()
    {
        foreach (PlayerCard card in player_cards)
        {
            card.cell_color = CellColorUtil.GetReverseCellColor(card.cell_color);
        }
    }

    void Awake()
    {
        instance = this;

        toggle_group = GetComponent<ToggleGroup>();
        pointer = GetComponentInChildren<PlayerCardPointer>();

        player_cards = new List<PlayerCard>();
    }

    void Start()
    {
        toggle_group.OnSelectionChange += OnSelectedPlayer;

        foreach (Button2 button in toggle_group.buttons)
        {
            PlayerCard card = button.GetComponent<PlayerCard>();
            player_cards.Add(card);
        }
    }

}
