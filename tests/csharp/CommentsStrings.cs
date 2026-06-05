// CommentsStrings.cs — verifies that fake declarations inside comments and
// string literals of every flavour are silently ignored.

// class FakeLineCommentClass { void FakeMethod() {} }
// interface IFakeLine { int Compute(); }
// const int FAKE_LINE_CONST = 99;

/* class FakeBlockClass { public void Go() {} } */
/*
 * Multi-line block comment containing fake signatures:
 * public struct FakeStruct { int X; }
 * public enum FakeEnum { A, B }
 * public const double FakeConst = 3.14;
 */

/** <summary>class XmlDocFake { }</summary> */

namespace Demo.CommentsStrings
{
    /// <summary>
    /// This XML doc comment must NOT produce any signatures:
    /// <code>
    /// class DocFakeClass { void DocFakeMethod() {} }
    /// interface IDocFake { string Get(); }
    /// </code>
    /// </summary>
    public class RealClass
    {
        // Ordinary string — fake decls inside must be ignored.
        private const string HelpText =
            "class NotReal { void AlsoNotReal() {} } const int Nope = 0;";

        // Verbatim string — fake decls must also be ignored.
        private static readonly string SqlQuery =
            @"SELECT class, interface FROM schema WHERE enum = 'record'
              AND struct > 0";

        // Triple-quoted (C# 11 raw) string literal with fake decls.
        private static readonly string RawLiteral = """
            class InsideRaw { public void Method() {} }
            interface IInsideRaw { void Foo(); }
            const int RAW_CONST = 42;
            """;

        // Interpolated string — the expressions are real code but the text isn't.
        private string _tag = "real";
        public string Describe(int n) =>
            $"class count: {n} interface count: {n - 1} struct count: 0";

        /// <summary>A real method — it must appear in output.</summary>
        public void RealMethod() { }

        /// <summary>Real constant.</summary>
        public const int RealConst = 7;
    }

    // Line-comment-terminated interface — only the real interface below counts.
    // interface ILineFake { void Phantom(); }

    /// <summary>Real interface.</summary>
    public interface IRealInterface
    {
        /// <summary>
        /// enum FakeInDoc { X }  — must be ignored (inside XML doc)
        /// </summary>
        void RealWork();
        int RealCompute(double value);
    }

    public enum RealEnum
    {
        First,
        Second,
        Third
    }

    public struct RealStruct
    {
        public int Value;
        public RealStruct(int value) { Value = value; }

        /* struct FakeInsideBlockComment { } */
        public override string ToString() => Value.ToString();
    }

    public static class StringHelper
    {
        public static string Repeat(string s, int times) =>
            string.Concat(Enumerable.Repeat(s, times));

        public static bool Contains(string haystack, string needle) =>
            haystack.Contains(needle, StringComparison.OrdinalIgnoreCase);
    }
}
