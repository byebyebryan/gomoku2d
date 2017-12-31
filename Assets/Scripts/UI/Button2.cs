using UnityEngine;
using System.Collections.Generic;
using UnityEngine.EventSystems;
using UnityEngine.UI;

public class Button2 : MonoBehaviour, IPointerEnterHandler, IPointerExitHandler, IPointerDownHandler, IPointerUpHandler, IPointerClickHandler
{
    public delegate void ClickHandler();

    public Image image;

    private List<RectTransform> children_rects;
    private List<Vector2> original_positions;

    public bool is_available = true;
    public bool is_toggled = false;
    public bool toggle_override = false;

    public ToggleGroup toggle_group;
    public int toggle_index;

    public event ClickHandler OnClick;

    public void ResetButton()
    {
        toggle_override = false;
        UnToggle();
    }

    public void Toggle()
    {
        if (toggle_override)
        {
            return;
        }

        image.sprite = EditorData.instance.button_pressed_sprite;

        for (int i = 0; i < children_rects.Count; i++)
        {
            children_rects[i].anchoredPosition = original_positions[i] + Vector2.down * EditorData.instance.button_pressed_offset;
        }

        is_toggled = true;
    }

    public void UnToggle()
    {
        if (toggle_override)
        {
            return;
        }

        image.sprite = EditorData.instance.button_normal_sprite;

        for (int i = 0; i < children_rects.Count; i++)
        {
            children_rects[i].anchoredPosition = original_positions[i];
        }

        is_toggled = false;
    }

    void Awake()
    {
        image = GetComponent<Image>();
        children_rects = new List<RectTransform>();
        original_positions = new List<Vector2>();
    }

	// Use this for initialization
	void Start () {
        for (int i = 0; i < transform.childCount; i++)
        {
            children_rects.Add(transform.GetChild(i).GetComponent<RectTransform>());
            original_positions.Add(children_rects[i].anchoredPosition);
        }
    }

    public void OnPointerEnter(PointerEventData eventData)
    {
        if (is_available && !is_toggled && !toggle_override)
        {
            image.sprite = EditorData.instance.button_hover_sprite;

            for (int i = 0; i < children_rects.Count; i++)
            {
                children_rects[i].anchoredPosition = original_positions[i] + Vector2.down * EditorData.instance.button_hover_offset;
            }
        }
    }

    public void OnPointerExit(PointerEventData eventData)
    {
        if (is_available && !is_toggled && !toggle_override)
        {
            image.sprite = EditorData.instance.button_normal_sprite;

            for (int i = 0; i < children_rects.Count; i++)
            {
                children_rects[i].anchoredPosition = original_positions[i];
            }
        }
    }

    public void OnPointerDown(PointerEventData eventData)
    {
        if (is_available && !is_toggled && !toggle_override)
        {
            image.sprite = EditorData.instance.button_pressed_sprite;

            for (int i = 0; i < children_rects.Count; i++)
            {
                children_rects[i].anchoredPosition = original_positions[i] + Vector2.down * EditorData.instance.button_pressed_offset;
            }
        }
    }

    public void OnPointerUp(PointerEventData eventData)
    {
        if (is_available && !is_toggled && !toggle_override)
        {
            image.sprite = EditorData.instance.button_normal_sprite;

            for (int i = 0; i < children_rects.Count; i++)
            {
                children_rects[i].anchoredPosition = original_positions[i];
            }
        }
    }

    public void OnPointerClick(PointerEventData eventData)
    {
        if (is_available && !is_toggled && !toggle_override)
        {
            if (OnClick != null)
            {
                OnClick();
            }
            if (toggle_group != null)
            {
                toggle_group.ReceiveToggle(this);
            }
            
        }
    }
}
