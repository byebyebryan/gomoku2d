using UnityEngine;
using System.Collections;
using UnityEngine.EventSystems;
using System;

public class Cell : MonoBehaviour, IPointerEnterHandler, IPointerExitHandler, IPointerDownHandler, IPointerUpHandler {
    public IVec2 coord;
    public CellColor cell_color;

    public Cell[] neighbors;
    public GridLine[] connected_lines;

    public GridLine gridline_prefab;

    public GameObject edge;
    public GameObject surface;
    
    public GameObject warning_surface;
    public GameObject warning_floater;

    public Vector3 target_pos;

    public GameObject stone;

    private Animator stone_animator;

    private float movement_timer;
    private float delay_timer;

    public void ResetCell()
    {
        RemoveStone();
        cell_color = CellColor.Empty;
        SetNormalMovementTarget();
        warning_surface.gameObject.SetActive(false);
    }

    public void SetHideMovementTarget()
    {
        target_pos = new Vector3(coord.x - Board.board_half_size, coord.y - Board.board_half_size + Camera.main.orthographicSize * 2, 0f);
    }

    public void SetNormalMovementTarget()
    {
        target_pos = new Vector3(coord.x - Board.board_half_size, coord.y - Board.board_half_size, 0f);
    }

    public void SetWinningPose()
    {
        target_pos = target_pos + Vector3.up * 0.25f;
        NotifyMovement();
        warning_surface.SetActive(true);
    }

    public void NotifyMovement()
    {
        movement_timer = EditorData.instance.cell_movement_time;
    }

    public void NotifySequenceMovement()
    {
        NotifyMovement();
        delay_timer = EditorData.instance.cell_delay_before_wake_neighbors;
    }

    void Awake()
    {
        stone_animator = stone.GetComponent<Animator>();
    }

    void Start()
    {
        cell_color = CellColor.Empty;
        SetNormalMovementTarget();
        warning_surface.gameObject.SetActive(false);
        //delay_timer = delay_before_wake_neighbors * ( 1f - (coord.x + coord.y) * (coord.x + coord.y) / ((2 * (Board.board_size - 1)) *(2*(Board.board_size - 1))));
        //delay_timer = Mathf.SmoothStep(0.01f, delay_before_wake_neighbors, (coord.x + coord.y)/(2*(Board.board_size - 1)));
        //delay_timer = delay_before_wake_neighbors;
    }

    void Update()
    {
        if (movement_timer > 0)
        {
            transform.localPosition = Vector3.Lerp(transform.localPosition, target_pos, EditorData.instance.cell_movement_lerp_speed);

            if (delay_timer > 0)
            {
                delay_timer -= 1;
                if (delay_timer <= 0)
                {
                    if (neighbors[0] != null)
                    {
                        neighbors[0].NotifySequenceMovement();
                    }
                    if (neighbors[2] != null)
                    {
                        neighbors[2].NotifySequenceMovement();
                    }
                }
            }

            movement_timer -= Time.deltaTime;
            if (movement_timer <= 0)
            {
                transform.localPosition = target_pos;
            }
        }
    }

    public void PlaceStone(CellColor color)
    {
        if (color == CellColor.Empty)
        {
            Debug.LogError("Try to place EMPTY stone at " + coord.ToString());
            return;
        }

        cell_color = color;
        stone_animator.SetInteger("color", (int)color);
        stone_animator.SetTrigger("show");
    }

    public void RemoveStone()
    {
        cell_color = CellColor.Empty;
        stone_animator.SetTrigger("hide");
    }

    public void OnPointerEnter(PointerEventData eventData)
    {
        Pointer.instance.OnEnterCell(this);
    }

    public void OnPointerExit(PointerEventData eventData)
    {
        Pointer.instance.OnExitCell(this);
    }

    public void OnPointerDown(PointerEventData eventData)
    {
        Pointer.instance.OnPressDown();
    }

    public void OnPointerUp(PointerEventData eventData)
    {
        Pointer.instance.OnPressUp();
    }

    public void GridLineInit()
    {
        connected_lines = new GridLine[4];

        if (coord.y < Board.board_size_minus_one)
        {
            GridLine line = Instantiate(gridline_prefab);
            connected_lines[0] = line;
            line.transform.SetParent(this.transform);
            line.transform.localPosition = Vector3.zero;
            line.transform.localEulerAngles = new Vector3(0, 0, 0);
            line.transform.localScale = new Vector3(1f, 1f, 1f);
            line.source = this;
            line.direction = IVec2.Directions()[0];
        }

        if (coord.x < Board.board_size_minus_one)
        {
            GridLine line = Instantiate(gridline_prefab);
            connected_lines[1] = line;
            line.transform.SetParent(this.transform);
            line.transform.localPosition = Vector3.zero;
            line.transform.localEulerAngles = new Vector3(0, 0, -90f);
            line.transform.localScale = new Vector3(-1f, 1f, 1f);
            line.source = this;
            line.direction = IVec2.Directions()[2];
        }     

        if (coord.y > 0)
        {
            GridLine line = Instantiate(gridline_prefab);
            connected_lines[2] = line;
            line.transform.SetParent(this.transform);
            line.transform.localPosition = Vector3.zero;
            line.transform.localEulerAngles = new Vector3(0, 0, 180f);
            line.transform.localScale = new Vector3(1f, 1f, 1f);
            line.source = this;
            line.direction = IVec2.Directions()[4];
        }

        if (coord.x > 0)
        {
            GridLine line = Instantiate(gridline_prefab);
            connected_lines[3] = line;
            line.transform.SetParent(this.transform);
            line.transform.localPosition = Vector3.zero;
            line.transform.localEulerAngles = new Vector3(0, 0, 90f);
            line.transform.localScale = new Vector3(-1f, 1f, 1f);
            line.source = this;
            line.direction = IVec2.Directions()[6];
        }
    }

    public void SetRenderOrders()
    {
        surface.GetComponent<SpriteRenderer>().sortingOrder = -coord.y * 10;
        edge.GetComponent<SpriteRenderer>().sortingOrder = -coord.y * 10;

        foreach (GridLine line in connected_lines)
        {
            if (line != null)
            {
                line.GetComponent<SpriteRenderer>().sortingOrder = -coord.y * 10 + 1;
            }
        }

        warning_surface.GetComponent<SpriteRenderer>().sortingOrder = -coord.y * 10 + 3;

        stone.GetComponent<SpriteRenderer>().sortingOrder = -coord.y * 10 + 6;
    }

    public void ConnectNeighors()
    {
        neighbors = new Cell[8];

        for (int d = 0; d < 8; d++)
        {
            IVec2 n_coord = coord + IVec2.Directions()[d];
            if (Board.CheckInBound(n_coord))
            {
                neighbors[d] = Board.instance.cells[n_coord.x, n_coord.y];
            }
        }

        for (int d = 0; d < 4; d++)
        {
            GridLine line = connected_lines[d];
            if (line != null)
            {
                IVec2 target_coord = coord + line.direction;
                line.target_line = Board.instance.cells[target_coord.x, target_coord.y].connected_lines[(1+d%2)*2 - d];
            }
        }
    }
}
