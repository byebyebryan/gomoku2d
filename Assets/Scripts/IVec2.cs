using UnityEngine;
using System;
using System.Collections;

[Serializable]
public struct IVec2
{
    public int x;
    public int y;

    public IVec2(int _x, int _y)
    {
        x = _x;
        y = _y;
    }

    public bool CheckInBound(IVec2 bottom_left, IVec2 top_right)
    {
        return x >= bottom_left.x && y >= bottom_left.y && x < top_right.x && y < top_right.y;
    }

    public bool CheckInBound(IVec2 top_right)
    {
        return CheckInBound(new IVec2(0, 0), top_right);
    }

    public static IVec2 Origin()
    {
        return new IVec2(0, 0);
    }

    public static IVec2[] Directions()
    {
        return new IVec2[]
        {
             new IVec2(0,1), new IVec2(1,1), new IVec2(1, 0), new IVec2(1, -1),
             new IVec2(0,-1), new IVec2(-1,-1), new IVec2(-1, 0), new IVec2(-1,1)
        };
    }

    public static IVec2 operator +(IVec2 lhs, IVec2 rhs)
    {
        return new IVec2(lhs.x + rhs.x, lhs.y + rhs.y);
    }

    public static IVec2 operator -(IVec2 lhs, IVec2 rhs)
    {
        return new IVec2(lhs.x - rhs.x, lhs.y - rhs.y);
    }

    public static IVec2 operator *(IVec2 lhs, int rhs)
    {
        return new IVec2(lhs.x * rhs, lhs.y * rhs);
    }

    public override string ToString()
    {
        return "(" + x.ToString() + ", " + y.ToString() + ")";
    }
}
