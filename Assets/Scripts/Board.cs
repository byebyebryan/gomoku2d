using UnityEngine;
using System.Collections;
using System.Collections.Generic;

public class Board : MonoBehaviour
{
    public static Board instance;

    public static int board_size = 15;
    public static int board_size_minus_one = board_size - 1;
    public static float board_half_size = board_size_minus_one/2f;

    public Cell[,] cells;

    public Cell cell_prefab;

    public void CreateCells()
    {
        cells = new Cell[board_size,board_size];

        float half_size = (board_size - 1) /2f;

        for (int x = 0; x < board_size; x++)
        {
            for (int y = 0; y < board_size; y ++)
            {
                Cell cell = Instantiate(cell_prefab);
                cell.transform.SetParent(this.transform);
                cell.transform.localPosition = new Vector3(x - half_size, y - half_size + Camera.main.orthographicSize * 2 + 1f, 0f);
                cell.coord = new IVec2(x, y);

                cells[x, y] = cell;

                cell.GridLineInit();
                cell.SetRenderOrders();
            }
        }

        for (int x = 0; x < board_size; x++)
        {
            for (int y = 0; y < board_size; y++)
            {
                Cell cell = cells[x, y];

                cell.ConnectNeighors();
            }
        }
    }

    public void ShowBoard()
    {
        foreach (Cell cell in cells)
        {
            cell.ResetCell();
        }

        cells[0, 0].NotifySequenceMovement();
    }

    public void HideBoard()
    {
        foreach (Cell cell in cells)
        {
            cell.SetHideMovementTarget();
        }

        cells[0, 0].NotifySequenceMovement();
    }

    public void ResetBoard()
    {
        foreach (Cell cell in cells)
        {
            cell.ResetCell();
            cell.NotifyMovement();
        }
    }

    public void PlaceStone(Cell cell)
    {
        if (cell.cell_color == CellColor.Empty)
        {
            cell.PlaceStone(Game.instance.current_color);
        }
    }

    public static bool CheckInBound(IVec2 coord)
    {
        return coord.CheckInBound(new IVec2(board_size, board_size));
    }

    void Awake()
    {
        Board.instance = this;
    }
}
