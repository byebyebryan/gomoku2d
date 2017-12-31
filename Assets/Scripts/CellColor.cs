using UnityEngine;
using System.Collections;

public enum CellColor
{
    Empty, Black, White
}

public class CellColorUtil
{
    public static CellColor GetCellColor(int index)
    {
        return (CellColor) index;
    }

    public static int GetIndex(CellColor cell_color)
    {
        return (int) cell_color;
    }

    public static Color GetSpriteColor(CellColor cell_color)
    {
        if (cell_color == CellColor.Empty)
        {
            return new Color32(127, 255, 145, 255);
        }
        else if (cell_color == CellColor.White)
        {
            return Color.white;
        }
        else
        {
            return new Color(0.25f, 0.25f, 0.25f);
        }
    }

    public static CellColor GetReverseCellColor(CellColor color)
    {
        return color == CellColor.Empty? CellColor.Empty : (CellColor) (3 - (int) color);
    }
}
