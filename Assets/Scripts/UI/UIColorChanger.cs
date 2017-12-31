using UnityEngine;
using System.Collections;
using UnityEngine.UI;

public class UIColorChanger : MonoBehaviour {

    public Image image;
    public Text[] texts;

    public CellColor cell_color;

    void Awake()
    {
        image = GetComponent<Image>();
        texts = GetComponentsInChildren<Text>();
    }

    // Use this for initialization
    void Start () {
	    
	}
	
	// Update is called once per frame
	void Update () {
        Color color = image.color;
        Color target_color = CellColorUtil.GetSpriteColor(cell_color);
        image.color = Color.Lerp(color, target_color, EditorData.instance.ui_color_lerp_speed);

        foreach (Text text in texts)
        {
            color = text.color;
            target_color = CellColorUtil.GetSpriteColor(CellColorUtil.GetReverseCellColor(cell_color));
            text.color = Color.Lerp(color, target_color, EditorData.instance.ui_color_lerp_speed);
        }
    }
}
