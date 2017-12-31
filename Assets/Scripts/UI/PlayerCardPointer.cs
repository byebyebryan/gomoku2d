using UnityEngine;
using System.Collections;
using UnityEngine.UI;

public class PlayerCardPointer : MonoBehaviour
{

    public PlayerCard target_card;

    public Image image;
    public UIMover mover;
    public UIColorChanger color_changer;

    public bool is_available;

    public void ForceShow()
    {
        is_available = true;
        image.enabled = true;
        mover.enabled = true;
        color_changer.enabled = true;
    }

    public void ForceHide()
    {
        is_available = false;
        image.enabled = false;
        mover.enabled = false;
        color_changer.enabled = false;
    }

    void Awake()
    {
        image = GetComponent<Image>();
        mover = GetComponent<UIMover>();
        color_changer = GetComponent<UIColorChanger>();
    }

    public void SetTargetCard(PlayerCard target)
    {
        target_card = target;
        if (is_available)
        {
            color_changer.cell_color = target.cell_color;
            mover.target = target.GetComponent<RectTransform>();
        }
    }
}
