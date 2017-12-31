using UnityEngine;
using System.Collections.Generic;
using UnityEngine.UI;

public class ToggleGroup : MonoBehaviour
{
    public delegate void SelectionHandler(int index);

    public List<Button2> buttons;
    public Button2 toggled_button;

    public event SelectionHandler OnSelectionChange;

    void Awake()
    {
        int i = 0;
        foreach (Button2 button in GetComponentsInChildren<Button2>())
        {
            button.toggle_index = i;
            i++;
            button.toggle_group = this;
            buttons.Add(button);
        }
    }

    public void DisableButtons()
    {
        foreach (Button2 button in buttons)
        {
            button.UnToggle();
            button.is_available = false;
        }
    }

    public void EnableButtons()
    {
        foreach (Button2 button in buttons)
        {
            button.UnToggle();
            button.is_available = true;
        }
    }

    public void ReceiveToggle(Button2 button)
    {
        toggled_button = button;
        for (int i = 0; i < buttons.Count; i++)
        {
            if (buttons[i] == button)
            {
                buttons[i].Toggle();
            }
            else
            {

                buttons[i].UnToggle(); 
            }
        }

        if (OnSelectionChange != null)
        {
            OnSelectionChange(button.toggle_index);
        }
    }

    public void DirectToggle(int index)
    {
        ReceiveToggle(buttons[index]);
    }
}
