using UnityEngine;
using System.Collections;

public class StoneIdle : StateMachineBehaviour
{

    private float idle_anim_timer;

    public static float idle_anim_min_interval = 5;
    public static float idle_anim_max_interval = 10;

    // OnStateEnter is called when a transition starts and the state machine starts to evaluate this state
    override public void OnStateEnter(Animator animator, AnimatorStateInfo stateInfo, int layerIndex)
    {
        idle_anim_timer = Random.Range(EditorData.instance.stone_relax_anim_min_interval, EditorData.instance.stone_relax_anim_max_interval);
    }

    // OnStateUpdate is called on each Update frame between OnStateEnter and OnStateExit callbacks
    override public void OnStateUpdate(Animator animator, AnimatorStateInfo stateInfo, int layerIndex)
    {
        if (idle_anim_timer > 0)
        {
            idle_anim_timer -= Time.deltaTime;

            if (idle_anim_timer <= 0)
            {
                animator.SetTrigger("idle");
                animator.SetInteger("idle_type", Random.Range(1, 5));
            }
        }
    }

    // OnStateExit is called when a transition ends and the state machine finishes evaluating this state
    //override public void OnStateExit(Animator animator, AnimatorStateInfo stateInfo, int layerIndex) {
    //
    //}

    // OnStateMove is called right after Animator.OnAnimatorMove(). Code that processes and affects root motion should be implemented here
    //override public void OnStateMove(Animator animator, AnimatorStateInfo stateInfo, int layerIndex) {
    //
    //}

    // OnStateIK is called right after Animator.OnAnimatorIK(). Code that sets up animation IK (inverse kinematics) should be implemented here.
    //override public void OnStateIK(Animator animator, AnimatorStateInfo stateInfo, int layerIndex) {
    //
    //}
}
