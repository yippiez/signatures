// Deeply nested namespaces, classes, and inner types.
namespace Outer
{
    namespace Inner
    {
        namespace Deep
        {
            public class Root
            {
                public const int RootConst = 1;

                public int RootMethod(string s) { return s.Length; }

                public class ChildA
                {
                    private static readonly string Label = "child-a";

                    public string GetLabel() => Label;

                    public struct GrandchildStruct
                    {
                        public int X;
                        public int Y;

                        public GrandchildStruct(int x, int y)
                        {
                            X = x;
                            Y = y;
                        }

                        public override string ToString() => $"({X},{Y})";
                    }

                    public enum GrandchildEnum
                    {
                        Alpha,
                        Beta,
                        Gamma
                    }
                }

                public interface IChildB
                {
                    void DoWork();
                    int ComputeValue(double input);
                }

                protected class ChildC : IChildB
                {
                    public void DoWork() { }
                    public int ComputeValue(double input) => (int)Math.Round(input);

                    private sealed class GreatGrandchild
                    {
                        public const string Tag = "ggc";
                        public void Run() { }
                    }
                }
            }
        }
    }

    namespace Sibling
    {
        public static class SiblingHelper
        {
            public static int Add(int a, int b) => a + b;
            public static int Multiply(int a, int b) => a * b;
        }

        public abstract class SiblingBase
        {
            public abstract void Execute();
            public virtual string Describe() => "sibling-base";
        }

        public class ConcreteSibling : SiblingBase
        {
            public override void Execute() { }
            public override string Describe() => "concrete-sibling";
        }
    }
}
