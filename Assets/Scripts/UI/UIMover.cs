using UnityEngine;
using System.Collections;

public class UIMover : MonoBehaviour
{

    public RectTransform target = null;

    public RectTransform resting_target;
    public RectTransform active_target;
    public RectTransform alt_target;

    private RectTransform rect;

    public bool lerp_speed_override;
    public float lerp_speed;

    public void ForceRestingPosition()
    {
        rect.anchoredPosition = resting_target.anchoredPosition;
        SetRestingPosition();
    }

    public void SetRestingPosition()
    {
        target = resting_target;
    }

    public void SetActivePosition()
    {
        target = active_target;
    }

    public void SetAltPosition()
    {
        target = alt_target;
    }

    void Awake()
    {
        rect = GetComponent<RectTransform>();
    }

    void Start()
    {
        //ForceRestingPosition();
        if (!lerp_speed_override)
        {
            lerp_speed = EditorData.instance.ui_move_lerp_speed;
        }
    }

	// Update is called once per frame
	void Update ()
	{
	    if (target != null)
	    {
            rect.anchoredPosition = Vector2.Lerp(rect.anchoredPosition, target.anchoredPosition, lerp_speed);
        }
	}
}
