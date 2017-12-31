using UnityEngine;
using System.Collections;
using System.Collections.Generic;

public class Gomoku {

    public static bool CheckForWin(Cell cell, out List<Cell> line)
    {
        line = new List<Cell>();

        for (int d = 0; d < 4; d++)
        {
            line.Clear();

            line.Add(cell);

            Cell n_cell = cell.neighbors[d];
            while (n_cell != null && n_cell.cell_color == cell.cell_color)
            {
                line.Add(n_cell);
                n_cell = n_cell.neighbors[d];
            }

            n_cell = cell.neighbors[d + 4];
            while (n_cell != null && n_cell.cell_color == cell.cell_color)
            {
                line.Add(n_cell);
                n_cell = n_cell.neighbors[d + 4];
            }

            if (line.Count >= 5)
            {
                break;
            }
        }

        if (line.Count < 5)
        {
            line.Clear();
        }

        return line.Count >= 5;
    }
}
