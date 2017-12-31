using UnityEngine;
using System.Collections;

public class Pointer : MonoBehaviour
{
    public static Pointer instance;

    public Cell target_cell;
    public int grid_row;

    private Vector3 target_pos;
    private float lerp_speed;
    private float soft_reset_timer;

    private Animator animator;
    private SpriteRenderer sprite_renderer;

    public void HardReset()
    {
        target_cell = null;
        soft_reset_timer = 0;
        transform.position = new Vector3(0f, Camera.main.orthographicSize + 1f, 0f);
        target_pos = new Vector3(0f, Camera.main.orthographicSize + 1f, 0f);

        sprite_renderer.color = CellColorUtil.GetSpriteColor(Game.instance.current_color);

        animator.SetBool("sp_animation", false);
    }

    public void SoftReset()
    {
        target_cell = null;
        soft_reset_timer = 0;
        animator.SetBool("sp_animation", false);
        sprite_renderer.color = CellColorUtil.GetSpriteColor(Game.instance.current_color);
        target_pos = new Vector3(transform.position.x, Camera.main.orthographicSize + 1f, 0f);
    }

    public void OnEnterCell(Cell cell)
    {
        target_cell = cell;
        animator.SetBool("non_empty_cell", cell.cell_color != CellColor.Empty);
        target_pos = cell.transform.position;
    }

    public void OnExitCell(Cell cell)
    {
        target_cell = null;
        soft_reset_timer = EditorData.instance.pointer_soft_reset_delay;
    }

    public void OnPressDown()
    {
        animator.SetBool("sp_animation", true);
    }

    public void OnPressUp()
    {
        if (Game.instance.game_ended)
        {
            Game.instance.GameReset();
            return;
        }

        if (target_cell)
        {
            if (target_cell.cell_color == CellColor.Empty)
            {
                Game.instance.ReceivingLastMove(target_cell);
            }
        }
    }

    void Awake()
    {
        instance = this;
        animator = GetComponent<Animator>();
        sprite_renderer = GetComponent<SpriteRenderer>();
        HardReset();
    }

    // Use this for initialization
    void Start ()
	{
	}
	
	// Update is called once per frame
	void Update ()
    {
	    if (target_cell == null && soft_reset_timer > 0)
	    {
	        soft_reset_timer -= Time.deltaTime;
	        if (soft_reset_timer <= 0)
	        {
                SoftReset();
	        }
	    }

	    if (transform.position.y > Camera.main.orthographicSize)
	    {
	        transform.position = new Vector3(target_pos.x, transform.position.y, 0f);
	    }

	    transform.position = Vector3.Lerp(transform.position, target_pos, EditorData.instance.pointer_lerp_speed);

        grid_row = Mathf.Clamp(Mathf.RoundToInt(transform.position.y - 0.4f + (Board.board_size - 1) / 2), 0, Board.board_size - 1);
	    sprite_renderer.sortingOrder = -grid_row * 10 + 4;
    }

    
}
