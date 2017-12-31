using UnityEngine;
using System.Collections;

public class EditorData : MonoBehaviour {

    public static EditorData instance;

    public Sprite button_normal_sprite;
    public Sprite button_hover_sprite;
    public Sprite button_pressed_sprite;

    public float button_hover_offset;
    public float button_pressed_offset;

    public float ui_move_lerp_speed;
    public float ui_color_lerp_speed;

    public float cell_movement_time;
    public float cell_movement_lerp_speed;
    public int cell_delay_before_wake_neighbors;

    public float stone_relax_anim_min_interval;
    public float stone_relax_anim_max_interval;

    public float pointer_lerp_speed;
    public float pointer_soft_reset_delay;

    public float splash_screen_time;

    void Awake()
    {
        instance = this;
    }
}
