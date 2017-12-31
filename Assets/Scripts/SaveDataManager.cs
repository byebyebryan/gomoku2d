using System;
using UnityEngine;
using System.Collections;
using System.IO;
using System.Xml.Serialization;
//using UnityEditor;

public class SaveDataManager : MonoBehaviour
{

    public static SaveDataManager instance;

    public SaveData save_data;

    void Awake()
    {
        instance = this;
        save_data = new SaveData();
    }

    //[MenuItem("Gomoku/CreateSaveData")]
    //public static void InitProfiles()
    //{
    //    Debug.Log(Application.persistentDataPath);
    //    Debug.Log("creating new save data.");
    //    SaveData save_data = new SaveData();
    //    XmlSerializer xml = new XmlSerializer(typeof(SaveData));
    //    FileStream file = File.Open(Application.persistentDataPath + "/save_data.xml", FileMode.Create);
    //    xml.Serialize(file, save_data);
    //    file.Close();
    //}

    public void Load()
    {
        XmlSerializer xml = new XmlSerializer(typeof(SaveData));
        FileStream file;

        file = File.Open(Application.persistentDataPath + "/save_data.xml", FileMode.OpenOrCreate);
        try
        {
            save_data = (SaveData) xml.Deserialize(file);
            Debug.Log("save data loaded.");
        }
        catch (Exception e)
        {
            Debug.Log("save data load error, creating new profile.");
            save_data = new SaveData();
        }

        file.Close();
    }

    public void Save()
    {
        XmlSerializer xml = new XmlSerializer(typeof(SaveData));
        FileStream file = File.Open(Application.persistentDataPath + "/save_data.xml", FileMode.Create);
        xml.Serialize(file, save_data);
        file.Close();
    }

	// Use this for initialization
	void Start () {
	    //Load();
        
	}
	
	// Update is called once per frame
	void Update () {
	
	}

    void OnApplicationQuit()
    {
        Save();
        Debug.Log("player profiles saved.");
    }
}
