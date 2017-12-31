using UnityEngine;
using System.Collections;

public class Stone : MonoBehaviour
{
    private Animator animator;
    private SpriteRenderer sprite_renderer;

    public void RequestShowAnimation()
    {
        animator.SetTrigger("show");
    }

    public void RequestHideAnimation()
    {
        animator.SetTrigger("hide");
    }

    public void SetColor(CellColor color)
    {
        sprite_renderer.color = CellColorUtil.GetSpriteColor(color);
    }

    void Awake()
    {
        animator = GetComponent<Animator>();
        sprite_renderer = GetComponent<SpriteRenderer>();
    }

	// Use this for initialization
	void Start ()
	{
    }
    
	// Update is called once per frame
	void Update ()
    {
	}
}
